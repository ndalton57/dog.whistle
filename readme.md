# Dog Whistle

A Rust command-line tool to find lost tamed wolves (dogs) in Minecraft worlds by scanning region files directly. Perfect for server admins who need to locate pets that players accidentally left sitting somewhere in the world.

## Features

- 🐕 Find all tamed wolves owned by specific players
- 📍 Get exact coordinates for teleportation
- 🪑 Filter to show only sitting wolves
- 🗺️ Scan entire worlds without loading chunks in-game
- ⚡ Fast Rust-based scanning of .mca region files
- 🔍 Verbose mode for debugging

## Installation

### Prerequisites
- Rust (install from [rustup.rs](https://rustup.rs/))

### Build from source
```bash
git clone https://github.com/yourusername/dog-whistle.git
cd dog-whistle
cargo build --release
```

The executable will be available at `target/release/dog-whistle` (or `dog-whistle.exe` on Windows).

## Usage

### Basic usage
```bash
dog-whistle --world /path/to/minecraft/world --players uuid1,uuid2
```

### Examples

Find all wolves owned by a specific player:
```bash
dog-whistle --world "./my_minecraft_world" --players "550e8400-e29b-41d4-a716-446655440000"
```

Find only sitting wolves (likely the lost ones):
```bash
dog-whistle --world "./world" --players "550e8400-e29b-41d4-a716-446655440000" --sitting-only
```

Search for multiple players with verbose output:
```bash
dog-whistle --world "./world" --players "uuid1,uuid2,uuid3" --verbose
```

### Getting Player UUIDs

You can get player UUIDs in several ways:

1. **In-game command**: `/data get entity @p UUID`
2. **Online lookup**: Use websites like [mcuuid.net](https://mcuuid.net/)
3. **Server logs**: UUIDs appear when players join
4. **Player data files**: Check `world/playerdata/` directory (filenames are UUIDs)

### Command Line Options

- `--world, -w`: Path to the Minecraft world directory (required)
- `--players, -p`: Comma-separated list of player UUIDs to search for (required)
- `--sitting-only, -s`: Only show wolves that are sitting (optional)
- `--verbose, -v`: Show additional debug information (optional)
- `--help, -h`: Show help information

## Output

The tool will display a table showing:
- **Coordinates**: X, Y, Z position of each wolf
- **Owner UUID**: Which player owns the wolf
- **Sitting**: Whether the wolf is sitting
- **Health**: Current health of the wolf
- **Region**: Which region file contains the wolf

Example output:
```
Found 2 wolves:
Coordinates     Owner UUID                               Sitting  Health   Region         
----------------------------------------------------------------------------------------------------
-1234, 64, 5678 550e8400-e29b-41d4-a716-446655440000     Yes      20.0     r.-1.0.mca     
-987, 72, 321   550e8400-e29b-41d4-a716-446655440000     No       18.5     r.0.0.mca      

To teleport to a wolf, use: /tp @s -1234 64 5678
```

## How It Works

This tool works by:

1. Reading Minecraft region files (`.mca`) directly from disk
2. Parsing the NBT (Named Binary Tag) data structure
3. Searching through all entities in all chunks
4. Filtering for wolves (`minecraft:wolf`) with the specified owner UUIDs
5. Extracting position, sitting state, and other relevant data

Unlike in-game commands, this method can find entities in unloaded chunks across the entire world.

## Supported Minecraft Versions

This tool works with Minecraft worlds from version 1.13+ (when the current NBT format was introduced). It should work with most modern versions including:
- Minecraft Java Edition 1.13+
- Most modded servers (as long as they use standard region file format)

## Performance

- Scanning typically takes 10-30 seconds for most worlds
- Large worlds (100+ GB) may take several minutes
- Memory usage is minimal (usually under 100MB)
- Uses multiple CPU cores for parallel chunk processing

## Contributing

Contributions are welcome! Please feel free to:
- Report bugs via GitHub Issues
- Submit feature requests
- Create pull requests with improvements

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Troubleshooting

### "World directory does not exist"
- Make sure you're pointing to the world folder (contains `level.dat`)
- Check that the path is correct and accessible

### "Region directory not found"
- Ensure the world has been played in (generates region files)
- Check that `world/region/` directory exists

### "Invalid UUID format"
- UUIDs can be with or without hyphens
- Example valid formats: `550e8400-e29b-41d4-a716-446655440000` or `550e8400e29b41d4a716446655440000`

### No wolves found
- Double-check the player UUIDs are correct
- Try running without `--sitting-only` to find all wolves
- Use `--verbose` to see which regions are being processed
