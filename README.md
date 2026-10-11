# QSO Tools

kb10uy's small toolbox to manage QSOs and QSL cards.
All tools work on ADIF only; use `wavelog-tools export` to get ADIF from Wavelog.

## Configuration

Settings are read from `qso-tools/config.toml` in the user config directory
(`$XDG_CONFIG_HOME` or `~/.config` on Linux and macOS, `%APPDATA%` on Windows).
Use `-c/--config` to specify another file. See `assets/config.example.toml`.

- `[operators.<CALLSIGN>]`: display name of operator
- `[operators.<CALLSIGN>.contest]`: `name` (defaults to display name), `address`, `tel`, `email`, `birthday` and `licensed_at` of operator for `loggen` summary sheet

`instruments.toml`, `parks.toml`, `japan-jcx.toml`, `japan-jcx-town.toml` and `subdivisions.toml` placed next to `config.toml` are also loaded by `qcgen` and `jcx` (see `assets/qcgen/*.example.toml`).
`cty.dat` placed next to `config.toml` is loaded by `callsign` and `qcgen -B`; get it from [Country Files](https://www.country-files.com/).

## Input and Output

`qcgen`, `loggen` and `json` share these options.

- `--adif <FILE>`: reads QSOs from ADIF file (ADX if `.adx`); ADI is read from stdin without it
- `-l/--lenient <MODE>`: counts ADI data lengths in `bytes` (default), `codepoints` or `graphemes`
- `--import-offset <OFFSET>`: reads `QSO_DATE` and `TIME_ON` in `UTC` (default), `JST` or `[+-]hh:mm`, for ADIF not written in UTC; `json` outputs them as is
- `-o/--output <FILE>`: writes into file; output goes to stdout without it

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
- `MY_CNTY` is passed as `station.location.county` as is; if `MY_DXCC` is Japan (339) and it is a JCC/JCG code like `100101` (with optional HAMLOG town suffix) found in `japan-jcx.toml`, its Japanese names (and town name from `japan-jcx-town.toml`) are passed as `station.location.japan_jcx`, with the containing prefecture and city (for wards of designated cities) as `prefecture` and `city`
- Operator comes from `OPERATOR`; its display name is looked up from `[operators]` in config
- `QSL_VIA` and `QSL_SENT_VIA` are passed as `qsl.via` and `qsl.sent_via`
- `MY_POTA_REF` is split into `station.references.pota` (reference, location and names from `parks.toml`)
- `!inst:<key>` in `COMMENT` (or `-I <key>`) selects instrument from `instruments.toml` and `-i` files; power is taken from `TX_PWR`, `--power`, then `default_power` of instrument
- `-B/--bureau-order` sorts QSOs in the order [JARL QSL bureau asks members to bundle cards](https://www.jarl.org/Japanese/5_Nyukai/qsl-sq.htm) before passing them to the script
    - Cards are routed to `QSL_VIA` if it looks like a callsign, otherwise to the home callsign of `CALL` (the longest segment of portable notation like `3D2/JA1XYZ`)
    - Domestic cards come first: JA1-JS1, JA2-JS2, ..., JA0-JR0, then 7J1-7J0, 7K1-7N4, 8J1-8N0, and SWL numbers like `JA1-12345` last
    - Foreign cards follow, grouped by primary prefix of DXCC entity from `cty.dat` (or by callsign prefix without it)

See `assets/qcgen/` for example files and `assets/schope-types/` for Lua type definitions.

## loggen

Generates contest logs through a Lua script that defines the contest rules.

```sh
# JARL eLog (summary sheet R2.1 and log sheet in JARL format)
qso-tools loggen jarl assets/loggen/acag.lua --adif qsos.adi -A category=XAM --qso-since 2026-10-10T21:00 --qso-until 2026-10-11T21:00 -o elog.txt

# Cabrillo 3.0
qso-tools loggen cabrillo assets/loggen/acag.lua --adif qsos.adi -A category=XAM -A name="Yu Kobayashi" -o acag.log
```

- `jarl` writes [JARL eLog](https://www.jarl.org/Japanese/1_Tanoshimo/1-1_Contest/logformat.html) with times in JST (or `offset` of the script), band in MHz (`10G` and above in GHz) and `DIGITALVOICE` as `DV`
- `cabrillo` writes Cabrillo 3.0 with times in UTC, frequency in kHz and mode as `CW`, `PH`, `FM`, `RY` or `DG`
- QSOs are processed in time order; `--qso-since` (inclusive) and `--qso-until` (exclusive) limit the contest period, otherwise all QSOs in the input are used
    - Datetimes are given like `2026-10-10T21:00`, in the offset of the script (JST by default), or with offset like `2026-10-10T12:00Z`
- The script returns a contest definition (`ContestDefinition`):
    - `categories`: categories keyed by code, with accepted `modes` (mode groups), `bands` and `exclude_bands`, summary items it `requires`, and `cabrillo` values for `CATEGORY-*`
    - `dupe_scope` and `multiplier_scope`: attributes (`band`, `band_group`, `mode`, `mode_group`) separating duplicate checks of callsigns and multipliers; both default to `{ "band" }`, and `mode_groups` defaults to `CW` and `PH`
    - `score_qso(entry, ctx)`: returns point and multiplier (or `multipliers` keyed by kind) of the QSO, or `nil` to exclude it; a second return value is shown as the reason
    - `dupe_key`, `multiplier_key` and `total` (optional): functions for rules the scopes cannot express and for total score (default: points times multipliers)
- `require("jarl")` provides contest number parsing, JCC/JCG lookup from `japan-jcx.toml` and call areas, and `require("cty")` provides DXCC entities from `cty.dat`
- Duplicate QSOs are written with 0 points; multipliers are shown only at their first appearance
- Category is given by `-A category=CODE` or chosen interactively
- Summary items are given by `-A KEY=VALUE` (`callsign`, `op_callsign`, `operator`, `name`, `address`, `tel`, `email`, `license_date`, `age`, `power`, `fd_coeff`, `op_place`, `power_supply`, `comments`, `multi_op_list`, `reg_club_number`, `date`); otherwise they are taken from QSOs and config, and missing required ones are asked interactively
    - Callsign comes from `STATION_CALLSIGN`, power from maximum `TX_PWR` and operating place from `MY_CNTY` of QSOs
    - Name, address, TEL, email, license date and age (from `birthday` at the start of the contest) come from `[operators.<CALLSIGN>]` and its `contest` table in config; the operator is `-A operator=`, `OPERATOR` of the first QSO, then the callsign
    - Callsign, name, address and power are always required
- Output is UTF-8

See `assets/loggen/` for example scripts (category codes and Cabrillo contest IDs should be checked against the latest rules) and `assets/schope-types/` for Lua type definitions.

## jcx

Shows full Japanese names of JCC/JCG codes like `100101` (with optional HAMLOG town suffix) from `japan-jcx.toml` and `japan-jcx-town.toml`.

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

## json

Converts ADIF into JSON.

```sh
wavelog-tools export --qso-since 2026-01-01 | qso-tools json -l codepoints -p
```

- QSOs are read from `--adif` file (ADX if `.adx`) or ADI from stdin, with the same `-l` option as `qcgen`
- Output is an object with `preamble` (string), `header` (object of header fields) and `records` (array of objects of record fields)
- Field names are uppercased and all values are strings as written in ADIF
- `-p/--pretty` pretty-prints JSON
