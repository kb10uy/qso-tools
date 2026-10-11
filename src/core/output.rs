use std::{
    fs::File,
    io::{BufWriter, Write, stdout},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::Args;

/// Output options shared by tools.
#[derive(Debug, Clone, Args)]
pub struct OutputFile {
    /// Write output into file instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

impl OutputFile {
    /// Opens `--output` file or stdout.
    pub fn writer(&self) -> Result<Box<dyn Write>> {
        match &self.output {
            Some(path) => {
                let file = File::create(path)
                    .with_context(|| format!("failed to create {}", path.display()))?;
                Ok(Box::new(BufWriter::new(file)))
            }
            None => Ok(Box::new(stdout().lock())),
        }
    }
}
