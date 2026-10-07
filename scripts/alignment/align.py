"""Compatibility entry; implementation lives in the fixed Chef framework."""
from pathlib import Path
import runpy
import sys
tool = Path(__file__).resolve().parents[2] / "framework/scripts/alignment/align.py"
sys.path.insert(0, str(tool.parent))
globals().update(runpy.run_path(str(tool), run_name="__main__" if __name__ == "__main__" else "chef_alignment"))
