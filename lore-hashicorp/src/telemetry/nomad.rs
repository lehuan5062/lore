// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::env;

use opentelemetry::KeyValue;
use opentelemetry_sdk::resource::Resource;
use opentelemetry_sdk::resource::ResourceDetector;

/// Resource detector for Nomad orchestration environment.
///
/// Detects Nomad-specific resource attributes like allocation ID and job ID
/// from environment variables set by Nomad.
pub struct NomadResourceDetector;

impl ResourceDetector for NomadResourceDetector {
    fn detect(&self) -> Resource {
        let alloc_id = env::var("NOMAD_ALLOC_ID").ok();
        let job_id = env::var("NOMAD_JOB_ID").ok();

        Resource::builder_empty()
            .with_attributes(
                [
                    alloc_id.map(|name| KeyValue::new("nomad.alloc.id", name)),
                    job_id.map(|name| KeyValue::new("nomad.job.id", name)),
                ]
                .into_iter()
                .flatten(),
            )
            .build()
    }
}

#[cfg(test)]
mod tests {
    use opentelemetry::Key;
    use opentelemetry::Value;

    use super::*;

    #[test]
    fn test_nomad_resource_detector_with_env_vars() {
        temp_env::with_vars(
            [
                ("NOMAD_ALLOC_ID", Some("test-alloc-id")),
                ("NOMAD_JOB_ID", Some("test-job-id")),
            ],
            || {
                let resource = NomadResourceDetector.detect();
                assert_eq!(resource.len(), 2);

                assert_eq!(
                    resource.get(&Key::from_static_str("nomad.alloc.id")),
                    Some(Value::from("test-alloc-id"))
                );
                assert_eq!(
                    resource.get(&Key::from_static_str("nomad.job.id")),
                    Some(Value::from("test-job-id"))
                );
            },
        );
    }

    #[test]
    fn test_nomad_resource_detector_with_missing_env_vars() {
        // make sure no env var is accidentally set
        temp_env::with_vars_unset(["NOMAD_ALLOC_ID"], || {
            let resource = NomadResourceDetector.detect();

            assert_eq!(resource.len(), 0);
        });
    }
}
