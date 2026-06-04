import shutil
import subprocess

import pytest

from etl_lib._utils.git_revision import (
    get_git_revision_hash,
    get_git_revision_short_hash,
)


def _in_git_repo() -> bool:
    if shutil.which("git") is None:
        return False
    result = subprocess.run(
        ["git", "rev-parse", "--is-inside-work-tree"],
        capture_output=True,
        text=True,
    )
    return result.returncode == 0


@pytest.mark.skipif(not _in_git_repo(), reason="not inside a git repository")
def test_get_git_revision_hash_returns_40_char_sha():
    sha = get_git_revision_hash()
    assert isinstance(sha, str)
    assert len(sha) == 40
    assert all(c in "0123456789abcdef" for c in sha)


@pytest.mark.skipif(not _in_git_repo(), reason="not inside a git repository")
def test_get_git_revision_short_hash_is_prefix_of_full():
    short = get_git_revision_short_hash()
    full = get_git_revision_hash()
    assert isinstance(short, str)
    assert 7 <= len(short) <= 12
    assert full.startswith(short)
