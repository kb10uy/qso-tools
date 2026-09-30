use std::{convert::Infallible, path::PathBuf, str::FromStr};

use clap::Args;

use crate::core::adif::LenientMode;

/// Generates JSON data for QSL cards.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Processor script file.
    pub script_path: PathBuf,

    /// Read QSOs from ADIF file instead of ADI from stdin.
    #[arg(long, value_name = "FILE")]
    pub adif: Option<PathBuf>,

    /// Enable lenient length count for ADI file.
    /// Pedantic ADI file must not contain non-ASCII characters.
    #[arg(short, long = "lenient")]
    pub lenient_length: Option<LenientMode>,

    /// Specify additional instruments definition file.
    /// instruments.toml next to config file is always loaded first if exists.
    #[arg(short, long = "instruments")]
    pub instruments_files: Vec<PathBuf>,

    /// Specify arguments passed to script as KEY or KEY=VALUE.
    #[arg(short = 'A', long = "args", value_name = "KEY[=VALUE]")]
    pub script_args: Vec<ScriptArg>,

    /// Specify default instrument.
    #[arg(short = 'I', long)]
    pub instrument: Option<String>,

    /// Specify default power.
    /// It overrides default instrument's value.
    #[arg(short = 'P', long)]
    pub power: Option<f64>,

    /// Sort QSOs in the order JARL QSL bureau asks members to bundle cards.
    /// cty.dat next to config file is used to group foreign cards by DXCC entity.
    #[arg(short = 'B', long)]
    pub bureau_order: bool,
}

/// Script argument given as `KEY` or `KEY=VALUE`; a bare `KEY` has an empty value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScriptArg {
    pub key: String,
    pub value: String,
}

impl FromStr for ScriptArg {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (key, value) = s.split_once('=').unwrap_or((s, ""));
        Ok(ScriptArg {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_script_arg() {
        let arg: ScriptArg = "key=value".parse().unwrap();
        assert_eq!(arg.key, "key");
        assert_eq!(arg.value, "value");

        let arg: ScriptArg = "a=b=c".parse().unwrap();
        assert_eq!(arg.key, "a");
        assert_eq!(arg.value, "b=c");

        let arg: ScriptArg = "flag".parse().unwrap();
        assert_eq!(arg.key, "flag");
        assert_eq!(arg.value, "");
    }
}
