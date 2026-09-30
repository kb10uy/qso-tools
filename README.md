# QSO Tools

kb10uy's small toolbox to manage QSOs and QSL cards.
All tools work on ADIF only; use `wavelog-tools export` to get ADIF from Wavelog.

## Configuration

Settings are read from `qso-tools/config.toml` in the user config directory
(`$XDG_CONFIG_HOME` or `~/.config` on Linux and macOS, `%APPDATA%` on Windows).
Use `-c/--config` to specify another file. See `assets/config.example.toml`.

- `[operators.<CALLSIGN>]`: display name of operator

`instruments.toml`, `parks.toml`, `japan-jcx.toml`, `japan-jcx-town.toml` and `subdivisions.toml` placed next to `config.toml` are also loaded by `qcgen` and `jcx` (see `assets/qcgen/*.example.toml`).
`cty.dat` placed next to `config.toml` is loaded by `callsign`; get it from [Country Files](https://www.country-files.com/).

## qcgen

Generates JSON data for QSL cards through a Lua script.

```sh
# QSOs requesting QSL cards from Wavelog
wavelog-tools export -f qsl_required --qso-since 2026-01-01 | qso-tools qcgen -l codepoints assets/qcgen/qslcard-single.lua

# From local ADIF file
qso-tools qcgen assets/qcgen/qslcard-single.lua --adif qsos.adi
```

- QSOs are read from `--adif` file (ADX if `.adx`) or ADI from stdin; all QSOs in the input are processed
- ADI data lengths are counted in bytes by default; use `-l codepoints` for ADI from Wavelog, which counts non-ASCII text in codepoints
- Station information comes from `STATION_CALLSIGN` and `MY_*` fields
- `MY_STATE` is passed as `station.location.state`; its name is looked up by `MY_DXCC` from `subdivisions.toml` (written by `wavelog-tools subdivisions`)
- `MY_CNTY` is passed as `station.location.county` as is; if `MY_DXCC` is Japan (339) and it is a JCC/JCG code (with optional HAMLOG town suffix like `15006C`) found in `japan-jcx.toml`, its names (and town name from `japan-jcx-town.toml`) are passed as `station.location.japan_jcx`
- Operator comes from `OPERATOR`; its display name is looked up from `[operators]` in config
- `QSL_VIA` and `QSL_SENT_VIA` are passed as `qsl.via` and `qsl.sent_via`
- `MY_POTA_REF` is split into `station.references.pota` (reference, location and names from `parks.toml`)
- `!inst:<key>` in `COMMENT` (or `-I <key>`) selects instrument from `instruments.toml` and `-i` files; power is taken from `TX_PWR`, `--power`, then `default_power` of instrument

See `assets/qcgen/` for example files and `assets/schope-types/` for Lua type definitions.

## jcx

Shows full Japanese names of JCC/JCG codes (with optional HAMLOG town suffix like `15006C`) from `japan-jcx.toml` and `japan-jcx-town.toml`.

```sh
qso-tools jcx 100101 01006A
```

- Each code is printed with its full name separated by a tab, e.g. `01006A` → `北海道虻田郡京極町`
- Exits with an error after printing the others if any code is unknown

## callsign

Shows DXCC entities of callsigns from AD1C `cty.dat`.

```sh
qso-tools callsign JL1HIS W1AW/KH6
```

- Each callsign is printed with entity name, primary prefix, continent, CQ zone and ITU zone separated by tabs, e.g. `JL1HIS` → `Japan	JA	AS	CQ25	ITU45`
- Portable notation like `W1AW/KH6` or `JA1XYZ/P` is resolved by the operating location
- `--waedc` keeps WAEDC-only entities such as Sicily (`IT9`) instead of folding them into DXCC entities
- Exits with an error after printing the others if any callsign is unknown
