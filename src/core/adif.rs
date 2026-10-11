use std::{
    fs::read_to_string,
    io::{IsTerminal, read_to_string as read_all, stdin},
    path::{Path, PathBuf},
};

use adif_reader::{LengthMode, document::AdifDocument, read_adi, read_adx};
use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};

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
}

impl AdifInput {
    /// Reads ADIF document from `--adif` file or stdin.
    pub fn read(&self) -> Result<AdifDocument> {
        read_document(
            self.adif.as_deref(),
            self.lenient_length.unwrap_or_default().into(),
        )
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
