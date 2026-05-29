// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use crate::lore::state::LoreState;

#[derive(Debug)]
#[allow(unused)]
pub struct OperationContext<'a> {
    pub start_state: LoreState,
    pub end_state: &'a LoreState,
}

impl<'a> OperationContext<'a> {
    pub fn new(start_state: LoreState, end_state: &'a LoreState) -> Self {
        Self {
            start_state,
            end_state,
        }
    }
}
