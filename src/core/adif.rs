use std::{
    fs::read_to_string,
    io::{IsTerminal, read_to_string as read_all, stdin},
    path::{Path, PathBuf},
    str::FromStr,
};

use adif_reader::{LengthMode, document::AdifDocument, read_adi, read_adx};
use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use time::{
    UtcOffset, error::Parse as TimeParseError, format_description::BorrowedFormatItem,
    macros::format_description,
};

const CUSTOM_OFFSET: &[BorrowedFormatItem<'_>] =
    format_description!("[offset_hour]:[offset_minute]");

/// ADIF input options shared by tools.
#[derive(Debug, Clone, Args)]
pub struct AdifInput {
    /// Read QSOs from ADIF file instead of ADI from stdin.
    #[arg(long, value_name = "FILE")]
    pub adif: Option<PathBuf>,

    /// Enable lenient length count for ADI file.
    /// Pedantic ADI file must not contain non-ASCII characters.
    #[arg(short, long = "lenient")]
    pub lenient_length: Option<LenientMode>,

    /// Specify UTC offset of QSO_DATE and TIME_ON as UTC, JST or [+-]hh:mm.
    /// Pedantic ADIF has them in UTC.
    #[arg(long, value_name = "OFFSET")]
    pub import_offset: Option<ImportOffset>,
}

impl AdifInput {
    /// Reads ADIF document from `--adif` file or stdin.
    pub fn read(&self) -> Result<AdifDocument> {
        read_document(
            self.adif.as_deref(),
            self.lenient_length.unwrap_or_default().into(),
        )
    }

    /// Returns UTC offset of datetimes in ADIF records.
    pub fn import_offset(&self) -> UtcOffset {
        self.import_offset.unwrap_or_default().into()
    }
}

/// Reads ADIF document from the file (ADX if `.adx`, otherwise ADI) or ADI from stdin.
fn read_document(path: Option<&Path>, length_mode: LengthMode) -> Result<AdifDocument> {
    let Some(path) = path else {
        let stdin = stdin();
        if stdin.is_terminal() {
            bail!("specify --adif or pipe ADI into stdin");
        }
        let text = read_all(stdin.lock()).context("failed to read stdin")?;
        return read_adi(&text, length_mode).context("failed to parse ADI from stdin");
    };

    let text =
        read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let is_adx = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("adx"));
    if is_adx {
        read_adx(&text)
    } else {
        read_adi(&text, length_mode)
    }
    .with_context(|| format!("failed to parse {}", path.display()))
}

/// Length count mode of ADI data for lenient files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum LenientMode {
    /// Count by bytes.
    #[default]
    Bytes,

    /// Count by codepoints.
    Codepoints,

    /// Count by grapheme clusters.
    Graphemes,
}

impl From<LenientMode> for LengthMode {
    fn from(value: LenientMode) -> Self {
        match value {
            LenientMode::Bytes => LengthMode::Bytes,
            LenientMode::Codepoints => LengthMode::Codepoints,
            LenientMode::Graphemes => LengthMode::Graphemes,
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
}
