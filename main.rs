use anyhow::Result;
use clap::Parser;
use log::info;
use sc_sysinfo::gather_hwbench;
use std::fs;
use std::path::Path;
use substrate_benchmark_machine::{check_hardware, MachineCmd};

fn main() -> Result<()> {
    lg::info::init()?;

    let cmd = MachineCmd::parse();
    cmd.validate_args()?;

    let base_path = cmd.base_path.clone().unwrap_or(".".into());
    fs::create_dir_all(&base_path)?;
    let dir = Path::new(&base_path);

    if !cmd.full {
        let hwbench = gather_hwbench(Some(dir));
        if !check_hardware(&hwbench) {
            info!("⚠  The hardware does not meet the minimal requirements for role 'Authority'.");
        } else {
            info!("🎉 The hardware meets the minimal requirements for role 'Authority'.");
        }
    } else {
        cmd.print_full_table(&dir)?;
    }

    Ok(())
}
