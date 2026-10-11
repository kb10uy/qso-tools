use std::path::PathBuf;

use clap::Args;

use crate::core::{adif::LenientMode, schope::args::ScriptArg};

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
