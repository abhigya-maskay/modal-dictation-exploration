"""Mock pulsectl module for testing."""

import sys
from unittest.mock import MagicMock

# Import real classes before mocking
from pulsectl import PulseStateEnum
from pulsectl.pulsectl import EnumValue

# Create and register the mock before any imports
mock = MagicMock()
mock.PulseStateEnum = PulseStateEnum

# Create pulsectl.pulsectl submodule mock with the real EnumValue
pulsectl_submodule = MagicMock()
pulsectl_submodule.EnumValue = EnumValue

if "pulsectl" not in sys.modules or not isinstance(sys.modules["pulsectl"], MagicMock):
    sys.modules["pulsectl"] = mock
    sys.modules["pulsectl.pulsectl"] = pulsectl_submodule