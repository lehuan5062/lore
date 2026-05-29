# SPDX-FileCopyrightText: 2026 Epic Games, Inc.
# SPDX-License-Identifier: MIT
import logging
import os

import pytest

from lore import Lore

logger = logging.getLogger(__name__)


@pytest.mark.smoke
def test_info(new_lore_repo):
    repo: Lore = new_lore_repo("Info")

    # Generate some files
    text_file = "path/to/file.txt"
    another_file = "another/path.txt"

    repo.make_dirs(os.path.dirname(text_file))
    with repo.open_file(text_file, "w+b") as output_file:
        output_file.write(os.urandom(1000))

    repo.make_dirs(os.path.dirname(another_file))
    with repo.open_file(another_file, "w+") as output_file:
        output_file.writelines(["One line\n", "Another line\n", "Third line\n"])

    # Stage the files
    repo.stage([text_file, another_file], offline=True)
    repo.commit(offline=True)

    # Describe
    files = repo.file_info([text_file, another_file], offline=True)

    file_paths = [file.path for file in files]

    assert text_file in file_paths, "Missing file in info output"
    assert another_file in file_paths, "Missing file in info output"

    with repo.open_file(text_file, "w+b") as output_file:
        output_file.write(os.urandom(1000))

    repo.remove_file(another_file)

    # Describe
    files = repo.file_info([text_file, another_file], offline=True, local=True)

    assert len(files) == 2, "Unexpected number of files in output"

    file = [file for file in files if text_file in file.path][0]
    assert file.status == "Modified", "Missing file status in info output"

    file = [file for file in files if another_file in file.path][0]
    assert file.status == "Deleted", "Missing file status in info output"
