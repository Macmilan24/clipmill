"""Start the CLI from a built installer and perform the MCP stdio handshake."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def verify(command: Path, environment: dict[str, str]) -> None:
    root = Path(__file__).resolve().parents[2]
    version = json.loads((root / "apps/desktop/src-tauri/tauri.conf.json").read_text())["version"]
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
            env={**environment, "CLIPMILL_DATA_DIR": state},
            text=True,
            capture_output=True,
            timeout=60,
            check=True,
        )
    reply = json.loads(result.stdout)
    info = reply["result"]["serverInfo"]
    if reply["id"] != 1 or info["name"] != "clipmill" or info["version"] != version:
        raise SystemExit(f"Expected the installed ClipMill {version} MCP server, got {info!r}")
    print(f"Installed CLI started the ClipMill {version} MCP server")


def verify_appimage(artifact: Path) -> None:
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary).resolve()
        image = root / "ClipMill portable studio.AppImage"
        shutil.copy2(artifact, image)
        for launch in ("first", "second"):
            scratch = root / launch
            scratch.mkdir()
            # The real AppImage runtime's no-FUSE mode uses a fresh AppDir.
            environment = {
                **os.environ,
                "APPIMAGE_EXTRACT_AND_RUN": "1",
                "TMPDIR": str(scratch),
                "CLIPMILL_DATA_DIR": str(root / "state"),
            }
            result = subprocess.run(
                [image, "--print-cli-path"],
                env=environment,
                text=True,
                capture_output=True,
                timeout=60,
                check=True,
            )
            configured = Path(result.stdout.strip())
            if configured != image:
                raise SystemExit(f"Portable Settings command changed to {configured}")
            verify(configured, environment)


if __name__ == "__main__":
    if sys.argv[1] == "--appimage":
        verify_appimage(Path(sys.argv[2]))
    else:
        executable = "clipmill.exe" if os.name == "nt" else "clipmill"
        verify(Path(sys.argv[1]) / executable, dict(os.environ))
