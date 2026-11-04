use std::path::PathBuf;

use clap::Args;

use crate::core::{adif::AdifInput, output::OutputFile, schope::args::ScriptArg};

/// Generates JSON data for QSL cards.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Processor script file.
    pub script_path: PathBuf,

    #[command(flatten)]
    pub input: AdifInput,

    /// Specify arguments passed to script as KEY or KEY=VALUE.
    #[arg(short = 'A', long = "args", value_name = "KEY[=VALUE]")]
    pub script_args: Vec<ScriptArg>,

    /// Sort QSOs in the order JARL QSL bureau asks members to bundle cards.
    /// cty.dat next to config file is used to group foreign cards by DXCC entity.
    #[arg(short = 'B', long)]
    pub bureau_order: bool,

    #[command(flatten)]
    pub output: OutputFile,
}
