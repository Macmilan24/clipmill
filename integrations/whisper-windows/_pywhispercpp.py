"""Provide the usual upstream module name for the selected native binding."""

import sys
from importlib import import_module

from pywhispercpp._clipmill_cpu import selected_variant

sys.modules[__name__] = import_module(f"pywhispercpp._{selected_variant()}._pywhispercpp")
