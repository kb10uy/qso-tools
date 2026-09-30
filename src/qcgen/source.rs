use std::{
    fs::read_to_string,
    io::{IsTerminal, read_to_string as read_all, stdin},
    path::Path,
};

use adif_reader::{LengthMode, document::AdifDocument, read_adi, read_adx};
use anyhow::{Context, Result, bail};

/// Reads ADIF document from the file (ADX if `.adx`, otherwise ADI) or ADI from stdin.
pub fn read_document(path: Option<&Path>, length_mode: LengthMode) -> Result<AdifDocument> {
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
