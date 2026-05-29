// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use tracing::error_span;
use tracing::info;

use crate::chaos::runner::ChaosRunner;
use crate::lore::interface::LoreInterface;
use crate::parallel::config::ParallelConfig;
use crate::probability::ProbabilityEngine;
use crate::probability::weighting::ProbabilityWeighting;

pub struct ParallelRunner {
    config: ParallelConfig,
    root_rng: StdRng,
    urc: LoreInterface,
}

impl ParallelRunner {
    pub fn new(config: ParallelConfig, seed: Option<u64>, urc: LoreInterface) -> Self {
        info!("ParallelRunner using seed {seed:?}");
        let seed = seed.unwrap_or_else(|| rand::rng().random());
        let root_rng = StdRng::seed_from_u64(seed);
        Self {
            config,
            root_rng,
            urc,
        }
    }

    pub fn run(&mut self) {
        let mut threads = Vec::new();
        for thread_index in 0..self.config.runners {
            let seed = self.root_rng.random();
            let config = self.config.chaos_config.clone();
            let urc = self.urc.clone();
            threads.push(std::thread::spawn(move || {
                let read_only = thread_index != 0;
                let _thread_span =
                    error_span!("parallel_thread", thread_index, read_only).entered();
                let mut runner = ChaosRunner::new(
                    config,
                    ProbabilityEngine::new(
                        if read_only {
                            ProbabilityWeighting::read_only()
                        } else {
                            ProbabilityWeighting::default()
                        },
                        Some(seed),
                    ),
                    urc,
                );
                runner.run();
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }
}
