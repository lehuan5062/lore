# SPDX-FileCopyrightText: 2026 Epic Games, Inc.
# SPDX-License-Identifier: MIT
r"""
Regression tests for Windows long-path (> MAX_PATH = 260) handling in clone.

On Windows, direct Win32 file-API calls (e.g. `MoveFileExW`) reject paths
longer than MAX_PATH (260 characters) unless the path carries the `\\?\`
verbatim prefix. lore's rename helpers call `MoveFileExW` directly, so any
clone that reassembles a file with an absolute path over MAX_PATH fails
the final temp-to-target rename with `os error 3` (ERROR_PATH_NOT_FOUND).

This test forces the failure mode by committing a multi-fragment file at
a deeply nested path and verifying that a subsequent clone succeeds.

Windows-only; the underlying constraint does not exist on Linux/macOS.
"""

import logging
import os
import platform

import pytest

from lore import Lore

logger = logging.getLogger(__name__)

# Cap a single path component. NTFS allows 255 UTF-16 code units per
# component; we stay well under.
_COMPONENT = "lpdir_abcdefghijklmnopqrstuvwxyz0123456789"  # 40 chars

# 5 components of ~43 chars + the leaf yields a relative path ~280 chars.
# Combined with a Windows pytest tmpdir prefix (~80–100 chars on a typical
# CI runner) the absolute clone path lands comfortably above 260. The
# `.~loretemp` temp variant adds another 10 chars on the rename source.
_DEPTH = 5

# Long leaf filename so the rename source name itself is over MAX_PATH.
_LEAF = "T_LongPathRegression_simulated_4K_ORDp.uasset"

# File payload size. The rename code path is only taken when the fragment
# is flagged `PayloadFragmented`. Non-fragmented fragments must have
# `size_content <= FRAGMENT_SIZE_THRESHOLD` (256 KiB —
# lore-base/src/types/mod.rs), so anything <= 256 KiB writes via the direct
# path and never invokes `MoveFileExW`. 512 KiB of random bytes is safely
# above the threshold and incompressible — the file is guaranteed to be
# split into multiple fragments and to trigger the rename on clone.
_PAYLOAD_BYTES = 512 * 1024


def _long_relative_path() -> str:
    """Compose a deterministic relative path comfortably over MAX_PATH."""
    parts = [f"{_COMPONENT}_{i:02d}" for i in range(_DEPTH)]
    parts.append(_LEAF)
    return "/".join(parts)


@pytest.mark.smoke
@pytest.mark.skipif(
    platform.system() != "Windows",
    reason="Long-path rename failure is a Win32 MAX_PATH constraint",
)
def test_clone_with_path_exceeding_max_path(new_lore_repo):
    r"""
    Cloning a repo whose nested files produce absolute paths longer than
    MAX_PATH must succeed. Without the `\\?\` prefix at the `MoveFileExW`
    call site, the temp-file rename fails with `os error 3`, surfacing as
    "Failed to clone file" mid-transfer.
    """
    repo: Lore = new_lore_repo()

    rel_path = _long_relative_path()

    # Confirm the absolute clone-side path will exceed 260 chars on this
    # runner before exercising the bug — if pytest's tmpdir is unusually
    # short, the test would silently pass without testing anything.
    expected_abs_len = len(repo.path) + 1 + len(rel_path)
    assert expected_abs_len > 260, (
        f"Test path is not long enough to exercise MAX_PATH on this runner: "
        f"{expected_abs_len} chars (need > 260). Increase _DEPTH."
    )

    # Random bytes so compression can't shrink the file back under the
    # fragment threshold; this guarantees the clone side takes the
    # temp-file-then-rename branch that calls `MoveFileExW`.
    repo.write_commit_push(
        "Seed long-path file",
        {rel_path: os.urandom(_PAYLOAD_BYTES)},
    )

    # The clone's absolute file path is what overflows MAX_PATH — the source
    # repo's path is the same length, but writes there succeed because they
    # go through `_fix_path`'s `\\?\` prefix on the Python side. The CLI
    # receives a normal absolute path for the clone target.
    clone = repo.clone()

    assert clone.file_exists(rel_path), (
        f"Cloned file missing at {rel_path}; long-path rename likely failed"
    )
