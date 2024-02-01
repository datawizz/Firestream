"""Tests for `pynessie` package."""
import pytest

from pynessie import init
from pynessie.error import NessieConflictException
from pynessie.model import Branch, Entries
import os

ENDPOINT_URI = os.environ.get("NESSIE_SERVER_URI")

@pytest.mark.vcr
def test_client_interface_e2e() -> None:
    """Test client object against live server."""
    client = init(config_dict={"auth_type": "none", "verify": False, "endpoint": ENDPOINT_URI })
    print(client.get_base_url())
    assert isinstance(client.get_base_url(), str)
    assert client.get_base_url() == ENDPOINT_URI
    references = client.list_references().references
    print(references)
    assert len(references) == 1
    assert references[0] == Branch("main", references[0].hash_)
    main_name = references[0].name
    print(main_name)
    main_commit = references[0].hash_
    print(main_commit)
    with pytest.raises(NessieConflictException):
        client.create_branch("main")
    created_reference = client.create_branch("test", main_name, main_commit)
    print(created_reference)
    references = client.list_references().references
    print(references)
    assert len(references) == 2
    assert next(i for i in references if i.name == "main") == Branch("main", main_commit)
    assert next(i for i in references if i.name == "test") == Branch("test", main_commit)
    reference = client.get_reference("test")
    print(reference)
    assert created_reference == reference
    assert isinstance(reference.hash_, str)
    tables = client.list_keys(reference.name, reference.hash_)
    print(tables)
    assert isinstance(tables, Entries)
    assert len(tables.entries) == 0
    assert isinstance(main_commit, str)
    client.delete_branch("test", main_commit)
    references = client.list_references().references
    print(references)
    assert len(references) == 1

if __name__ == "__main__":
    test_client_interface_e2e()

