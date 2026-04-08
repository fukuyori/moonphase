# moon

[日本語 README](README.ja.md)

`moon` is a command-line tool that shows moon age, phase, sunrise, sunset, moonrise, moonset, and optional ASCII moon art.

See also: [CHANGELOG](CHANGELOG.md)

## Features

- Calculates moon age, illumination, and distance at `00:00` in the selected timezone
- Shows `sunrise / sunset / moonrise / moonset` for a chosen location
- Lets you enable or disable ASCII art
- Supports both Japanese and English output
- Works with defaults even when no config file exists

## Build

```bash
cargo build --release
```

The binary will be created at `target/release/moon`. On Windows, the file is `moon.exe`.

## Usage

```bash
# Today's moon data (selected timezone)
moon

# Relative dates
moon prev
moon next
moon today

# Specific date
moon 2026-04-08

# Specific date and coordinates
moon 2026-04-08 35.6762 139.6503

# Option form
moon --date 2026-04-08 --lat 35.6762 --lon 139.6503

# Timezone offset
moon --tz 09:00
moon --date today --tz -05:00

# Toggle ASCII art
moon --art
moon --no-art

# Switch display language
moon --lang ja
moon --lang en

# Help and version
moon --help
moon --version
```

## Options

- `--date <YYYY-MM-DD|today|prev|next>`
- `--lat <latitude>`
- `--lon <longitude>`
- `--art`
- `--no-art`
- `--lang <ja|en>`
- `--tz <09:00|+09:00|-05:00|UTC>`
- `--help`
- `--version`

You can also use positional arguments in the form `moon YYYY-MM-DD LAT LON`.

## Config File

The config file is optional. If it does not exist, the app runs with built-in defaults.

- Default latitude: `35.6762`
- Default longitude: `139.6503`
- Default ASCII art: `true`
- Default language: `ja`
- Default timezone: `09:00`

Config file location:

- Windows: `%APPDATA%\moon\config.toml`
- macOS: `~/Library/Application Support/moon/config.toml`
- Linux: `$XDG_CONFIG_HOME/moon/config.toml`
  If unset, `~/.config/moon/config.toml`

Example:

```toml
lat = 35.6762
lon = 139.6503
art = true
lang = "en"
tz = "09:00"
```

Command-line values override the config file.

## Example Output

```text
    Timezone : UTC+09:00
    Age      : 19.57 days
    Illum.   : 73.7%
    Phase    : Nemachi Moon
    Distance : 404933 km
    Sunrise  : 05:20 (081°)
    Sunset   : 18:08 (280°)
    Moonset  : 07:50 (227°)
    Moonrise : 23:20 (119°)
```

## Sample Run

Example command:

```bash
moon --lang en --no-art 2026-04-08
```

Sample output:

```text
    Timezone : UTC+09:00
    Age      : 19.57 days
    Illum.   : 73.7%
    Phase    : Nemachi Moon
    Distance : 404933 km
    Sunrise  : 05:20 (081°)
    Sunset   : 18:08 (280°)
    Moonset  : 07:50 (227°)
    Moonrise : 23:20 (119°)
```

Example command with ASCII art:

```bash
moon --lang en 2026-04-08
```

Sample output:

```text
                          ・                      
                 ░▒▓▓▓▓▓▓▓▓▓▓▓▒▒░░ ・・            
             ░▓▓▓███████████▓▓▓▓▒▒▒░░・・・・        
          ░▓▓█████████████████▓▓▓▓▒▒▒░░・・ ・      
       ・▒▓▓█████████████████████▓▓▓▒▒▒░░░・・ ・・   
      ░▓▓████████████████████████▓▓▓▒▒▒░░░・・・ ・  
     ░▓▓██████████████████████████▓▓▓▒▒▒░░░・・・ ・ 
    ░▓▓███████████████████████████▓▓▓▓▒▒▒░░░・・・ ・
    ▓▓████████████████████████████▓▓▓▓▒▒▒░░░░・・・ 
    ▓██████████████████████████████▓▓▓▓▒▒▒░░░・・・ 
    ▓██████████████████████████████▓▓▓▓▒▒▒░░░・・・ 
    ▓██████████████████████████████▓▓▓▒▒▒▒░░░・・・ 
    ▓▓█████████████████████████████▓▓▓▒▒▒░░░░・・・ 
    ░▓▓███████████████████████████▓▓▓▓▒▒▒░░░・・・ ・
     ░▓▓██████████████████████████▓▓▓▒▒▒░░░・・・ ・ 
      ░▓▓████████████████████████▓▓▓▒▒▒░░░・・・ ・  
       ・▒▓▓█████████████████████▓▓▓▒▒▒░░░・・ ・・   
          ░▓▓██████████████████▓▓▓▒▒▒░░・・ ・      
             ░▓▓▓████████████▓▓▓▒▒▒░░・・・・        
                 ░▒▓▓▓▓▓▓▓▓▓▓▓▒▒░░ ・・            
                          ・                      

    Timezone : UTC+09:00
    Age      : 19.57 days
    Illum.   : 73.7%
    Phase    : Nemachi Moon
    Distance : 404933 km
    Sunrise  : 05:20 (081°)
    Sunset   : 18:08 (280°)
    Moonset  : 07:50 (227°)
    Moonrise : 23:20 (119°)
```

## Notes

- Time values are calculated in the selected timezone.
- Moonrise and moonset are filtered so only events belonging to the selected local date are shown.
- Astronomical calculations prefer the `astro` crate when available.
