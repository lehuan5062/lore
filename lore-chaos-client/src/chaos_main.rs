use clap::Parser;
use lore::interface::LoreString;

use crate::chaos::config::RunnerConfig;
use crate::chaos::runner::ChaosRunner;
use crate::cli::CliArgs;
use crate::cli::CliCommand;
use crate::lore::interface::LoreInterface;
use crate::parallel::config::ParallelConfig;
use crate::parallel::runner::ParallelRunner;
use crate::probability::ProbabilityEngine;
use crate::probability::weighting::ProbabilityWeighting;
use crate::tracing::setup_tracing;

pub fn chaos_main() {
    // Parse command line arguments
    let args = CliArgs::parse();

    setup_tracing(&args);

    match &args.command {
        CliCommand::Chaos(chaos_args) => {
            let mut engine = ChaosRunner::new(
                RunnerConfig::from_cli(&args, chaos_args),
                ProbabilityEngine::new(ProbabilityWeighting::default(), chaos_args.seed),
                LoreInterface::new(LoreString::from(&args.repository_path), args.offline),
            );
            engine.run();
        }
        CliCommand::Parallel(parallel_args) => {
            let mut engine = ParallelRunner::new(
                ParallelConfig::from_cli(&args, parallel_args),
                parallel_args.seed,
                LoreInterface::new(LoreString::from(&args.repository_path), args.offline),
            );
            engine.run();
        }
    }
}
