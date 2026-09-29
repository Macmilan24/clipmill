"""llama.cpp's server must pack into a wheel the daemon can install and run."""

from __future__ import annotations

import hashlib
import importlib.util
import io
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "llama", Path(__file__).resolve().parents[1] / "llama.py"
)
assert SPEC is not None and SPEC.loader is not None
llama = importlib.util.module_from_spec(SPEC)
sys.modules["llama"] = llama
SPEC.loader.exec_module(llama)


def windows_archive(folder: Path) -> Path:
    archive = folder / "llama-b7-bin-win-vulkan-x64.zip"
    with zipfile.ZipFile(archive, "w") as bundle:
        for name in ["llama-server.exe", "llama-cli.exe", "llama.dll", "ggml-vulkan.dll"]:
            bundle.writestr(name, f"{name} bytes")
        bundle.writestr("LICENSE-LLVM-OpenMP", "LLVM licence")
    return archive


def linux_archive(folder: Path) -> Path:
    """The upstream layout: one folder, sonames as symbolic links."""
    archive = folder / "llama-b7-bin-ubuntu-vulkan-x64.tar.gz"
    real = {
        "llama-server": b"server",
        "llama-cli": b"a tool the component does not need",
        "libllama-server-impl.so": b"impl",
        "libllama-common.so.0.5.0": b"common",
        "libllama.so.0.5.0": b"llama",
        "libmtmd.so.0.5.0": b"mtmd",
        "libggml.so.0.25.3": b"ggml",
        "libggml-base.so.0.25.3": b"base",
        "libggml-vulkan.so": b"vulkan",
        "libggml-cpu-haswell.so": b"haswell",
        "libggml-cpu-x64.so": b"x64",
        "LICENSE": b"MIT",
    }
    links = {
        "libllama-common.so.0": "libllama-common.so.0.5.0",
        "libllama.so.0": "libllama.so.0.5.0",
        "libllama.so": "libllama.so.0",
        "libmtmd.so.0": "libmtmd.so.0.5.0",
        "libggml.so.0": "libggml.so.0.25.3",
        "libggml-base.so.0": "libggml-base.so.0.25.3",
    }
    with tarfile.open(archive, "w:gz") as bundle:
        for name, data in real.items():
            info = tarfile.TarInfo(f"llama-b7/{name}")
            info.size = len(data)
            bundle.addfile(info, io.BytesIO(data))
        for name, target in links.items():
            info = tarfile.TarInfo(f"llama-b7/{name}")
            info.type = tarfile.SYMTYPE
            info.linkname = target
            bundle.addfile(info)
    return archive


def entries(wheel: Path) -> dict[str, tuple[bytes, int]]:
    with zipfile.ZipFile(wheel) as bundle:
        return {
            info.filename: (bundle.read(info), info.external_attr >> 16)
            for info in bundle.infolist()
        }


class LlamaWheel(unittest.TestCase):
    def test_windows_packs_the_server_its_libraries_and_licences_and_nothing_else(self):
        with tempfile.TemporaryDirectory() as scratch:
            folder = Path(scratch)
            wheel = llama.build_wheel(windows_archive(folder), "b7", folder, windows=True)
            self.assertEqual(wheel.name, "clipmill_llama_server-7-py3-none-win_amd64.whl")
            files = entries(wheel)
            packed = sorted(name.rsplit("/", 1)[1] for name in files if "/bin/" in name)
            self.assertEqual(
                packed, ["LICENSE-LLVM-OpenMP", "ggml-vulkan.dll", "llama-server.exe", "llama.dll"]
            )
            init = files["clipmill_llama_server/__init__.py"][0].decode()
            self.assertIn('BUILD = "b7"', init)
            self.assertIn('"llama-server.exe"', init)

    def test_linux_packs_each_library_under_the_name_the_loader_asks_for(self):
        with tempfile.TemporaryDirectory() as scratch:
            folder = Path(scratch)
            wheel = llama.build_wheel(linux_archive(folder), "b7", folder, windows=False)
            self.assertEqual(
                wheel.name, "clipmill_llama_server-7-py3-none-manylinux_2_34_x86_64.whl"
            )
            files = entries(wheel)
            bin_files = {
                name.rsplit("/", 1)[1]: value for name, value in files.items() if "/bin/" in name
            }
            self.assertEqual(bin_files["libllama.so.0"][0], b"llama")
            self.assertEqual(bin_files["libggml-base.so.0"][0], b"base")
            self.assertEqual(
                sorted(bin_files),
                sorted(
                    [
                        "llama-server",
                        *llama.LINUX_LIBRARIES,
                        "libggml-cpu-haswell.so",
                        "libggml-cpu-x64.so",
                        "LICENSE",
                    ]
                ),
            )
            self.assertEqual(bin_files["llama-server"][1] & 0o777, 0o755)
            self.assertEqual(bin_files["libllama.so.0"][1] & 0o777, 0o755)
            self.assertEqual(bin_files["LICENSE"][1] & 0o777, 0o644)

    def test_the_record_states_every_file_and_the_wheel_is_reproducible(self):
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            archive = linux_archive(Path(first))
            one = llama.build_wheel(archive, "b7", Path(first), windows=False)
            two = llama.build_wheel(archive, "b7", Path(second), windows=False)
            self.assertEqual(
                hashlib.sha256(one.read_bytes()).digest(), hashlib.sha256(two.read_bytes()).digest()
            )
            files = entries(one)
            record = files["clipmill_llama_server-7.dist-info/RECORD"][0].decode().splitlines()
            stated = {}
            for line in record:
                path, digest, size = line.split(",")
                stated[path] = (digest, size)
            self.assertEqual(set(stated), set(files))
            for path, (data, _mode) in files.items():
                if path.endswith("RECORD"):
                    continue
                self.assertEqual(
                    stated[path], (f"sha256={llama.record_digest(data)}", str(len(data)))
                )

    def test_an_archive_missing_a_library_stops_the_release(self):
        with tempfile.TemporaryDirectory() as scratch:
            folder = Path(scratch)
            archive = folder / "broken.tar.gz"
            with tarfile.open(archive, "w:gz") as bundle:
                info = tarfile.TarInfo("llama-b7/llama-server")
                info.size = 1
                bundle.addfile(info, io.BytesIO(b"s"))
            with self.assertRaises(SystemExit):
                llama.build_wheel(archive, "b7", folder, windows=False)


if __name__ == "__main__":
    unittest.main()
