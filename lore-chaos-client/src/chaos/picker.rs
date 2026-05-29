// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use crate::chaos::operation_context::OperationContext;
use crate::lore::state::LoreState;
use crate::operations::BranchInfoOperation;
use crate::operations::CommitOperation;
use crate::operations::CreateBranchOperation;
use crate::operations::GoToBranchOperation;
use crate::operations::MergeOperation;
use crate::operations::RepoOperation;
use crate::operations::RepoOperationKind;
use crate::operations::StatusOperation;
use crate::probability::ProbabilityEngine;

pub trait Picker {
    fn pick_repo_operation(&mut self) -> Option<(RepoOperation, Option<OperationContext<'_>>)>;
}

pub struct RandomPicker {
    probability: Rc<RefCell<ProbabilityEngine>>,
    lore_state: LoreState,
}

impl Picker for RandomPicker {
    fn pick_repo_operation(&mut self) -> Option<(RepoOperation, Option<OperationContext<'_>>)> {
        loop {
            let start_state = self.lore_state.clone();
            if let Some(operation) = self.try_pick_repo_operation() {
                return Some((
                    operation,
                    Some(OperationContext::new(start_state, &self.lore_state)),
                ));
            }
        }
    }
}

impl RandomPicker {
    pub fn new(probability: Rc<RefCell<ProbabilityEngine>>) -> Self {
        Self {
            probability,
            lore_state: LoreState::default(),
        }
    }

    fn try_pick_repo_operation(&mut self) -> Option<RepoOperation> {
        let kind = self.probability.borrow_mut().pick_repo_operation_kind();
        match kind {
            RepoOperationKind::Commit => Some(RepoOperation::Commit(self.pick_commit_operation())),
            RepoOperationKind::GoToBranch => self
                .pick_go_to_branch_operation()
                .map(RepoOperation::GoToBranch),
            RepoOperationKind::CreateBranch => Some(RepoOperation::CreateBranch(
                self.pick_create_branch_operation(),
            )),
            RepoOperationKind::Merge => self.pick_merge_operation().map(RepoOperation::Merge),
            RepoOperationKind::Status => Some(RepoOperation::Status(self.pick_status_operation())),
            RepoOperationKind::BranchInfo => {
                Some(RepoOperation::BranchInfo(self.pick_branch_info_operation()))
            }
        }
    }

    fn pick_commit_operation(&mut self) -> CommitOperation {
        let mut current_branch_files = Vec::new();
        let current_branch = self.lore_state.mut_current_branch();
        std::mem::swap(&mut current_branch_files, &mut current_branch.files);
        let (commit_operation, new_files) = self
            .probability
            .borrow_mut()
            .pick_file_operations(current_branch_files);
        current_branch.files = new_files;
        commit_operation
    }

    fn pick_go_to_branch_operation(&mut self) -> Option<GoToBranchOperation> {
        if let Some(target_branch) = self
            .probability
            .borrow_mut()
            .pick_other_branch(&self.lore_state)
        {
            self.lore_state.current_branch = target_branch;
            Some(GoToBranchOperation {
                target_branch: self.lore_state.branches[target_branch].name.clone(),
            })
        } else {
            None
        }
    }

    fn pick_create_branch_operation(&mut self) -> CreateBranchOperation {
        let new_branch_name = self
            .probability
            .borrow_mut()
            .make_branch_name(&self.lore_state);
        self.lore_state
            .add_new_current_branch(new_branch_name.clone());
        CreateBranchOperation {
            new_branch: new_branch_name,
        }
    }

    fn pick_merge_operation(&mut self) -> Option<MergeOperation> {
        let mut probability = self.probability.borrow_mut();
        if let Some(branches) = self
            .lore_state
            .mut_current_and_other_branch(&mut probability)
        {
            let mut current_branch_files = Vec::new();
            std::mem::swap(&mut current_branch_files, &mut branches.current.files);
            let (merge_operations, new_files) = probability
                .pick_merge_resolutions(current_branch_files, branches.other.files.as_slice());
            branches.current.files.extend(new_files);
            Some(MergeOperation {
                target_branch: branches.other.name.clone(),
                resolutions: merge_operations,
            })
        } else {
            None
        }
    }

    fn pick_status_operation(&mut self) -> StatusOperation {
        let mut probability = self.probability.borrow_mut();
        StatusOperation {
            staged: probability.make_bool(),
            unstaged: probability.make_bool(),
            sync_point: probability.make_bool(),
        }
    }

    fn pick_branch_info_operation(&mut self) -> BranchInfoOperation {
        let mut probability = self.probability.borrow_mut();
        if probability.make_bool() {
            BranchInfoOperation { name: None }
        } else {
            BranchInfoOperation {
                name: Some(
                    self.lore_state.branches[probability.pick_branch(&self.lore_state)]
                        .name
                        .clone(),
                ),
            }
        }
    }
}

pub struct JsonPicker {
    operation_iter: <Vec<RepoOperation> as IntoIterator>::IntoIter,
}

impl Picker for JsonPicker {
    fn pick_repo_operation(&mut self) -> Option<(RepoOperation, Option<OperationContext<'_>>)> {
        self.operation_iter
            .next()
            .map(|operation| (operation, None))
    }
}

impl JsonPicker {
    pub fn new(json_file: impl AsRef<Path>) -> Self {
        let mut json_bytes = Vec::new();
        File::open(json_file)
            .unwrap()
            .read_to_end(&mut json_bytes)
            .unwrap();

        let operations: Vec<RepoOperation> = serde_json::from_slice(&json_bytes).unwrap();

        Self {
            operation_iter: operations.into_iter(),
        }
    }
}
