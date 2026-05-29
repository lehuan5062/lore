// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use rand::distr::Alphanumeric;
use rand::distr::SampleString;

pub struct TempDir(std::path::PathBuf);

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        let name = format!(
            "{prefix}{}",
            Alphanumeric.sample_string(&mut rand::rng(), 8)
        );
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&path).expect("Failed to create temp directory");
        Self(path)
    }

    pub fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
