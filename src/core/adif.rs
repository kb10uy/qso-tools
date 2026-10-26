use std::{
    fs::read,
    io::{IsTerminal, Read, stdin},
    path::{Path, PathBuf},
    slice::from_ref,
    str::FromStr,
};

use adif_manip::{
    LengthMode, document::AdifDocument, encoding_rs::SHIFT_JIS, read_adi_detecting, read_adx,
};
use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum, builder::PossibleValue};
use time::{
    UtcOffset, error::Parse as TimeParseError, format_description::BorrowedFormatItem,
    macros::format_description,
};
use tracing::info;

const CUSTOM_OFFSET: &[BorrowedFormatItem<'_>] =
    format_description!("[offset_hour]:[offset_minute]");

const PEDANTIC: &[LenientMode] = &[LenientMode::Count(LengthCount::Bytes)];

const AUTO_UTF8: &[LengthCount] = &[
    LengthCount::Bytes,
    LengthCount::Codepoints,
    LengthCount::Utf16,
    LengthCount::ShiftJis,
    LengthCount::Graphemes,
];

const AUTO_SHIFT_JIS: &[LengthCount] = &[
    LengthCount::ShiftJis,
    LengthCount::Codepoints,
    LengthCount::Utf16,
    LengthCount::Graphemes,
];

/// ADIF input options shared by tools.
#[derive(Debug, Clone, Args)]
pub struct AdifInput {
    /// Read QSOs from ADIF file instead of ADI from stdin.
    #[arg(long, value_name = "FILE")]
    pub adif: Option<PathBuf>,

    /// Enable lenient length count for ADI file.
    /// Comma-separated modes are detected for each value.
    /// ADI not in UTF-8 is read as Shift_JIS with shift-jis or auto.
    /// Pedantic ADI file must not contain non-ASCII characters.
    #[arg(short, long, value_name = "MODE", value_delimiter = ',')]
    pub lenient: Vec<LenientMode>,

    /// Specify UTC offset of QSO_DATE and TIME_ON as UTC, JST or [+-]hh:mm.
    /// Pedantic ADIF has them in UTC.
    #[arg(long, value_name = "OFFSET")]
    pub import_offset: Option<ImportOffset>,
}

impl AdifInput {
    /// Reads ADIF document from `--adif` file or stdin.
    pub fn read(&self) -> Result<AdifDocument> {
        let lenient = if self.lenient.is_empty() {
            PEDANTIC
        } else {
            &self.lenient
        };
        read_document(self.adif.as_deref(), lenient)
    }

    /// Returns UTC offset of datetimes in ADIF records.
    pub fn import_offset(&self) -> UtcOffset {
        self.import_offset.unwrap_or_default().into()
    }
}

/// Reads ADIF document from the file (ADX if `.adx`, otherwise ADI) or ADI from stdin.
fn read_document(path: Option<&Path>, lenient: &[LenientMode]) -> Result<AdifDocument> {
    let Some(path) = path else {
        let stdin = stdin();
        if stdin.is_terminal() {
            bail!("specify --adif or pipe ADI into stdin");
        }
        let mut bytes = Vec::new();
        stdin
            .lock()
            .read_to_end(&mut bytes)
            .context("failed to read stdin")?;
        return read_adi(bytes, lenient).context("failed to parse ADI from stdin");
    };

    let bytes = read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let is_adx = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("adx"));
    if is_adx {
        String::from_utf8(bytes)
            .map_err(Into::into)
            .and_then(|text| Ok(read_adx(&text)?))
    } else {
        read_adi(bytes, lenient)
    }
    .with_context(|| format!("failed to parse {}", path.display()))
}

/// Reads ADI with lengths counted in any of `lenient` modes.
fn read_adi(bytes: Vec<u8>, lenient: &[LenientMode]) -> Result<AdifDocument> {
    let (text, shift_jis) = decode_adi(bytes, lenient)?;
    let modes = length_modes(lenient, shift_jis);
    Ok(read_adi_detecting(&text, &modes)?)
}

/// Decodes ADI as UTF-8, or as Shift_JIS if it is not UTF-8 and `lenient` accepts Shift_JIS.
fn decode_adi(bytes: Vec<u8>, lenient: &[LenientMode]) -> Result<(String, bool)> {
    let error = match String::from_utf8(bytes) {
        Ok(text) => return Ok((text, false)),
        Err(error) => error,
    };
    let accepts_shift_jis = lenient.iter().any(|m| {
        matches!(
            m,
            LenientMode::Auto | LenientMode::Count(LengthCount::ShiftJis)
        )
    });
    if !accepts_shift_jis {
        return Err(error.utf8_error()).context("ADI is not in UTF-8; use -l shift-jis or auto");
    }

    let text = SHIFT_JIS
        .decode_without_bom_handling_and_without_replacement(error.as_bytes())
        .context("ADI is in neither UTF-8 nor Shift_JIS")?;
    info!("reading ADI in Shift_JIS");
    Ok((text.into_owned(), true))
}

/// Expands `lenient` modes into distinct length modes in order.
fn length_modes(lenient: &[LenientMode], shift_jis: bool) -> Vec<LengthMode> {
    let auto = if shift_jis { AUTO_SHIFT_JIS } else { AUTO_UTF8 };
    let mut modes = Vec::new();
    for &count in lenient.iter().flat_map(|m| match m {
        LenientMode::Auto => auto,
        LenientMode::Count(count) => from_ref(count),
    }) {
        let mode = count.into();
        if !modes.contains(&mode) {
            modes.push(mode);
        }
    }
    modes
}

/// Unit of ADI data lengths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LengthCount {
    /// Count by UTF-8 bytes.
    Bytes,

    /// Count by codepoints.
    Codepoints,

    /// Count by grapheme clusters.
    Graphemes,

    /// Count by UTF-16 code units.
    Utf16,

    /// Count by Shift_JIS bytes.
    ShiftJis,
}

impl From<LengthCount> for LengthMode {
    fn from(value: LengthCount) -> Self {
        match value {
            LengthCount::Bytes => LengthMode::Bytes,
            LengthCount::Codepoints => LengthMode::Codepoints,
            LengthCount::Graphemes => LengthMode::Graphemes,
            LengthCount::Utf16 => LengthMode::Utf16Units,
            LengthCount::ShiftJis => LengthMode::EncodedBytes(SHIFT_JIS),
        }
    }
}

/// Length count mode of ADI data for lenient files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LenientMode {
    /// Detects the unit for each value among all units.
    Auto,

    /// Counts in the unit.
    Count(LengthCount),
}

impl ValueEnum for LenientMode {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            LenientMode::Count(LengthCount::Bytes),
            LenientMode::Count(LengthCount::Codepoints),
            LenientMode::Count(LengthCount::Graphemes),
            LenientMode::Count(LengthCount::Utf16),
            LenientMode::Count(LengthCount::ShiftJis),
            LenientMode::Auto,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            LenientMode::Auto => Some(PossibleValue::new("auto").help("Detect for each value")),
            LenientMode::Count(count) => count.to_possible_value(),
        }
    }
}

/// UTC offset of datetimes in ADIF records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportOffset {
    #[default]
    Utc,
    Jst,
    Custom(UtcOffset),
}

impl FromStr for ImportOffset {
    type Err = TimeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UTC" => Ok(ImportOffset::Utc),
            "JST" => Ok(ImportOffset::Jst),
            _ => Ok(ImportOffset::Custom(UtcOffset::parse(s, CUSTOM_OFFSET)?)),
        }
    }
}

impl From<ImportOffset> for UtcOffset {
    fn from(value: ImportOffset) -> Self {
        match value {
            ImportOffset::Utc => UtcOffset::UTC,
            ImportOffset::Jst => UtcOffset::from_hms(9, 0, 0).expect("valid offset"),
            ImportOffset::Custom(offset) => offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn parses_import_offset() {
        let offset = |s: &str| UtcOffset::from(s.parse::<ImportOffset>().unwrap());
        assert_eq!(offset("UTC"), UtcOffset::UTC);
        assert_eq!(offset("JST"), UtcOffset::from_hms(9, 0, 0).unwrap());
        assert_eq!(offset("+09:00"), UtcOffset::from_hms(9, 0, 0).unwrap());
        assert_eq!(offset("-05:30"), UtcOffset::from_hms(-5, -30, 0).unwrap());
        assert!("jst".parse::<ImportOffset>().is_err());
    }

    #[test]
    fn parses_lenient_modes() {
        #[derive(Debug, Parser)]
        struct Parsed {
            #[command(flatten)]
            input: AdifInput,
        }

        let parse = |args: &[&str]| {
            Parsed::try_parse_from(["test"].iter().chain(args)).map(|p| p.input.lenient)
        };
        assert_eq!(parse(&[]).unwrap(), []);
        assert_eq!(
            parse(&["-l", "codepoints,shift-jis"]).unwrap(),
            [
                LenientMode::Count(LengthCount::Codepoints),
                LenientMode::Count(LengthCount::ShiftJis),
            ]
        );
        assert_eq!(parse(&["-l", "auto"]).unwrap(), [LenientMode::Auto]);
        assert!(parse(&["-l", "utf8"]).is_err());
    }

    #[test]
    fn expands_lenient_modes() {
        let lenient = [
            LenientMode::Count(LengthCount::Codepoints),
            LenientMode::Auto,
        ];
        assert_eq!(
            length_modes(&lenient, false),
            [
                LengthMode::Codepoints,
                LengthMode::Bytes,
                LengthMode::Utf16Units,
                LengthMode::EncodedBytes(SHIFT_JIS),
                LengthMode::Graphemes,
            ]
        );
        assert_eq!(
            length_modes(&[LenientMode::Auto], true)[0],
            LengthMode::EncodedBytes(SHIFT_JIS)
        );
    }

    #[test]
    fn detects_lengths_in_auto_mode() {
        let text = "<NAME:6>太郎 <CALL:6>JA1XYZ <EOR>\n<NAME:2>花子 <EOR>\n<NAME:4>次郎 <EOR>\n";
        let document = read_adi(text.as_bytes().to_vec(), &[LenientMode::Auto]).unwrap();
        let names: Vec<_> = document.records().iter().map(|r| r.field("NAME")).collect();
        assert_eq!(names, [Some("太郎"), Some("花子"), Some("次郎")]);
    }

    #[test]
    fn reads_shift_jis_only_if_accepted() {
        let bytes = SHIFT_JIS.encode("<NAME:4>太郎 <EOR>\n").0.into_owned();
        assert!(read_adi(bytes.clone(), PEDANTIC).is_err());

        for lenient in [LenientMode::Auto, LenientMode::Count(LengthCount::ShiftJis)] {
            let document = read_adi(bytes.clone(), &[lenient]).unwrap();
            assert_eq!(document.records()[0].field("NAME"), Some("太郎"));
        }
    }
}
