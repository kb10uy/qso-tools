use anyhow::{Result, bail};
use callfind::cty::Scope;
use clap::Args;

use crate::core::{
    config::Config,
    cty::{CTY_FILENAME, load_cty},
};

/// Shows DXCC entities of callsigns.
#[derive(Debug, Clone, Args)]
pub struct Arguments {
    /// Callsigns, optionally with prefix or suffix like W1AW/KH6.
    #[arg(required = true, value_name = "CALLSIGN")]
    pub callsigns: Vec<String>,

    /// Resolve WAEDC-only entities such as Sicily instead of folding them into DXCC entities.
    #[arg(long)]
    pub waedc: bool,
}

pub fn run(args: Arguments, config: &Config) -> Result<()> {
    let Some(cty) = load_cty(config)? else {
        bail!("{CTY_FILENAME} not found next to config file");
    };
    let scope = if args.waedc {
        Scope::Waedc
    } else {
        Scope::Dxcc
    };

    let mut unknown_callsigns = vec![];
    for callsign in &args.callsigns {
        let callsign = callsign.trim().to_ascii_uppercase();
        let Some(found) = cty.lookup_in(&callsign, scope) else {
            unknown_callsigns.push(callsign);
            continue;
        };
        let location = found.location;
        println!(
            "{callsign}\t{}\t{}\t{}\tCQ{}\tITU{}",
            found.entity.name,
            found.entity.primary_prefix,
            location.continent,
            location.cq_zone,
            location.itu_zone,
        );
    }
    if !unknown_callsigns.is_empty() {
        bail!("unknown callsigns: {}", unknown_callsigns.join(", "));
    }

    Ok(())
}
