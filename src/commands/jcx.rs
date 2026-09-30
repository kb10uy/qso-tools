use anyhow::{Result, bail};
use clap::Args;

use crate::core::{
    config::Config,
    jcx::{COUNTIES_FILENAME, Jcx, JcxCode},
};

/// Shows full names of JCC/JCG codes.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// JCC/JCG codes, optionally with HAMLOG town suffix like 15006C.
    #[arg(required = true, value_name = "CODE")]
    pub codes: Vec<JcxCode>,
}

pub fn run(args: Arguments, config: &Config) -> Result<()> {
    let jcx = Jcx::load(config)?;
    if jcx.is_empty() {
        bail!("{COUNTIES_FILENAME} not found next to config file");
    }

    let mut unknown_codes = vec![];
    for code in &args.codes {
        match jcx.full_name_ja(code) {
            Some(name) => println!("{code}	{name}"),
            None => unknown_codes.push(code.to_string()),
        }
    }
    if !unknown_codes.is_empty() {
        bail!("unknown codes: {}", unknown_codes.join(", "));
    }

    Ok(())
}
