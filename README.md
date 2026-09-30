# QSO Tools

kb10uy's small toolbox to manage QSOs and QSL cards.
All tools work on ADIF only; use `wavelog-tools export` to get ADIF from Wavelog.

## Configuration

Settings are read from `qso-tools/config.toml` in the user config directory
(`$XDG_CONFIG_HOME` or `~/.config` on Linux and macOS, `%APPDATA%` on Windows).
Use `-c/--config` to specify another file. See `assets/config.example.toml`.

- `[operators.<CALLSIGN>]`: display name of operator

`instruments.toml`, `parks.toml`, `jcc-jcg.toml` and `subdivisions.toml` placed next to `config.toml` are also loaded by `qcgen` (see `assets/qcgen/*.example.toml`).

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
- `MY_CNTY` is passed as `station.location.county`; JCC/JCG codes (with optional HAMLOG town suffix like `15006C`) are resolved from `jcc-jcg.toml`
- Operator comes from `OPERATOR`; its display name is looked up from `[operators]` in config
- `QSL_VIA` and `QSL_SENT_VIA` are passed as `qsl.via` and `qsl.sent_via`
- `MY_POTA_REF` is split into `station.references.pota` (reference, location and names from `parks.toml`)
- `!inst:<key>` in `COMMENT` (or `-I <key>`) selects instrument from `instruments.toml` and `-i` files; power is taken from `TX_PWR`, `--power`, then `default_power` of instrument

See `assets/qcgen/` for example files and `assets/schope-types/` for Lua type definitions.
