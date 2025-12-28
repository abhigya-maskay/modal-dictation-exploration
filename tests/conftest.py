"""Pytest configuration and fixtures."""

import sys
from unittest.mock import MagicMock

import pytest

# Ensure pystray mock exists before any imports
if "pystray" not in sys.modules or not isinstance(sys.modules["pystray"], MagicMock):
    sys.modules["pystray"] = MagicMock()


@pytest.fixture
def pystray_mock():
    """Provide a fresh pystray mock for tests that need it."""
    mock = sys.modules["pystray"]
    mock.reset_mock()
    return mock
