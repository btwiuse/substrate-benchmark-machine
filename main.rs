use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::Path;
use substrate_benchmark_machine::{MachineCmd, SUBSTRATE_REFERENCE_HARDWARE};

fn main() -> Result<()> {
    lg::info::init()?;

    let cmd = MachineCmd::parse();
    cmd.validate_args()?;

    let base_path = cmd.base_path.clone().unwrap_or(".".into());
    fs::create_dir_all(&base_path)?;

    // info!("Running machine benchmarks...");
    let requirements = &SUBSTRATE_REFERENCE_HARDWARE.clone();
    let mut results = Vec::new();
    let dir = Path::new(&base_path);
    for requirement in &requirements.0 {
        let result = cmd.run_benchmark(requirement, &dir)?;
        results.push(result);
    }
    cmd.print_summary(requirements.clone(), results)?;

    Ok(())
}
