use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use clap::Parser;
use anvil_region::AnvilChunk;
use fastnbt::{Value, from_bytes};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "dog-whistle")]
#[command(about = "Find lost tamed wolves in Minecraft worlds")]
#[command(version = "0.1.0")]
struct Args {
    /// Path to the Minecraft world directory
    #[arg(short, long)]
    world: PathBuf,
    
    /// Player UUIDs to search for (can specify multiple)
    #[arg(short, long, value_delimiter = ',')]
    players: Vec<String>,
    
    /// Only show sitting wolves
    #[arg(short, long)]
    sitting_only: bool,
    
    /// Show additional debug information
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug)]
struct WolfInfo {
    pos: [f64; 3],
    owner_uuid: String,
    sitting: bool,
    health: f32,
    region_file: String,
    chunk_x: i32,
    chunk_z: i32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate world directory
    let world_path = &args.world;
    if !world_path.exists() {
        anyhow::bail!("World directory does not exist: {}", world_path.display());
    }
    
    let region_dir = world_path.join("region");
    if !region_dir.exists() {
        anyhow::bail!("Region directory not found: {}", region_dir.display());
    }
    
    // Validate UUIDs
    let target_uuids: Vec<String> = args.players.iter()
        .map(|uuid_str| {
            // Try to parse and reformat UUID to ensure consistency
            match Uuid::parse_str(uuid_str) {
                Ok(uuid) => Ok(uuid.to_string()),
                Err(_) => {
                    // Try without hyphens
                    if uuid_str.len() == 32 {
                        let formatted = format!(
                            "{}-{}-{}-{}-{}",
                            &uuid_str[0..8],
                            &uuid_str[8..12],
                            &uuid_str[12..16],
                            &uuid_str[16..20],
                            &uuid_str[20..32]
                        );
                        match Uuid::parse_str(&formatted) {
                            Ok(uuid) => Ok(uuid.to_string()),
                            Err(_) => anyhow::bail!("Invalid UUID format: {}", uuid_str),
                        }
                    } else {
                        anyhow::bail!("Invalid UUID format: {}", uuid_str)
                    }
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;
    
    if args.verbose {
        println!("Searching for wolves owned by: {:?}", target_uuids);
        println!("World path: {}", world_path.display());
    }
    
    let mut found_wolves = Vec::new();
    
    // Read all region files
    let region_files = fs::read_dir(&region_dir)
        .context("Failed to read region directory")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "mca" {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    
    println!("Scanning {} region files...", region_files.len());
    
    for region_file in region_files {
        if args.verbose {
            println!("Processing: {}", region_file.display());
        }
        
        match search_region_file(&region_file, &target_uuids, args.sitting_only) {
            Ok(wolves) => {
                found_wolves.extend(wolves);
            }
            Err(e) => {
                if args.verbose {
                    eprintln!("Error processing {}: {}", region_file.display(), e);
                }
            }
        }
    }
    
    // Display results
    if found_wolves.is_empty() {
        println!("No wolves found for the specified players.");
    } else {
        println!("\nFound {} wolves:", found_wolves.len());
        println!("{:<15} {:<40} {:<8} {:<8} {:<15}", "Coordinates", "Owner UUID", "Sitting", "Health", "Region");
        println!("{}", "-".repeat(100));
        
        for wolf in found_wolves {
            println!(
                "{:<15} {:<40} {:<8} {:<8.1} {:<15}",
                format!("{:.0}, {:.0}, {:.0}", wolf.pos[0], wolf.pos[1], wolf.pos[2]),
                wolf.owner_uuid,
                if wolf.sitting { "Yes" } else { "No" },
                wolf.health,
                wolf.region_file
            );
            
            if args.verbose {
                println!("  -> Chunk: ({}, {})", wolf.chunk_x, wolf.chunk_z);
            }
        }
        
        println!("\nTo teleport to a wolf, use: /tp @s {:.0} {:.0} {:.0}", 
                 found_wolves[0].pos[0], found_wolves[0].pos[1], found_wolves[0].pos[2]);
    }
    
    Ok(())
}

fn search_region_file(region_path: &Path, target_uuids: &[String], sitting_only: bool) -> Result<Vec<WolfInfo>> {
    let region_file = std::fs::File::open(region_path)
        .with_context(|| format!("Failed to open region file: {}", region_path.display()))?;
    
    let mut region = anvil_region::Region::from_stream(region_file)
        .with_context(|| format!("Failed to parse region file: {}", region_path.display()))?;
    
    let mut wolves = Vec::new();
    let region_name = region_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Iterate through all chunks in the region
    for chunk_x in 0..32 {
        for chunk_z in 0..32 {
            if let Ok(Some(chunk_data)) = region.read_chunk(chunk_x, chunk_z) {
                match AnvilChunk::from_bytes(&chunk_data) {
                    Ok(chunk) => {
                        if let Some(chunk_wolves) = search_chunk_for_wolves(&chunk, target_uuids, sitting_only, &region_name)? {
                            wolves.extend(chunk_wolves);
                        }
                    }
                    Err(_) => {
                        // Skip corrupted chunks
                        continue;
                    }
                }
            }
        }
    }
    
    Ok(wolves)
}

fn search_chunk_for_wolves(
    chunk: &AnvilChunk, 
    target_uuids: &[String], 
    sitting_only: bool,
    region_name: &str
) -> Result<Option<Vec<WolfInfo>>> {
    let mut wolves = Vec::new();
    
    // Get chunk coordinates
    let chunk_x = chunk.x_pos;
    let chunk_z = chunk.z_pos;
    
    // Look for entities in the chunk
    if let Some(entities) = chunk.entities.as_ref() {
        for entity in entities {
            if let Some(wolf_info) = check_entity_for_wolf(entity, target_uuids, sitting_only, region_name, chunk_x, chunk_z)? {
                wolves.push(wolf_info);
            }
        }
    }
    
    Ok(if wolves.is_empty() { None } else { Some(wolves) })
}

fn check_entity_for_wolf(
    entity: &Value,
    target_uuids: &[String],
    sitting_only: bool,
    region_name: &str,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<Option<WolfInfo>> {
    if let Value::Compound(entity_data) = entity {
        // Check if this is a wolf
        if let Some(Value::String(entity_id)) = entity_data.get("id") {
            if entity_id != "minecraft:wolf" {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }
        
        // Check if it's tamed and get owner
        let owner_uuid = if let Some(Value::String(owner)) = entity_data.get("Owner") {
            owner.clone()
        } else {
            return Ok(None); // Not tamed
        };
        
        // Check if owner is in our target list
        if !target_uuids.contains(&owner_uuid) {
            return Ok(None);
        }
        
        // Get position
        let pos = if let Some(Value::List(pos_list)) = entity_data.get("Pos") {
            if pos_list.len() != 3 {
                return Ok(None);
            }
            [
                extract_double(&pos_list[0])?,
                extract_double(&pos_list[1])?,
                extract_double(&pos_list[2])?,
            ]
        } else {
            return Ok(None);
        };
        
        // Check sitting status
        let sitting = if let Some(Value::Byte(sitting_byte)) = entity_data.get("Sitting") {
            *sitting_byte != 0
        } else {
            false
        };
        
        // If we only want sitting wolves and this one isn't sitting, skip it
        if sitting_only && !sitting {
            return Ok(None);
        }
        
        // Get health
        let health = if let Some(Value::Float(health_val)) = entity_data.get("Health") {
            *health_val
        } else {
            20.0 // Default health
        };
        
        Ok(Some(WolfInfo {
            pos,
            owner_uuid,
            sitting,
            health,
            region_file: region_name.to_string(),
            chunk_x,
            chunk_z,
        }))
    } else {
        Ok(None)
    }
}

fn extract_double(value: &Value) -> Result<f64> {
    match value {
        Value::Double(d) => Ok(*d),
        Value::Float(f) => Ok(*f as f64),
        Value::Int(i) => Ok(*i as f64),
        Value::Long(l) => Ok(*l as f64),
        _ => anyhow::bail!("Expected numeric value for position"),
    }
}