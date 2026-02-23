import importlib
import sys

mod = importlib.import_module("function")
input_data = sys.stdin.read()
try:
    result = mod.run(input_data)
    print(result)
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
