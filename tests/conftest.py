"""Pytest configuration and fixtures."""

import sys

import pytest

# Imported for side effects: registers mocks in sys.modules before other imports
from tests.mocks import pystray as _pystray  # noqa: F401


@pytest.fixture
def pystray_mock():
    """Provide a fresh pystray mock for tests that need it."""
    mock = sys.modules["pystray"]
    mock.reset_mock()
    mock.Icon.side_effect = None
    return mock
