# SPDX-FileCopyrightText: 2026 Epic Games, Inc.
# SPDX-License-Identifier: MIT
import logging
import re

import pytest

from error_types import InvalidRepositoryPath
from lore import Lore

logger = logging.getLogger(__name__)


def get_url(repository_info_output: str) -> str | None:
    match = re.search(".*Remote URL: (.*)", repository_info_output)
    if match is not None:
        return match.group(1)
    return None


@pytest.mark.smoke
def test_repository_info_url(new_lore_repo, tmp_path_factory, monkeypatch):
    no_repo_urc: Lore = new_lore_repo(create_repo=False)

    repo = new_lore_repo()

    monkeypatch.chdir(repo.path)

    assert get_url(no_repo_urc.repository_info(use_os_dir=True)) + "/" == repo.remote

    assert get_url(no_repo_urc.repository_info(path=repo.path)) + "/" == repo.remote
    with pytest.raises(InvalidRepositoryPath):
        assert no_repo_urc.repository_info()
    assert (
        get_url(no_repo_urc.repository_info(url=repo.remote_path)) + "/" == repo.remote
    )
