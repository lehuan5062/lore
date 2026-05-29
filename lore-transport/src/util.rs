// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::time::Duration;

pub struct Retry {
    current: u64,
    maximum: u64,
    jitter: f32,
    counter: usize,
    limit: usize,
}

impl Retry {
    pub async fn wait(&mut self) -> bool {
        if self.counter >= self.limit {
            return false;
        }

        // Generate some jitter to avoid alignment storms
        let jitter = rand::random::<f32>() * self.jitter;
        let jitter = std::cmp::min((jitter * self.current as f32) as u64, 100);

        tokio::time::sleep(Duration::from_millis(self.current + jitter)).await;

        self.current = std::cmp::min(self.current * 2, self.maximum);
        self.counter += 1;

        true
    }

    pub fn counter(&self) -> usize {
        self.counter
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

const DEFAULT_JITTER: f32 = 0.1;

/// Create a retry waiter, start and maximum times in milliseconds. Will give up
/// after trying for the limit number of times.
pub fn retry(start: u64, maximum: u64, limit: usize) -> Retry {
    Retry {
        current: start,
        maximum,
        jitter: DEFAULT_JITTER,
        counter: 0,
        limit,
    }
}
