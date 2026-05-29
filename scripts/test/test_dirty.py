# SPDX-FileCopyrightText: 2026 Epic Games, Inc.
# SPDX-License-Identifier: MIT
"""Smoke tests for dirty file tracking.

Tests the file dirty API, stage/unstage/reset/commit interactions with dirty flags,
scan (--scan) setting dirty flags, and cross-operation sequences.
All tests use --json structured output to validate event data.
"""

import json
import logging
import os

import pytest
from lore_parsers import parse_status_json
from test_utils import to_posix

from lore import Lore

logger = logging.getLogger(__name__)


def get_status_files(repo: Lore, **kwargs) -> list[dict]:
    """Get status file events via JSON output."""
    output = repo.status(json=True, offline=True, **kwargs)
    return parse_status_json(output)


def get_status_files_twice(repo: Lore, **kwargs) -> list[dict]:
    """Run status twice with the same args and assert the second invocation
    reports the same set of files with identical dirty/staged flags as the
    first. Returns the entries from the second run.

    Regression guard: --scan (and its --unstaged alias) must be idempotent —
    a second invocation must not forget files (added, modified, or deleted)
    that the first invocation reported.
    """
    first = get_status_files(repo, **kwargs)
    second = get_status_files(repo, **kwargs)

    def fingerprint(entries: list[dict]) -> list[tuple]:
        return sorted(
            (to_posix(e.get("path", "")), e.get("flagDirty"), e.get("flagStaged"))
            for e in entries
        )

    first_fp = fingerprint(first)
    second_fp = fingerprint(second)
    assert first_fp == second_fp, (
        f"status should be idempotent across repeated invocations with {kwargs}.\n"
        f"first:  {first_fp}\n"
        f"second: {second_fp}"
    )
    return second


def find_status_entry(entries: list[dict], path: str) -> dict | None:
    """Find a status entry by path (posix-normalized)."""
    target = to_posix(path)
    for entry in entries:
        if to_posix(entry.get("path", "")) == target:
            return entry
    return None


def has_staged_anchor(repo: Lore) -> bool:
    """Check whether the repository has a staged revision."""
    output = repo.status(json=True, offline=True)
    zero_hash = "0" * 64
    for line in output.splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        data = event.get("data", {})
        revision_staged = data.get("revisionStaged", "")
        if revision_staged and revision_staged != zero_hash:
            return True
    return False


# ===========================================================================
# Task 18: file dirty API (modify/add/delete/dir/move/copy)
# ===========================================================================


@pytest.mark.smoke
def test_dirty_modify(new_lore_repo):
    """Mark an existing file as dirty and verify status shows it."""
    repo: Lore = new_lore_repo()

    # Create, stage, commit a file
    with repo.open_file("file.txt", "w+") as f:
        f.write("original content\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify the file on disk
    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content\n")

    # Mark as dirty
    repo.dirty("file.txt", offline=True)

    # Verify status shows dirty file
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "file.txt should appear in status"
    assert entry["flagDirty"] is True, "file.txt should be flagDirty"
    assert entry["flagStaged"] is False, "file.txt should not be flagStaged"


@pytest.mark.smoke
def test_dirty_add(new_lore_repo):
    """Mark a new file as dirty (add) and verify status."""
    repo: Lore = new_lore_repo()

    # Create and commit an initial file
    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Create a new file (not in revision) and mark dirty
    with repo.open_file("new_file.txt", "w+") as f:
        f.write("new content\n")
    repo.dirty("new_file.txt", offline=True)

    # Verify status
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "new_file.txt")
    assert entry is not None, "new_file.txt should appear in status"
    assert entry["flagDirty"] is True


@pytest.mark.smoke
def test_dirty_delete(new_lore_repo):
    """Mark a deleted file as dirty and verify status."""
    repo: Lore = new_lore_repo()

    with repo.open_file("to_delete.txt", "w+") as f:
        f.write("will be deleted\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Delete the file from disk
    os.remove(os.path.join(repo.path, "to_delete.txt"))

    # Mark as dirty
    repo.dirty("to_delete.txt", offline=True)

    # Verify status shows dirty delete
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "to_delete.txt")
    assert entry is not None, "to_delete.txt should appear in status"
    assert entry["flagDirty"] is True


@pytest.mark.smoke
def test_dirty_directory(new_lore_repo):
    """Mark a directory as dirty and verify all children are processed."""
    repo: Lore = new_lore_repo()

    # Create directory with files, stage, commit
    repo.make_dirs("src")
    with repo.open_file(os.path.join("src", "a.txt"), "w+") as f:
        f.write("aaa\n")
    with repo.open_file(os.path.join("src", "b.txt"), "w+") as f:
        f.write("bbb\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify one, delete one, add one
    with repo.open_file(os.path.join("src", "a.txt"), "w+") as f:
        f.write("aaa modified\n")
    os.remove(os.path.join(repo.path, "src", "b.txt"))
    with repo.open_file(os.path.join("src", "c.txt"), "w+") as f:
        f.write("new file\n")

    # Mark the directory as dirty
    repo.dirty("src", offline=True)

    # Verify all children are dirty
    entries = get_status_files(repo)
    a_entry = find_status_entry(entries, os.path.join("src", "a.txt"))
    b_entry = find_status_entry(entries, os.path.join("src", "b.txt"))
    c_entry = find_status_entry(entries, os.path.join("src", "c.txt"))

    assert a_entry is not None, "src/a.txt should appear (modify)"
    assert a_entry["flagDirty"] is True
    assert b_entry is not None, "src/b.txt should appear (delete)"
    assert b_entry["flagDirty"] is True
    assert c_entry is not None, "src/c.txt should appear (add)"
    assert c_entry["flagDirty"] is True


@pytest.mark.smoke
def test_dirty_move(new_lore_repo):
    """Mark a file as dirty-moved and verify status."""
    repo: Lore = new_lore_repo()

    repo.make_dirs("src")
    repo.make_dirs("dest")
    with repo.open_file(os.path.join("src", "file.txt"), "w+") as f:
        f.write("content\n")
    with repo.open_file(os.path.join("dest", "placeholder.txt"), "w+") as f:
        f.write("placeholder\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Move the file on disk
    os.rename(
        os.path.join(repo.path, "src", "file.txt"),
        os.path.join(repo.path, "dest", "file.txt"),
    )

    # Mark as dirty move
    repo.dirty_move(
        os.path.join("src", "file.txt"), os.path.join("dest", "file.txt"), offline=True
    )

    # Verify
    entries = get_status_files(repo)
    # The moved file should appear at the destination
    entry = find_status_entry(entries, os.path.join("dest", "file.txt"))
    assert entry is not None, "dest/file.txt should appear in status"
    assert entry["flagDirty"] is True


@pytest.mark.smoke
def test_dirty_copy(new_lore_repo):
    """Mark a file as dirty-copied and verify status."""
    repo: Lore = new_lore_repo()

    with repo.open_file("original.txt", "w+") as f:
        f.write("content\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Mark as dirty copy
    repo.dirty_copy("original.txt", "copy.txt", offline=True)

    # Verify copy appears in status
    entries = get_status_files(repo)
    copy_entry = find_status_entry(entries, "copy.txt")
    assert copy_entry is not None, "copy.txt should appear in status"
    assert copy_entry["flagDirty"] is True

    # Original should NOT be dirty
    orig_entry = find_status_entry(entries, "original.txt")
    assert orig_entry is None, "original.txt should not appear (not dirty)"


@pytest.mark.smoke
def test_dirty_ignore(new_lore_repo):
    """File that doesn't exist on disk or in revision is ignored."""
    repo: Lore = new_lore_repo()

    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Mark non-existent file as dirty — should be silently ignored
    repo.dirty("ghost.txt", offline=True)

    # Verify nothing extra in status
    entries = get_status_files(repo)
    ghost_entry = find_status_entry(entries, "ghost.txt")
    assert ghost_entry is None, "ghost.txt should not appear"


# ===========================================================================
# Task 15: stage/unstage interaction with Dirty
# ===========================================================================


@pytest.mark.smoke
def test_stage_preserves_dirty(new_lore_repo):
    """Staging a dirty file preserves the Dirty flag (orthogonal)."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify and mark dirty
    with repo.open_file("file.txt", "w+") as f:
        f.write("modified\n")
    repo.dirty("file.txt", offline=True)

    # Stage the file
    repo.stage(offline=True)

    # Verify both dirty AND staged
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "file.txt should appear"
    assert entry["flagDirty"] is True, "Dirty should be preserved after stage"
    assert entry["flagStaged"] is True, "Should be staged"


@pytest.mark.smoke
def test_unstage_preserves_dirty_when_file_differs(new_lore_repo):
    """Unstaging a dirty+staged file preserves Dirty when file still differs."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content longer\n")
    repo.dirty("file.txt", offline=True)
    repo.stage(offline=True)

    # Unstage
    repo.unstage(offline=True)

    # Dirty should remain (file still differs from committed)
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "file.txt should appear"
    assert entry["flagDirty"] is True, "Dirty should remain after unstage"
    assert entry["flagStaged"] is False, "Should not be staged after unstage"


@pytest.mark.smoke
def test_unstage_preserves_anchor_when_dirty_remain(new_lore_repo):
    """Unstaging all staged files preserves anchor if dirty-only files remain."""
    repo: Lore = new_lore_repo()

    with repo.open_file("staged.txt", "w+") as f:
        f.write("staged\n")
    with repo.open_file("dirty.txt", "w+") as f:
        f.write("dirty original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify both
    with repo.open_file("staged.txt", "w+") as f:
        f.write("staged modified\n")
    with repo.open_file("dirty.txt", "w+") as f:
        f.write("dirty modified longer\n")

    # Mark dirty.txt as dirty only
    repo.dirty("dirty.txt", offline=True)
    # Stage only staged.txt
    repo.stage("staged.txt", offline=True)
    # Unstage staged.txt
    repo.unstage("staged.txt", offline=True)

    # Anchor should still exist (dirty.txt is still dirty)
    assert has_staged_anchor(repo), "Anchor should be preserved when dirty nodes remain"


# ===========================================================================
# Task 17: reset interaction with Dirty
# ===========================================================================


@pytest.mark.smoke
def test_reset_clears_dirty(new_lore_repo):
    """Resetting a dirty-only file clears the Dirty flag."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content longer\n")
    repo.dirty("file.txt", offline=True)

    # Verify dirty before reset
    entries = get_status_files(repo)
    assert find_status_entry(entries, "file.txt") is not None

    # Reset
    repo.reset("file.txt", offline=True)

    # File should be restored
    with repo.open_file("file.txt", "r") as f:
        content = f.read()
    assert content == "original\n", "File content should be restored"

    # Dirty should be cleared
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "file.txt")
    assert entry is None, "file.txt should not appear after reset (clean)"


# ===========================================================================
# Task 16: commit with mixed dirty/staged states
# ===========================================================================


@pytest.mark.smoke
def test_commit_preserves_dirty_only(new_lore_repo):
    """Commit clears dirty on committed files, preserves dirty-only files."""
    repo: Lore = new_lore_repo()

    with repo.open_file("committed.txt", "w+") as f:
        f.write("will commit\n")
    with repo.open_file("dirty_only.txt", "w+") as f:
        f.write("stay dirty\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify both
    with repo.open_file("committed.txt", "w+") as f:
        f.write("committed modified longer\n")
    with repo.open_file("dirty_only.txt", "w+") as f:
        f.write("dirty modified longer\n")

    # Mark both dirty
    repo.dirty(["committed.txt", "dirty_only.txt"], offline=True)

    # Stage only committed.txt
    repo.stage("committed.txt", offline=True)

    # Commit
    repo.commit(offline=True)

    # dirty_only.txt should still be dirty
    entries = get_status_files(repo)
    dirty_entry = find_status_entry(entries, "dirty_only.txt")
    assert dirty_entry is not None, "dirty_only.txt should still appear"
    assert dirty_entry["flagDirty"] is True, "dirty_only.txt should still be dirty"

    # committed.txt should be clean
    committed_entry = find_status_entry(entries, "committed.txt")
    assert committed_entry is None, "committed.txt should not appear (committed)"

    # Anchor should still exist
    assert has_staged_anchor(repo), "Anchor should exist (dirty_only.txt remains)"


@pytest.mark.smoke
def test_commit_deletes_anchor_when_clean(new_lore_repo):
    """Commit deletes anchor when no dirty or staged nodes remain."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("file.txt", "w+") as f:
        f.write("modified\n")
    repo.dirty("file.txt", offline=True)
    repo.stage(offline=True)
    repo.commit(offline=True)

    # No dirty, no staged — anchor should be gone
    assert not has_staged_anchor(repo), "Anchor should be deleted when clean"


# ===========================================================================
# Task 19: --scan setting dirty flags
# ===========================================================================


@pytest.mark.smoke
def test_scan_detects_modified_file(new_lore_repo):
    """--scan detects filesystem modifications and persists Dirty."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify file without calling dirty
    with repo.open_file("file.txt", "w+") as f:
        f.write("modified by scan detection\n")

    # Run scan twice — should detect the change and be idempotent
    entries = get_status_files_twice(repo, scan=True)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "file.txt should be detected by scan"

    # Verify persisted — status without scan should show it
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "file.txt should persist as dirty after scan"
    assert entry["flagDirty"] is True


@pytest.mark.smoke
def test_scan_detects_deleted_file(new_lore_repo):
    """--scan detects deleted files."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("content\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    os.remove(os.path.join(repo.path, "file.txt"))

    entries = get_status_files_twice(repo, scan=True)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "Deleted file should be detected by scan"


@pytest.mark.smoke
def test_scan_detects_new_file(new_lore_repo):
    """--scan detects new (untracked) files and persists Dirty+Add."""
    repo: Lore = new_lore_repo()

    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("new_file.txt", "w+") as f:
        f.write("untracked\n")

    entries = get_status_files_twice(repo, scan=True)
    entry = find_status_entry(entries, "new_file.txt")
    assert entry is not None, "New file should be detected by scan"

    # Verify persisted — status without scan should show it
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "new_file.txt")
    assert entry is not None, "New file should persist as dirty after scan"
    assert entry["flagDirty"] is True


@pytest.mark.smoke
def test_scan_detects_new_empty_file(new_lore_repo):
    """--scan detects empty untracked files idempotently.

    An empty file hashes to the zero address, which equals the zero address
    of a DirtyAdd state node — a naive content comparison would classify
    such a file as unmodified and clear its dirty flag on the second scan.
    """
    repo: Lore = new_lore_repo()

    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Empty file (zero bytes) — hashes to zero address.
    with repo.open_file("empty.txt", "w+"):
        pass

    entries = get_status_files_twice(repo, scan=True)
    entry = find_status_entry(entries, "empty.txt")
    assert entry is not None, "Empty new file should be detected by scan"
    assert entry["flagDirty"] is True

    # Verify persisted — status without scan should still show it.
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "empty.txt")
    assert entry is not None, "Empty new file should persist as dirty after scan"
    assert entry["flagDirty"] is True


@pytest.mark.smoke
def test_scan_drops_node_when_unstaged_add_deleted(new_lore_repo):
    """--scan discards a DirtyAdd node when its file is removed from disk.

    An unstaged add only exists in staged state. Removing the file reverts
    the add, so the node must not remain in state_staged — neither as
    DirtyAdd nor as a spurious Delete change.
    """
    repo: Lore = new_lore_repo()

    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("empty.txt", "w+"):
        pass

    entries = get_status_files_twice(repo, scan=True)
    assert find_status_entry(entries, "empty.txt") is not None

    repo.remove_file("empty.txt")

    entries = get_status_files_twice(repo, scan=True)
    assert find_status_entry(entries, "empty.txt") is None, (
        "Deleted unstaged-add should be removed from state, not reported"
    )

    # Confirm without scan as well — the node must be gone from state.
    entries = get_status_files(repo)
    assert find_status_entry(entries, "empty.txt") is None


@pytest.mark.smoke
def test_unstaged_alias_works(new_lore_repo):
    """--unstaged is a hidden alias for --scan."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("file.txt", "w+") as f:
        f.write("modified\n")

    entries = get_status_files_twice(repo, unstaged=True)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "--unstaged should work as alias for --scan"


# ===========================================================================
# Task 14: cross-operation sequences
# ===========================================================================


@pytest.mark.smoke
def test_dirty_stage_unstage_sequence(new_lore_repo):
    """dirty → stage → unstage preserves dirty state."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content longer\n")

    # dirty → stage → unstage
    repo.dirty("file.txt", offline=True)
    repo.stage(offline=True)
    repo.unstage(offline=True)

    # Should still be dirty (file still differs)
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "file.txt")
    assert entry is not None, "file.txt should still appear"
    assert entry["flagDirty"] is True, "Dirty should survive stage→unstage"
    assert entry["flagStaged"] is False, "Should not be staged"


@pytest.mark.smoke
def test_dirty_stage_commit_preserves_other_dirty(new_lore_repo):
    """dirty → stage → commit: committed file clean, other dirty survives."""
    repo: Lore = new_lore_repo()

    with repo.open_file("a.txt", "w+") as f:
        f.write("aaa\n")
    with repo.open_file("b.txt", "w+") as f:
        f.write("bbb\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("a.txt", "w+") as f:
        f.write("aaa modified longer\n")
    with repo.open_file("b.txt", "w+") as f:
        f.write("bbb modified longer\n")

    repo.dirty(["a.txt", "b.txt"], offline=True)
    repo.stage("a.txt", offline=True)
    repo.commit(offline=True)

    entries = get_status_files(repo)
    a_entry = find_status_entry(entries, "a.txt")
    b_entry = find_status_entry(entries, "b.txt")
    assert a_entry is None, "a.txt should be clean after commit"
    assert b_entry is not None, "b.txt should survive commit"
    assert b_entry["flagDirty"] is True


@pytest.mark.smoke
def test_scan_clears_stale_dirty_on_reverted_file(new_lore_repo):
    """--scan clears Dirty on a file that was reverted to match committed content."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify and mark dirty
    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content longer\n")
    repo.dirty("file.txt", offline=True)

    # Verify dirty
    entries = get_status_files(repo)
    assert find_status_entry(entries, "file.txt") is not None, "Should be dirty before revert"

    # Revert the file back to original content
    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")

    # Run scan twice — should detect file matches committed, clear Dirty, and stay clean
    get_status_files_twice(repo, scan=True)

    # Verify dirty cleared — status without scan should show nothing
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "file.txt")
    assert entry is None, "Dirty should be cleared after scan on reverted file"


@pytest.mark.smoke
def test_scan_clears_one_dirty_keeps_other(new_lore_repo):
    """--scan clears Dirty on reverted file but keeps Dirty on still-modified file."""
    repo: Lore = new_lore_repo()

    with repo.open_file("reverted.txt", "w+") as f:
        f.write("original reverted\n")
    with repo.open_file("still_dirty.txt", "w+") as f:
        f.write("original dirty\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Modify both and mark dirty
    with repo.open_file("reverted.txt", "w+") as f:
        f.write("modified reverted longer\n")
    with repo.open_file("still_dirty.txt", "w+") as f:
        f.write("modified dirty longer\n")
    repo.dirty(["reverted.txt", "still_dirty.txt"], offline=True)

    # Revert one file
    with repo.open_file("reverted.txt", "w+") as f:
        f.write("original reverted\n")

    # Scan twice — idempotent
    get_status_files_twice(repo, scan=True)

    # Check: reverted.txt should be clean, still_dirty.txt should remain dirty
    entries = get_status_files(repo)
    reverted_entry = find_status_entry(entries, "reverted.txt")
    dirty_entry = find_status_entry(entries, "still_dirty.txt")
    assert reverted_entry is None, "reverted.txt should be clean after scan"
    assert dirty_entry is not None, "still_dirty.txt should still be dirty"
    assert dirty_entry["flagDirty"] is True


@pytest.mark.smoke
def test_scan_after_commit_shows_remaining_dirty(new_lore_repo):
    """After commit with dirty-only files, scan confirms they're still dirty."""
    repo: Lore = new_lore_repo()

    with repo.open_file("committed.txt", "w+") as f:
        f.write("committed\n")
    with repo.open_file("remaining.txt", "w+") as f:
        f.write("remaining\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("committed.txt", "w+") as f:
        f.write("committed modified\n")
    with repo.open_file("remaining.txt", "w+") as f:
        f.write("remaining modified longer\n")

    repo.dirty(["committed.txt", "remaining.txt"], offline=True)
    repo.stage("committed.txt", offline=True)
    repo.commit(offline=True)

    # Scan twice should confirm remaining.txt is still dirty
    entries = get_status_files_twice(repo, scan=True)
    entry = find_status_entry(entries, "remaining.txt")
    assert entry is not None, "remaining.txt should still appear after commit + scan"
    assert entry["flagDirty"] is True


# ===========================================================================
# Dirty add in new directories, nonexistent paths, ignored paths
# ===========================================================================


@pytest.mark.smoke
def test_dirty_add_in_new_directory(new_lore_repo):
    """file dirty on a new file in a new directory creates intermediate directory nodes."""
    repo: Lore = new_lore_repo()

    with repo.open_file("existing.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Create a file in a new directory (not in current revision)
    repo.make_dirs("new_dir/sub_dir")
    with repo.open_file("new_dir/sub_dir/new_file.txt", "w+") as f:
        f.write("new content\n")

    repo.dirty("new_dir/sub_dir/new_file.txt", offline=True)

    entries = get_status_files(repo)
    entry = find_status_entry(entries, "new_dir/sub_dir/new_file.txt")
    assert entry is not None, "new file in new dir should be dirty"
    assert entry["flagDirty"] is True


@pytest.mark.smoke
def test_dirty_nonexistent_path_ignored(new_lore_repo):
    """file dirty on a path that doesn't exist on disk or in state is ignored."""
    repo: Lore = new_lore_repo()

    with repo.open_file("existing.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Call dirty on a completely nonexistent path
    repo.dirty("nonexistent/path/file.txt", offline=True)

    # Should have no effect
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "nonexistent/path/file.txt")
    assert entry is None, "nonexistent path should be ignored"


@pytest.mark.smoke
def test_dirty_ignored_path_skipped(new_lore_repo):
    """file dirty on a path under an ignored directory is skipped."""
    repo: Lore = new_lore_repo()

    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Set up ignore rule for some/path/
    with repo.open_file(repo.ignore_file(), "w+") as f:
        f.write("some/path/\n")

    # Create a file under the ignored path
    repo.make_dirs("some/path")
    with repo.open_file("some/path/file.txt", "w+") as f:
        f.write("should be ignored\n")

    repo.dirty("some/path/file.txt", offline=True)

    entries = get_status_files(repo)
    entry = find_status_entry(entries, "some/path/file.txt")
    assert entry is None, "file under ignored path should be skipped"


@pytest.mark.smoke
def test_commit_excludes_dirty_only_node(new_lore_repo):
    """Dirty-only nodes must not change the committed revision.

    Setup committed with a baseline file inside a kept directory. Then
    in one staged commit, three independent things happen:

    1. A file in a brand-new directory is staged — must end up in the
       committed tree.
    2. A file in a different brand-new directory is marked dirty (add)
       without being staged — its node and its dirty-add parent must be
       discarded before the merkle tree is sealed.
    3. A previously-committed file is deleted on disk and marked dirty
       (delete) without being staged — the dirty-delete must be reverted
       at commit time so the file stays in the committed tree.

    After commit, status (no scan) must still flag both pending dirty
    entries; after --reset drops the staged tracking, a repository dump
    of the latest committed revision must show the staged path and the
    untouched previously-committed file, but not the dirty-add path.
    """
    repo: Lore = new_lore_repo()

    # Base revision with a file that will later be deleted+dirtied.
    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.make_dirs("kept_dir")
    with repo.open_file("kept_dir/kept.txt", "w+") as f:
        f.write("kept content here\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Stage a file in a brand-new directory.
    repo.make_dirs("staged_dir")
    with repo.open_file("staged_dir/staged.txt", "w+") as f:
        f.write("staged content\n")
    repo.stage("staged_dir/staged.txt", offline=True)

    # Mark a file in a different brand-new directory as dirty (add).
    repo.make_dirs("dirty_dir")
    with repo.open_file("dirty_dir/dirty.txt", "w+") as f:
        f.write("dirty content\n")
    repo.dirty("dirty_dir/dirty.txt", offline=True)

    # Delete a committed file from disk and mark it dirty (delete) — not
    # staged.
    os.remove(os.path.join(repo.path, "kept_dir", "kept.txt"))
    repo.dirty("kept_dir/kept.txt", offline=True)

    # Before commit, status reports both dirty entries as pending and the
    # deleted file as an unstaged delete.
    pre = get_status_files(repo)
    pre_add = find_status_entry(pre, "dirty_dir/dirty.txt")
    assert pre_add is not None, "dirty add should be reported pre-commit"
    assert pre_add["flagDirty"] is True
    assert pre_add["flagStaged"] is False
    pre_del = find_status_entry(pre, "kept_dir/kept.txt")
    assert pre_del is not None, "dirty delete should be reported pre-commit"
    assert pre_del["flagDirty"] is True
    assert pre_del["flagStaged"] is False
    assert pre_del.get("action") == "delete", (
        f"kept_dir/kept.txt should report action=delete, got {pre_del.get('action')}"
    )

    repo.commit(offline=True)

    # Status (no scan) must still flag both dirty entries as pending —
    # neither was staged, so commit must leave them tracked.
    entries = get_status_files(repo)
    dirty_add = find_status_entry(entries, "dirty_dir/dirty.txt")
    assert dirty_add is not None, "dirty add should remain pending after commit"
    assert dirty_add["flagDirty"] is True
    assert dirty_add["flagStaged"] is False
    dirty_del = find_status_entry(entries, "kept_dir/kept.txt")
    assert dirty_del is not None, "dirty delete should remain pending after commit"
    assert dirty_del["flagDirty"] is True
    assert dirty_del["flagStaged"] is False
    assert dirty_del.get("action") == "delete", (
        f"kept_dir/kept.txt should still report action=delete, got {dirty_del.get('action')}"
    )
    # The staged file is now part of the committed revision and must not
    # appear in status anymore.
    assert find_status_entry(entries, "staged_dir/staged.txt") is None, (
        "staged_dir/staged.txt should be clean after commit"
    )

    # Drop the tracked staged state so dump shows only the committed tree.
    repo.status(reset=True, offline=True)

    dump = repo.repository_dump()
    assert "staged.txt" in dump, (
        f"staged_dir/staged.txt should appear in committed revision:\n{dump}"
    )
    # Dirty-only add must be absent from the committed tree.
    assert "dirty.txt" not in dump, (
        f"dirty-only added file should not appear in committed revision:\n{dump}"
    )
    assert "dirty_dir" not in dump, (
        f"dirty-only added directory should not appear in committed revision:\n{dump}"
    )
    # Dirty-only delete must be reverted — the file is still in the tree.
    assert "kept.txt" in dump, (
        f"dirty-only deleted file should remain in committed revision:\n{dump}"
    )
    assert "kept_dir" in dump, (
        f"directory of dirty-only deleted file should remain in committed revision:\n{dump}"
    )


@pytest.mark.smoke
def test_status_reapplies_ignore_to_dirty_add(new_lore_repo):
    """Status applies the current ignore filter to dirty-marked unstaged
    files. After marking a file dirty in a subdirectory two levels deep,
    status reports it as an unstaged add. Adding the parent directory to
    the ignore file and re-running status must drop the file from the
    report — the user may have updated the ignore between the dirty mark
    and the status query.
    """
    repo: Lore = new_lore_repo()

    with repo.open_file("base.txt", "w+") as f:
        f.write("base\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Create an untracked file two levels deep and mark it dirty.
    repo.make_dirs("some/path")
    with repo.open_file("some/path/file.txt", "w+") as f:
        f.write("new content\n")
    repo.dirty("some/path/file.txt", offline=True)

    # Before the ignore is added, status reports the file as an unstaged add.
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "some/path/file.txt")
    assert entry is not None, (
        "some/path/file.txt should be reported as unstaged add before ignore"
    )
    assert entry["flagDirty"] is True
    assert entry["flagStaged"] is False

    # Add the second-level directory to the ignore file after the dirty
    # mark has already been persisted.
    with repo.open_file(repo.ignore_file(), "w+") as f:
        f.write("some/path/\n")

    # Status must re-apply the current ignore filter and stop reporting
    # the dirty-marked file as unstaged add.
    entries = get_status_files(repo)
    entry = find_status_entry(entries, "some/path/file.txt")
    assert entry is None, (
        "some/path/file.txt should not be reported after ignore is added"
    )


# ===========================================================================
# Dirty flag does not block operations that check filesystem state
# ===========================================================================


@pytest.mark.smoke
def test_branch_switch_with_stale_dirty_flag(new_lore_repo):
    """Branch switch proceeds when a file's Dirty flag is set but its on-disk content matches the committed revision."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Target branch to switch to.
    repo.branch_create("other", offline=True)
    repo.branch_switch("main", offline=True)

    # Modify, mark dirty, revert content so the filesystem matches the committed revision.
    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content longer\n")
    repo.dirty("file.txt", offline=True)
    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")

    repo.branch_switch("other", offline=True)
    assert "On branch other" in repo.status(offline=True)


@pytest.mark.smoke
def test_branch_create_with_stale_dirty_flag(new_lore_repo):
    """Branch create proceeds when a file's Dirty flag is set but its on-disk content matches the committed revision."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content longer\n")
    repo.dirty("file.txt", offline=True)
    with repo.open_file("file.txt", "w+") as f:
        f.write("original\n")

    repo.branch_create("new-branch", offline=True)
    assert "On branch new-branch" in repo.status(offline=True)


@pytest.mark.smoke
def test_sync_with_stale_dirty_flag(new_lore_repo):
    """Sync proceeds when a file's Dirty flag is set but its on-disk content matches the current revision."""
    repo: Lore = new_lore_repo()

    # Two commits give sync a target to move toward.
    with repo.open_file("file.txt", "w+") as f:
        f.write("v1\n")
    repo.stage(offline=True)
    repo.commit(offline=True)
    rev_v1 = repo.revision_history(offline=True)[0].signature

    with repo.open_file("file.txt", "w+") as f:
        f.write("v2 longer content\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Step back to v1, then mark file dirty against an intermediate modification before reverting on disk.
    repo.sync(rev_v1, offline=True)
    with repo.open_file("file.txt", "w+") as f:
        f.write("modified content longer\n")
    repo.dirty("file.txt", offline=True)
    with repo.open_file("file.txt", "w+") as f:
        f.write("v1\n")

    repo.sync(offline=True)
    with repo.open_file("file.txt", "r") as f:
        assert f.read() == "v2 longer content\n"


@pytest.mark.smoke
def test_status_reset_clears_tracked_state(new_lore_repo):
    """--reset drops the existing tracked state before computing status.

    Phase 1: with tracked dirty files and a staged file, --reset alone
    clears all tracking and returns an empty status.

    Phase 2: after reset, modifying and staging a single file then
    running --reset --scan drops the stage and rescans from the
    filesystem, listing the file as an unstaged modification against
    the committed revision.
    """
    repo: Lore = new_lore_repo()

    for name in ("a.txt", "b.txt", "c.txt", "d.txt"):
        with repo.open_file(name, "w+") as f:
            f.write(f"{name} original\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Track two files as dirty
    with repo.open_file("a.txt", "w+") as f:
        f.write("a modified content longer\n")
    with repo.open_file("b.txt", "w+") as f:
        f.write("b modified content longer\n")
    repo.dirty(["a.txt", "b.txt"], offline=True)

    # Stage a third file
    with repo.open_file("c.txt", "w+") as f:
        f.write("c modified content longer\n")
    repo.stage("c.txt", offline=True)

    pre_paths = {to_posix(e["path"]) for e in get_status_files(repo)}
    assert pre_paths == {"a.txt", "b.txt", "c.txt"}, (
        f"Expected three pending paths before reset, got {pre_paths}"
    )

    # --reset alone drops the tracked state; status reports nothing
    entries = get_status_files(repo, reset=True)
    assert entries == [], f"--reset should yield empty status, got {entries}"
    assert not has_staged_anchor(repo), "Anchor should be cleared after --reset"

    # Revert prior on-disk modifications so the tree matches the committed revision
    for name in ("a.txt", "b.txt", "c.txt"):
        with repo.open_file(name, "w+") as f:
            f.write(f"{name} original\n")

    # Modify and stage a single file on the cleared tracked state
    with repo.open_file("d.txt", "w+") as f:
        f.write("d modified content longer\n")
    repo.stage("d.txt", offline=True)

    # --reset drops the stage; --scan re-detects d.txt as a dirty modification
    entries = get_status_files(repo, reset=True, scan=True)
    paths = {to_posix(e["path"]) for e in entries}
    assert paths == {"d.txt"}, (
        f"After --reset --scan only d.txt should appear as unstaged modified, got {paths}"
    )
    entry = find_status_entry(entries, "d.txt")
    assert entry["flagDirty"] is True, "d.txt should be dirty after rescan"
    assert entry["flagStaged"] is False, "d.txt should be unstaged after --reset"


@pytest.mark.smoke
def test_branch_switch_rebases_staged_anchor(new_lore_repo):
    """Branch switch leaves a clean status reflecting the new branch's content."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("alice content here\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    repo.branch_create("other", offline=True)
    with repo.open_file("file.txt", "w+") as f:
        f.write("bob content here\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    repo.branch_switch("main", offline=True)
    assert not has_staged_anchor(repo), (
        "Staged anchor should be cleared after branch switch with no dirty work"
    )
    entries = get_status_files_twice(repo, scan=True)
    file_entry = find_status_entry(entries, "file.txt")
    assert file_entry is None, (
        f"file.txt should not appear in status after branch switch, got: {file_entry}"
    )
    with repo.open_file("file.txt", "r") as f:
        assert f.read() == "alice content here\n"


@pytest.mark.smoke
def test_sync_rebases_staged_anchor(new_lore_repo):
    """Sync to a previous revision updates the staged anchor so status reflects the new current."""
    repo: Lore = new_lore_repo()

    with repo.open_file("file.txt", "w+") as f:
        f.write("v1 content\n")
    repo.stage(offline=True)
    repo.commit(offline=True)
    rev_v1 = repo.revision_history(offline=True)[0].signature

    with repo.open_file("file.txt", "w+") as f:
        f.write("v2 content longer\n")
    repo.stage(offline=True)
    repo.commit(offline=True)

    # Sync back to v1 and scan; file should match v1 with no false modifications.
    repo.sync(rev_v1, offline=True)
    entries = get_status_files_twice(repo, scan=True)
    entry = find_status_entry(entries, "file.txt")
    assert entry is None, (
        f"file.txt should not appear in status after sync, got: {entry}"
    )
    with repo.open_file("file.txt", "r") as f:
        assert f.read() == "v1 content\n"


@pytest.mark.smoke
def test_branch_switch_rebases_only_dirty_files(new_lore_repo):
    """Switching branches carries only dirty paths into the new staged state.

    The repo has files in multiple paths committed on main. A feature branch
    commits unrelated modify/add/delete changes. While on feature, separate
    files are dirtied (modify, add, delete). Switching back to main rebases
    the staged anchor: only the dirtied paths remain dirty, and the rest of
    the tree matches main's revision.
    """
    repo: Lore = new_lore_repo()

    repo.write_files(
        {
            "app/main.py": "main entrypoint\n",
            "app/utils/helper.py": "helper original\n",
            "docs/readme.md": "readme original\n",
            "data/sample.txt": "sample data\n",
            "data/config.json": "{}\n",
        }
    )
    repo.stage(offline=True)
    repo.commit(offline=True)

    repo.branch_create("feature", offline=True)

    repo.write_files(
        {
            "app/main.py": "modified on feature\n",
            "app/new_feature.py": "new feature code\n",
        }
    )
    os.remove(repo._fix_path("docs/readme.md"))
    repo.stage(offline=True)
    repo.commit(offline=True)

    repo.write_files(
        {
            "data/sample.txt": "dirty modified sample\n",
            "data/extra.txt": "dirty new file\n",
        }
    )
    os.remove(repo._fix_path("app/utils/helper.py"))
    repo.dirty(
        ["data/sample.txt", "data/extra.txt", "app/utils/helper.py"], offline=True
    )

    pre_entries = get_status_files_twice(repo)
    pre_dirty = {to_posix(e["path"]) for e in pre_entries if e.get("flagDirty")}
    assert pre_dirty == {"data/sample.txt", "data/extra.txt", "app/utils/helper.py"}, (
        f"Dirty paths before switch should match expectations, got {pre_dirty}"
    )

    repo.branch_switch("main", offline=True)

    entries = get_status_files_twice(repo)
    dirty_paths = {to_posix(e["path"]) for e in entries if e.get("flagDirty")}
    expected_dirty = {"data/sample.txt", "data/extra.txt", "app/utils/helper.py"}
    assert dirty_paths == expected_dirty, (
        f"After switch, only dirtied paths should be dirty. "
        f"Expected {expected_dirty}, got {dirty_paths}"
    )

    with repo.open_file("app/main.py", "r") as f:
        assert f.read() == "main entrypoint\n"
    assert not os.path.exists(repo._fix_path("app/new_feature.py")), (
        "Feature-only file should not exist on main"
    )
    with repo.open_file("docs/readme.md", "r") as f:
        assert f.read() == "readme original\n"
    with repo.open_file("data/config.json", "r") as f:
        assert f.read() == "{}\n"

    with repo.open_file("data/sample.txt", "r") as f:
        assert f.read() == "dirty modified sample\n"
    with repo.open_file("data/extra.txt", "r") as f:
        assert f.read() == "dirty new file\n"
    assert not os.path.exists(repo._fix_path("app/utils/helper.py")), (
        "Locally deleted file should remain absent after switch"
    )
