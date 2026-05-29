// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Runner for compile-fail tests using trybuild.
//!
//! These tests verify that the type system enforces the read/write boundary
//! on the mutable store handles introduced for parallel read access:
//!
//! - `ReadHandle` does not expose `store` / `compare_and_swap` / `flush`.
//! - `write_mutable_store` requires a `&RepositoryWriteToken` in scope, so a
//!   read-only command callback (which has no token bound) cannot reach it.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
