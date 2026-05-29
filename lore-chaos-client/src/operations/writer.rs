// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::operations::RepoOperation;

pub struct OperationWriter {
    operations: Vec<RepoOperation>,
    output_file: File,
}

impl OperationWriter {
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        Self {
            operations: Vec::new(),
            output_file: File::create(file_path).unwrap(),
        }
    }

    pub fn add_operation(&mut self, operation: RepoOperation) {
        self.operations.push(operation);
    }

    pub fn write_to_file(mut self) {
        self.output_file
            .write_all(&serde_json::to_vec_pretty(&self.operations).unwrap())
            .unwrap();
    }
}
