// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Contains the [`MachineCmd`] as entry point for the node
//! and the core benchmarking logic.

pub mod hardware;

use std::{boxed::Box, path::Path};

use clap::Parser;
use comfy_table::{Row, Table};
use log::{error, info, warn};

use sc_cli::Result;
use sc_sysinfo::{
    benchmark_cpu, benchmark_cpu_parallelism, benchmark_disk_random_writes,
    benchmark_disk_sequential_writes, benchmark_memory, benchmark_sr25519_verify, ExecutionLimit,
    HwBench, Metric, Requirement, Requirements, Throughput,
};

// use crate::shared::check_build_profile;
pub use hardware::SUBSTRATE_REFERENCE_HARDWARE;

/// Command to benchmark the hardware.
///
/// Runs multiple benchmarks and prints their output to console.
/// Can be used to gauge if the hardware is fast enough to keep up with a chain's requirements.
/// This command must be integrated by the client since the client can set compiler flags
/// which influence the results.
///
/// You can use the `--base-path` flag to set a location for the disk benchmarks.
#[derive(Debug, Parser)]
pub struct MachineCmd {
    /// Path to database.
    #[arg(long, short = 'd')]
    pub base_path: Option<String>,

    /// Run full benchmarks instead of quick hardware check.
    #[arg(long, short = 'f')]
    pub full: bool,

    /// Do not return an error if any check fails.
    ///
    /// Should only be used for debugging.
    #[arg(long)]
    pub allow_fail: bool,

    /// Set a fault tolerance for passing a requirement.
    ///
    /// 10% means that the test would pass even when only 90% score was archived.
    /// Can be used to mitigate outliers of the benchmarks.
    #[arg(long, default_value_t = 10.0, value_name = "PERCENT")]
    pub tolerance: f64,

    /// Time limit for the verification benchmark.
    #[arg(long, default_value_t = 5.0, value_name = "SECONDS")]
    pub verify_duration: f32,

    /// Time limit for the hash function benchmark.
    #[arg(long, default_value_t = 5.0, value_name = "SECONDS")]
    pub hash_duration: f32,

    /// Time limit for the memory benchmark.
    #[arg(long, default_value_t = 5.0, value_name = "SECONDS")]
    pub memory_duration: f32,

    /// Time limit for each disk benchmark.
    #[arg(long, default_value_t = 5.0, value_name = "SECONDS")]
    pub disk_duration: f32,
}

/// Helper for the result of a concrete benchmark.
#[derive(Debug)]
pub struct BenchResult {
    /// Did the hardware pass the benchmark?
    passed: bool,

    /// The absolute score that was archived.
    score: Throughput,

    /// The score relative to the minimal required score.
    ///
    /// Is in range [0, 1].
    rel_score: f64,
}

/// Errors that can be returned by the this command.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("One of the benchmarks had a score that was lower than its requirement")]
    UnmetRequirement,

    // #[error("The build profile is unfit for benchmarking: {0}")]
    // BadBuildProfile(String),
    #[error("Benchmark results are off by at least factor 100")]
    BadResults,
}

impl MachineCmd {
    /// Benchmarks a specific metric of the hardware and judges the resulting score.
    pub fn run_benchmark(&self, requirement: &Requirement, dir: &Path) -> Result<BenchResult> {
        // Dispatch the concrete function from `sc-sysinfo`.

        let score = self.measure(&requirement.metric, dir)?;
        let rel_score = score.as_bytes() / requirement.minimum.as_bytes();

        // Sanity check if the result is off by factor >100x.
        if rel_score >= 100.0 || rel_score <= 0.01 {
            self.check_failed(Error::BadResults)?;
        }
        let passed = rel_score >= (1.0 - (self.tolerance / 100.0));
        Ok(BenchResult {
            passed,
            score,
            rel_score,
        })
    }

    /// Measures a metric of the hardware.
    fn measure(&self, metric: &Metric, dir: &Path) -> Result<Throughput> {
        let verify_limit = ExecutionLimit::from_secs_f32(self.verify_duration);
        let disk_limit = ExecutionLimit::from_secs_f32(self.disk_duration);
        let hash_limit = ExecutionLimit::from_secs_f32(self.hash_duration);
        let memory_limit = ExecutionLimit::from_secs_f32(self.memory_duration);

        let score = match metric {
            Metric::Blake2256 => benchmark_cpu(hash_limit),
            Metric::Sr25519Verify => benchmark_sr25519_verify(verify_limit),
            Metric::Blake2256Parallel { num_cores } => {
                benchmark_cpu_parallelism(hash_limit, *num_cores)
            }
            Metric::MemCopy => benchmark_memory(memory_limit),
            Metric::DiskSeqWrite => benchmark_disk_sequential_writes(disk_limit, dir)?,
            Metric::DiskRndWrite => benchmark_disk_random_writes(disk_limit, dir)?,
        };
        Ok(score)
    }

    pub fn print_full_table(&self, dir: &Path) -> Result<()> {
        info!("Running full machine benchmarks...");
        let requirements = &SUBSTRATE_REFERENCE_HARDWARE.clone();
        let mut results = Vec::new();
        for requirement in &requirements.0 {
            let result = self.run_benchmark(requirement, &dir)?;
            results.push(result);
        }
        self.print_summary(requirements.clone(), results)?;
        Ok(())
    }

    /// Prints a human-readable summary.
    pub fn print_summary(
        &self,
        requirements: Requirements,
        results: Vec<BenchResult>,
    ) -> Result<()> {
        // Use a table for nicer console output.
        let mut table = Table::new();
        table.set_header(["Category", "Function", "Score", "Minimum", "Result"]);
        // Count how many passed and how many failed.
        let (mut passed, mut failed) = (0, 0);
        for (requirement, result) in requirements.0.iter().zip(results.iter()) {
            if result.passed {
                passed += 1
            } else {
                failed += 1
            }

            table.add_row(result.to_row(requirement));
        }
        // Print the table and a summary.
        info!(
            "\n{}\nFrom {} benchmarks in total, {} passed and {} failed ({:.0?}% fault tolerance).",
            table,
            passed + failed,
            passed,
            failed,
            self.tolerance
        );
        // Print the final result.
        if failed != 0 {
            info!("The hardware fails to meet the requirements");
            self.check_failed(Error::UnmetRequirement)?;
        } else {
            info!("The hardware meets the requirements ");
        }
        Ok(())
    }

    /// Returns `Ok` if [`self.allow_fail`] is set and otherwise the error argument.
    fn check_failed(&self, e: Error) -> Result<()> {
        if !self.allow_fail {
            error!("Failing since --allow-fail is not set");
            Err(sc_cli::Error::Application(Box::new(e)))
        } else {
            warn!("Ignoring error since --allow-fail is set: {:?}", e);
            Ok(())
        }
    }

    /// Validates the CLI arguments.
    pub fn validate_args(&self) -> Result<()> {
        if self.tolerance > 100.0 || self.tolerance < 0.0 {
            return Err("The --tolerance argument is out of range".into());
        }
        Ok(())
    }
}

impl BenchResult {
    /// Format [`Self`] as row that can be printed in a table.
    fn to_row(&self, req: &Requirement) -> Row {
        let passed = if self.passed { "✅ Pass" } else { "❌ Fail" };
        vec![
            req.metric.category().into(),
            req.metric.name().into(),
            format!("{}", self.score),
            format!("{}", req.minimum),
            format!("{} ({: >5.1?} %)", passed, self.rel_score * 100.0),
        ]
        .into()
    }
}

fn status_emoji(s: bool) -> String {
    if s {
        "✅".into()
    } else {
        "❌".into()
    }
}

/// Whether the hardware requirements are met by the provided benchmark results.
pub fn check_hardware(hwbench: &HwBench) -> bool {
    info!("Performing quick hardware check...");
    let req = &SUBSTRATE_REFERENCE_HARDWARE;

    let mut cpu_ok = true;
    let mut parallel_cpu_ok = true;
    let mut mem_ok = true;
    let mut dsk_seq_write_ok = true;
    let mut dsk_rnd_write_ok = true;

    for requirement in req.0.iter() {
        match requirement.metric {
            Metric::Blake2256 => {
                if requirement.minimum > hwbench.cpu_hashrate_score {
                    cpu_ok = false;
                }
                info!(
                    "🏁 CPU score: {} ({})",
                    hwbench.cpu_hashrate_score,
                    format!(
                        "{} Blake2256: expected minimum {}",
                        status_emoji(cpu_ok),
                        requirement.minimum
                    )
                );
            }
            Metric::Blake2256Parallel { .. } => {
                if requirement.minimum > hwbench.parallel_cpu_hashrate_score {
                    parallel_cpu_ok = false;
                }
                info!(
                    "🏁 Parallel CPU score: {} ({})",
                    hwbench.parallel_cpu_hashrate_score,
                    format!(
                        "{} Blake2256Parallel: expected minimum {}",
                        status_emoji(parallel_cpu_ok),
                        requirement.minimum
                    )
                );
            }
            Metric::MemCopy => {
                if requirement.minimum > hwbench.memory_memcpy_score {
                    mem_ok = false;
                }
                info!(
                    "🏁 Memory score: {} ({})",
                    hwbench.memory_memcpy_score,
                    format!(
                        "{} MemCopy: expected minimum {}",
                        status_emoji(mem_ok),
                        requirement.minimum
                    )
                );
            }
            Metric::DiskSeqWrite => {
                if let Some(score) = hwbench.disk_sequential_write_score {
                    if requirement.minimum > score {
                        dsk_seq_write_ok = false;
                    }
                    info!(
                        "🏁 Disk score (seq. writes): {} ({})",
                        score,
                        format!(
                            "{} DiskSeqWrite: expected minimum {}",
                            status_emoji(dsk_seq_write_ok),
                            requirement.minimum
                        )
                    );
                }
            }
            Metric::DiskRndWrite => {
                if let Some(score) = hwbench.disk_random_write_score {
                    if requirement.minimum > score {
                        dsk_rnd_write_ok = false;
                    }
                    info!(
                        "🏁 Disk score (rand. writes): {} ({})",
                        score,
                        format!(
                            "{} DiskRndWrite: expected minimum {}",
                            status_emoji(dsk_rnd_write_ok),
                            requirement.minimum
                        )
                    );
                }
            }
            Metric::Sr25519Verify => {}
        }
    }

    cpu_ok && mem_ok && dsk_seq_write_ok && dsk_rnd_write_ok
}
