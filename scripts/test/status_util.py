# SPDX-FileCopyrightText: 2026 Epic Games, Inc.
# SPDX-License-Identifier: MIT
import re
from typing import Tuple, Iterable, List
from enum import Enum


class Operation(Enum):
    ADD = "A"
    MODIFY = "M"
    DELETE = "D"
    COPY = "C"
    MOVE = "V"


def make_regex(
    kind: Operation, name: str, move_destination_name: str | None = None
) -> Tuple[str, str]:
    if kind == Operation.MOVE:
        assert move_destination_name is not None
        return (
            rf"V\s+{re.escape(name)}/?\s+->\s+{re.escape(move_destination_name)}/?",
            f"V {name} -> {move_destination_name}",
        )
    else:
        return rf"{kind.value}/?\s+{re.escape(name)}/?", f"{kind.value} {name}"


def _missing_operations_in_status_text(
    status_text: str,
    operations: Iterable[Tuple[Operation, str] | Tuple[Operation, str, str]],
) -> List[str]:
    result = []
    for operation in operations:
        regex, explanation = make_regex(*operation)
        if not re.search(regex, status_text):
            result.append(explanation)
    return result


def verify_operations_in_status(
    status_text: str,
    staged_operations: Iterable[Tuple[Operation, str] | Tuple[Operation, str, str]],
    unstaged_operations: Iterable[Tuple[Operation, str] | Tuple[Operation, str, str]]
    | None = None,
    untracked_files: Iterable[str] | None = None,
):
    split_status = status_text.split("Untracked files:")
    untracked_text = ""
    if len(split_status) > 1:
        untracked_text = split_status[1]
    split_status = split_status[0].split("Changes not staged for commit:")
    unstaged_status = ""
    if len(split_status) > 1:
        unstaged_status = split_status[1]
    staged_status = split_status[0]

    error_explanation = ""
    missing = _missing_operations_in_status_text(staged_status, staged_operations)
    if missing:
        error_explanation = "\n".join(
            [
                "Missing from status:",
                *("\t" + missing_entry for missing_entry in missing),
            ]
        )
    if unstaged_operations is not None:
        missing = _missing_operations_in_status_text(
            unstaged_status, unstaged_operations
        )
        if missing:
            error_explanation = "\n".join(
                [
                    error_explanation,
                    "Missing from unstaged status:",
                    *("\t" + missing_entry for missing_entry in missing),
                ]
            )
    if untracked_files is not None:
        missing = _missing_operations_in_status_text(
            untracked_text, ((Operation.ADD, name) for name in untracked_files)
        )
        if missing:
            error_explanation = "\n".join(
                [
                    error_explanation,
                    "Missing from untracked files:",
                    *("\t" + missing_entry for missing_entry in missing),
                ]
            )

    assert error_explanation == "", (
        f"Status text:\n{status_text}\nExplanation:\n{error_explanation}"
    )
