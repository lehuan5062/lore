# SPDX-FileCopyrightText: 2026 Epic Games, Inc.
# SPDX-License-Identifier: MIT
"""Shared test utility functions."""

from pathlib import Path


def posix_join(*parts: str) -> str:
    """Join path components using forward slashes.

    Lore always returns paths with forward slashes regardless of platform.
    Using os.path.join on Windows would produce backslashes, causing
    mismatches in path comparisons and regex patterns against Lore output.
    """
    return "/".join(parts)


def to_posix(path: str | Path) -> str:
    """Normalize a path to forward slashes for comparison against Lore output.

    Lore JSON output always uses forward slashes regardless of platform.
    Use this to normalize os.path.join results before comparing against
    paths extracted from Lore status, unstage events, etc.
    """
    return str(path).replace("\\", "/")
