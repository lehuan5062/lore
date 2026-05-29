// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lore::interface::LoreArray;
use lore::interface::LoreString;
use tracing::error;
use tracing::info;

use crate::chaos::config::RunnerConfig;
use crate::chaos::files::add_or_modify;
use crate::chaos::files::delete;
use crate::lore::interface::LoreInterface;
use crate::operations::FileChangeOperation;
use crate::operations::FileMergeOperation;
use crate::operations::MergeOperation;
use crate::operations::RepoOperation;
use crate::probability::ProbabilityEngine;
use crate::probability::ProbabilityType;

pub struct Performer {
    config: Rc<RunnerConfig>,
    lore_interface: LoreInterface,
    probability: Rc<RefCell<ProbabilityEngine>>,
    file_contents: usize,
}

impl Performer {
    pub fn new(
        config: Rc<RunnerConfig>,
        interface: LoreInterface,
        probability: Rc<RefCell<ProbabilityEngine>>,
    ) -> Self {
        Self {
            config,
            lore_interface: interface,
            probability,
            file_contents: 0,
        }
    }

    pub fn perform_repo_operation(&mut self, operation: RepoOperation) -> bool {
        if self.config.dry_run {
            return true;
        }
        match operation {
            RepoOperation::Commit(operation) => {
                for (file, operation) in operation.changes.iter() {
                    if let Err(err) = match operation {
                        FileChangeOperation::Add | FileChangeOperation::Modify => {
                            let contents = self.make_file_contents();
                            add_or_modify(file, &self.config, &contents)
                        }
                        FileChangeOperation::Delete => delete(file, &self.config),
                    } {
                        Self::handle_io_error(err);
                        return false;
                    }
                }
                let file_names = LoreArray::from_vec(
                    operation
                        .changes
                        .iter()
                        .map(|(file, _)| LoreString::from(self.config.path_in_repo(file)))
                        .collect(),
                );
                let force_commit = file_names.is_empty();
                self.lore_interface.stage_file(file_names);
                self.lore_interface
                    .commit(operation.description.into(), force_commit);
            }
            RepoOperation::GoToBranch(operation) => {
                self.lore_interface
                    .switch_branch_to(&LoreString::from(operation.target_branch));
            }
            RepoOperation::CreateBranch(operation) => {
                let lore_name = LoreString::from(operation.new_branch);
                self.lore_interface.create_branch(&lore_name);
                self.lore_interface.switch_branch_to(&lore_name);
            }
            RepoOperation::Merge(operation) => {
                if let Ok(conflicts) = self
                    .lore_interface
                    .merge(&LoreString::from(&operation.target_branch))
                    && let Err(err) = self.finish_merge(operation, conflicts)
                {
                    Self::handle_io_error(err);
                    return false;
                }
            }
            RepoOperation::Status(operation) => {
                self.lore_interface.status(&operation);
            }
            RepoOperation::BranchInfo(operation) => {
                self.lore_interface.branch_info(&operation);
            }
        }
        true
    }

    fn handle_io_error(error: std::io::Error) {
        error!("IO Error: {}", error);
    }

    fn finish_merge(
        &mut self,
        operation: MergeOperation,
        conflicts: Vec<LoreString>,
    ) -> std::io::Result<()> {
        info!("Resolving conflicts: {conflicts:?}");
        let resolution_lookup: HashMap<&str, &FileMergeOperation> = operation
            .resolutions
            .iter()
            .map(|(k, v)| (k.to_str().unwrap(), v))
            .collect();
        for conflict in conflicts.iter() {
            // A stack variable to store any on-the-fly generated resolution that can be passed by reference in the same way as the predicted resolutions.
            let mut runtime_resolution = None;
            let resolution = resolution_lookup
                .get(&conflict.as_str())
                .copied()
                .unwrap_or_else(|| {
                    runtime_resolution = Some(
                        self.probability
                            .borrow_mut()
                            .pick_file_merge_operation(ProbabilityType::Runtime),
                    );
                    runtime_resolution.as_ref().unwrap()
                });
            self.lore_interface.merge_file(
                &self.config,
                &self.config.lore_string_to_path_in_repo(conflict),
                resolution,
            )?;
        }
        if !conflicts.is_empty() {
            self.lore_interface.finish_merge();
        }
        Ok(())
    }

    fn make_file_contents(&mut self) -> String {
        let contents = format!("{}", self.file_contents);
        self.file_contents += 1;
        contents
    }
}
