from etl_lib.secrets import Secrets


def test_secrets():
    """
    Confirm access to the Secrets server
    """
    known_good_data = {"test": 1}

    secret_client = Secrets()
    creds = secret_client.get_secret("test")
    assert creds == 1
    assert known_good_data.get("test") == creds


def test_alpaca():
    secret_client = Secrets()
    creds = secret_client.get_secret("alpaca-paper")
    print(creds)


if __name__ == "__main__":
    test_alpaca()
    test_secrets()
