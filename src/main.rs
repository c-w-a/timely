// written by cwa on 17/aug/2025

use anyhow::Result;
use chrono::{DateTime, Utc};

mod analysis;
mod utc;

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let utc_now: DateTime<Utc> = rt.block_on(utc::fetch_current_utc_datetime(
        111, // per-host timeout ms
        19,  // cutoff in ms
        3,   // minimum keep
    ))?;
    println!("current utc datetime:     {utc_now}");

    Ok(())
}
