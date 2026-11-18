"""Start the CLI from a built installer and perform the MCP stdio handshake."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def verify(directory: Path) -> None:
    root = Path(__file__).resolve().parents[2]
    version = json.loads((root / "apps/desktop/src-tauri/tauri.conf.json").read_text())["version"]
    command = directory / ("clipmill.exe" if os.name == "nt" else "clipmill")
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "installer-check", "version": "1"},
        },
    }
    # An installer check must never connect to the build host's personal engine.
    with tempfile.TemporaryDirectory() as state:
        result = subprocess.run(
            [command.resolve(), "mcp"],
            input=json.dumps(request) + "\n",
            env={**os.environ, "CLIPMILL_DATA_DIR": state},
            text=True,
            capture_output=True,
            timeout=15,
            check=True,
        )
    reply = json.loads(result.stdout)
    info = reply["result"]["serverInfo"]
    if reply["id"] != 1 or info["name"] != "clipmill" or info["version"] != version:
        raise SystemExit(f"Expected the installed ClipMill {version} MCP server, got {info!r}")
    print(f"Installed CLI started the ClipMill {version} MCP server")


if __name__ == "__main__":
    verify(Path(sys.argv[1]))
