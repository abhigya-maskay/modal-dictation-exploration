from modal_dictation import greet


def test_greet_returns_expected_message() -> None:
    assert greet("Modal") == "Hello, Modal."
