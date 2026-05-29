// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::time::Duration;

use serde::Deserialize;

pub fn timestamp() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}

/// Serializable settings. Do not rename without being careful of what toml files
/// might be referencing these values
#[derive(Clone, Debug, Deserialize)]
pub struct RetrySettings {
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    max_attempts: usize,
    jitter: Option<f32>,
}

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

#[derive(Copy, Clone, Debug)]
pub struct RetryPolicy {
    initial_backoff: Duration,
    max_backoff: Duration,
    limit: usize,
    jitter: f32,
}

impl RetryPolicy {
    pub fn builder() -> RetryPolicyBuilder<WantsInitialBackoff> {
        RetryPolicyBuilder(WantsInitialBackoff(()))
    }

    pub fn retry(self) -> Retry {
        retry_with_policy(self)
    }
}

const DEFAULT_JITTER: f32 = 0.1;

pub struct RetryPolicyBuilder<State>(State);

pub struct WantsInitialBackoff(());

pub struct WantsMaxBackoff {
    initial_backoff: Duration,
}

pub struct WantsLimit {
    initial_backoff: Duration,
    max_backoff: Duration,
}

pub struct MaybeWantsJitter {
    initial_backoff: Duration,
    max_backoff: Duration,
    limit: usize,
}

impl RetryPolicyBuilder<WantsInitialBackoff> {
    pub fn with_initial_backoff(
        self,
        initial_backoff: Duration,
    ) -> RetryPolicyBuilder<WantsMaxBackoff> {
        RetryPolicyBuilder(WantsMaxBackoff { initial_backoff })
    }

    pub fn with_initial_backoff_millis(
        self,
        initial_backoff_millis: u64,
    ) -> RetryPolicyBuilder<WantsMaxBackoff> {
        RetryPolicyBuilder(WantsMaxBackoff {
            initial_backoff: Duration::from_millis(initial_backoff_millis),
        })
    }
}

impl RetryPolicyBuilder<WantsMaxBackoff> {
    pub fn with_max_backoff(self, max_backoff: Duration) -> RetryPolicyBuilder<WantsLimit> {
        RetryPolicyBuilder(WantsLimit {
            initial_backoff: self.0.initial_backoff,
            max_backoff,
        })
    }

    pub fn with_max_backoff_millis(
        self,
        max_backoff_millis: u64,
    ) -> RetryPolicyBuilder<WantsLimit> {
        RetryPolicyBuilder(WantsLimit {
            initial_backoff: self.0.initial_backoff,
            max_backoff: Duration::from_millis(max_backoff_millis),
        })
    }
}

impl RetryPolicyBuilder<WantsLimit> {
    pub fn with_limit(self, limit: usize) -> RetryPolicyBuilder<MaybeWantsJitter> {
        RetryPolicyBuilder(MaybeWantsJitter {
            initial_backoff: self.0.initial_backoff,
            max_backoff: self.0.max_backoff,
            limit,
        })
    }
}

impl RetryPolicyBuilder<MaybeWantsJitter> {
    pub fn build_with_jitter(self, jitter: f32) -> RetryPolicy {
        RetryPolicy {
            initial_backoff: self.0.initial_backoff,
            max_backoff: self.0.max_backoff,
            limit: self.0.limit,
            jitter,
        }
    }

    pub fn build(self) -> RetryPolicy {
        RetryPolicy {
            initial_backoff: self.0.initial_backoff,
            max_backoff: self.0.max_backoff,
            limit: self.0.limit,
            jitter: DEFAULT_JITTER,
        }
    }
}

impl From<&RetrySettings> for RetryPolicy {
    fn from(settings: &RetrySettings) -> Self {
        Self {
            initial_backoff: Duration::from_millis(settings.initial_backoff_ms),
            max_backoff: Duration::from_millis(settings.max_backoff_ms),
            jitter: settings.jitter.unwrap_or(DEFAULT_JITTER),
            limit: settings.max_attempts,
        }
    }
}

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

pub fn retry_with_policy(policy: RetryPolicy) -> Retry {
    Retry {
        current: policy.initial_backoff.as_millis() as u64,
        maximum: policy.max_backoff.as_millis() as u64,
        jitter: policy.jitter,
        counter: 0,
        limit: policy.limit,
    }
}
