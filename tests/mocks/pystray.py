"""Mock pystray module for testing."""

import sys
from unittest.mock import MagicMock

# Create and register the mock before any imports
mock = MagicMock()

if "pystray" not in sys.modules or not isinstance(sys.modules["pystray"], MagicMock):
    sys.modules["pystray"] = mock
