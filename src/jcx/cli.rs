use clap::Args;

use crate::jcx::JcxCode;

/// Shows full names of JCC/JCG codes.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// JCC/JCG codes, optionally with HAMLOG town suffix like 15006C.
    #[arg(required = true, value_name = "CODE")]
    pub codes: Vec<JcxCode>,
}
