# Changelog

All notable changes to this project will be documented in this file.

## [0.9.2] - 2026-04-09

### Added

- Added configurable timezone offsets via `--tz` and `config.toml`
- Added sunrise, sunset, moonrise, and moonset azimuth display
- Added moon event highlighting for `Supermoon`, `Micromoon`, and `Blue Moon`
- Added a dedicated `Timezone` / `時差` output line
- Added bilingual documentation with separate English and Japanese READMEs

### Changed

- Changed moon-event highlighting to use the event center day plus/minus one day
- Changed English output separators to ASCII-friendly formatting
- Changed Japanese label alignment to use display width instead of raw string length
- Changed sample documentation to reflect current output and timezone behavior

## [0.9.1] - 2026-04-08

### Changed

- Refined CLI output layout and multilingual display
- Improved event display formatting and moon rise/set ordering

## [0.9.0] - 2026-04-08

### Added

- Initial public release
- Moon age, illumination, distance, phase, rise/set, and ASCII moon art
- Config file support for default latitude, longitude, language, and art settings
