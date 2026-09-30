use std::{collections::BTreeMap, io::stdout, path::PathBuf};

use adif_reader::document::{AdifDocument, Field};
use anyhow::Result;
use clap::Args;
use serde::Serialize;

use crate::core::{
    adif::{LenientMode, read_document},
    config::Config,
};

/// Converts ADIF into JSON.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Read QSOs from ADIF file instead of ADI from stdin.
    #[arg(long, value_name = "FILE")]
    pub adif: Option<PathBuf>,

    /// Enable lenient length count for ADI file.
    /// Pedantic ADI file must not contain non-ASCII characters.
    #[arg(short, long = "lenient")]
    pub lenient_length: Option<LenientMode>,

    /// Pretty-print JSON.
    #[arg(short, long)]
    pub pretty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct JsonDocument<'a> {
    preamble: &'a str,
    header: BTreeMap<&'a str, &'a str>,
    records: Vec<BTreeMap<&'a str, &'a str>>,
}

impl<'a> From<&'a AdifDocument> for JsonDocument<'a> {
    fn from(document: &'a AdifDocument) -> Self {
        JsonDocument {
            preamble: document.preamble(),
            header: fields_to_map(document.headers()),
            records: document
                .records()
                .iter()
                .map(|r| fields_to_map(r.fields()))
                .collect(),
        }
    }
}

pub fn run(args: Arguments, _config: &Config) -> Result<()> {
    let document = read_document(
        args.adif.as_deref(),
        args.lenient_length.unwrap_or_default().into(),
    )?;

    let json_document = JsonDocument::from(&document);
    let writer = stdout().lock();
    if args.pretty {
        serde_json::to_writer_pretty(writer, &json_document)?;
    } else {
        serde_json::to_writer(writer, &json_document)?;
    }
    println!();

    Ok(())
}

fn fields_to_map<'a>(
    fields: impl IntoIterator<Item = (&'a String, &'a Field)>,
) -> BTreeMap<&'a str, &'a str> {
    fields
        .into_iter()
        .map(|(k, v)| (k.as_str(), v.value()))
        .collect()
}

#[cfg(test)]
mod tests {
    use adif_reader::{LengthMode, read_adi};
    use serde_json::json;

    use super::*;

    #[test]
    fn converts_document() {
        let document = read_adi(
            "Exported\n<ADIF_VER:5>3.1.5<EOH>\n<CALL:6>JL1HIS<band:3>20m<EOR>\n<CALL:4>W1AW<EOR>\n",
            LengthMode::Bytes,
        )
        .unwrap();

        let converted = serde_json::to_value(JsonDocument::from(&document)).unwrap();
        assert_eq!(
            converted,
            json!({
                "preamble": document.preamble(),
                "header": { "ADIF_VER": "3.1.5" },
                "records": [
                    { "BAND": "20m", "CALL": "JL1HIS" },
                    { "CALL": "W1AW" },
                ],
            })
        );
    }
}
