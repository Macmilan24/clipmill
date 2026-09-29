"""llama.cpp's server, packed as a wheel of the editorial component.

Windows and Linux run the editorial model in llama.cpp's own server (bom.toml
[llama_cpp]). A wheel, so the daemon installs it with the component it belongs
to, offline and by digest, as it installs ClipMill's own wheels, and the
component's digest changes when the build does. stage.py calls this with the
pinned archive; it needs nothing beyond the standard library.
"""

from __future__ import annotations

import base64
import hashlib
import tarfile
import zipfile
from pathlib import Path

PACKAGE = "clipmill_llama_server"
# The Linux server finds its libraries beside itself ($ORIGIN) by these names.
# The archive holds them as symbolic links, which a wheel cannot carry, so each
# is packed under the name the loader asks for. The CPU and Vulkan backends are
# found by name when the server starts (libggml-*.so).
LINUX_LIBRARIES = (
    "libllama-server-impl.so",
    "libllama-common.so.0",
    "libllama.so.0",
    "libmtmd.so.0",
    "libggml.so.0",
    "libggml-base.so.0",
    "libggml-vulkan.so",
)
# Run inside the component's environment: the server loads every library it
# needs just to say its version.
VERSION_CHECK = (
    "import subprocess, clipmill_llama_server; "
    "subprocess.run([str(clipmill_llama_server.executable()), '--version'], check=True)"
)
INIT = '''"""llama.cpp's server ({build}), as ClipMill pins it in bom.toml.

Packed from the upstream release archive by tools/release/stage.py. The
editorial worker starts it for GGUF models (server_runtime.py).
"""

from pathlib import Path

BUILD = "{build}"


def executable() -> Path:
    return Path(__file__).parent / "bin" / "{server}"
'''


def server_files(archive: Path, *, windows: bool) -> dict[str, bytes]:
    """The server and what it loads, each under the name it is loaded by."""
    if windows:
        with zipfile.ZipFile(archive) as bundle:
            return {
                name: bundle.read(name)
                for name in bundle.namelist()
                if name == "llama-server.exe" or name.endswith(".dll") or name.startswith("LICENSE")
            }
    files = {}
    with tarfile.open(archive) as bundle:
        members = {Path(member.name).name: member for member in bundle.getmembers()}
        backends = sorted(name for name in members if name.startswith("libggml-cpu-"))
        for name in ["llama-server", *LINUX_LIBRARIES, *backends, "LICENSE"]:
            member = members.get(name)
            while member is not None and member.issym():
                member = members.get(Path(member.linkname).name)
            if member is None or not member.isfile():
                raise SystemExit(f"stage: the llama.cpp archive has no {name}")
            source = bundle.extractfile(member)
            if source is None:
                raise SystemExit(f"stage: cannot read {name} from the llama.cpp archive")
            files[name] = source.read()
    return files


def build_wheel(archive: Path, build: str, out: Path, *, windows: bool) -> Path:
    """The wheel, reproducibly: fixed times, one order, and the modes the files need."""
    files = server_files(archive, windows=windows)
    server = "llama-server.exe" if windows else "llama-server"
    # glibc 2.34 is the newest symbol version the Linux build asks for.
    tag = "win_amd64" if windows else "manylinux_2_34_x86_64"
    version = build.removeprefix("b")
    dist = f"{PACKAGE}-{version}.dist-info"
    entries = [
        (f"{PACKAGE}/__init__.py", INIT.format(build=build, server=server).encode()),
        *((f"{PACKAGE}/bin/{name}", data) for name, data in sorted(files.items())),
        (
            f"{dist}/METADATA",
            (
                "Metadata-Version: 2.1\nName: clipmill-llama-server\n"
                f"Version: {version}\nSummary: llama.cpp's server ({build}), as ClipMill pins it\n"
                "License: MIT\n"
            ).encode(),
        ),
        (
            f"{dist}/WHEEL",
            (
                "Wheel-Version: 1.0\nGenerator: clipmill-stage\nRoot-Is-Purelib: false\n"
                f"Tag: py3-none-{tag}\n"
            ).encode(),
        ),
    ]
    record = "".join(f"{path},sha256={record_digest(data)},{len(data)}\n" for path, data in entries)
    entries.append((f"{dist}/RECORD", f"{record}{dist}/RECORD,,\n".encode()))
    wheel = out / f"{PACKAGE}-{version}-py3-none-{tag}.whl"
    with zipfile.ZipFile(wheel, "w") as bundle:
        for path, data in entries:
            info = zipfile.ZipInfo(path, date_time=(1980, 1, 1, 0, 0, 0))
            name = Path(path).name
            runnable = name == server or name.endswith((".so", ".dll")) or ".so." in name
            info.external_attr = (0o100755 if runnable else 0o100644) << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            bundle.writestr(info, data)
    return wheel


def record_digest(data: bytes) -> str:
    """A file's digest as a wheel's RECORD states it."""
    return base64.urlsafe_b64encode(hashlib.sha256(data).digest()).rstrip(b"=").decode()


def notice(build: str, url: str) -> str:
    return (
        f"llama.cpp {build}\n\n"
        "The editorial component carries llama.cpp's server\n"
        "(https://github.com/ggml-org/llama.cpp), which runs the editorial model on\n"
        "this platform. It is offered under the MIT licence, llama.cpp-LICENSE.txt,\n"
        f"and ClipMill carries it unchanged from {url}.\n"
        f"Its source is at https://github.com/ggml-org/llama.cpp/tree/{build}.\n"
    )
