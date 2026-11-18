"""Build the Windows binding from the pinned upstream sources with MSVC."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import urllib.request
from pathlib import Path

from hatchling.builders.hooks.plugin.interface import BuildHookInterface

VARIANTS = {
    "x64": set(),
    "avx": {"SSE42", "AVX"},
    "avx2": {"SSE42", "AVX", "AVX2", "FMA", "F16C", "BMI2"},
}


def source_archive(pin: dict, root: Path) -> Path:
    cached = os.environ.get("CLIPMILL_WHISPER_SOURCE")
    archive = Path(cached) if cached else root / "source.tar.gz"
    if not cached:
        with urllib.request.urlopen(pin["url"], timeout=120) as response, archive.open("wb") as out:
            shutil.copyfileobj(response, out)
    with archive.open("rb") as handle:
        actual = hashlib.file_digest(handle, "sha256").hexdigest()
    if actual != pin["sha256"]:
        raise ValueError(f"Whisper source digest {actual} does not match {pin['sha256']}")
    with tarfile.open(archive, "r:gz") as files:
        files.extractall(root, filter="data")
    return root / "pywhispercpp-1.5.0"


def build_variant(source: Path, build: Path, enabled: set[str]) -> tuple[Path, str]:
    # GGML_NATIVE=OFF still enables AVX2 by default in this upstream release.
    # Set every eligible ggml instruction flag independently of the build host.
    features = sorted(VARIANTS["avx2"] | {"AVX_VNNI", "AVX512"})
    flags = [f"-DGGML_{feature}={'ON' if feature in enabled else 'OFF'}" for feature in features]
    output = build / "extension"
    env = dict(os.environ)
    for key in (
        "CFLAGS",
        "CXXFLAGS",
        "LDFLAGS",
        "CL",
        "_CL_",
        "LINK",
        "CC",
        "CXX",
        "CMAKE_GENERATOR",
        "CMAKE_GENERATOR_PLATFORM",
        "CMAKE_TOOLCHAIN_FILE",
    ):
        env.pop(key, None)
    env["CL"] = f'/Brepro "/pathmap:{source.parent}=whisper-build"'
    subprocess.run(
        [
            "cmake",
            "-S",
            str(source),
            "-B",
            str(build),
            "-A",
            "x64",
            # VS 2026 also installs the VS 2022 toolset. Its 14.3x/14.4x
            # runtime is covered by the 14.51 redistributable in the installer.
            "-T",
            "v143",
            # The top-level upstream project still declares CMake 3.4.
            "-DCMAKE_POLICY_VERSION_MINIMUM=3.5",
            "-DBUILD_SHARED_LIBS=OFF",
            "-DGGML_NATIVE=OFF",
            "-DGGML_OPENMP=OFF",
            "-DGGML_LLAMAFILE=OFF",
            "-DGGML_BACKEND_DL=OFF",
            "-DGGML_CPU_ALL_VARIANTS=OFF",
            "-DCMAKE_INTERPROCEDURAL_OPTIMIZATION=OFF",
            "-DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL",
            "-DCMAKE_MODULE_LINKER_FLAGS=/Brepro",
            "-DWHISPER_BUILD_TESTS=OFF",
            "-DWHISPER_BUILD_EXAMPLES=OFF",
            "-DWHISPER_BUILD_SERVER=OFF",
            "-DPYBIND11_FINDPYTHON=ON",
            f"-DPython_EXECUTABLE={sys.executable}",
            f"-DCMAKE_LIBRARY_OUTPUT_DIRECTORY_RELEASE={output}",
            f"-DCMAKE_RUNTIME_OUTPUT_DIRECTORY_RELEASE={output}",
            *flags,
        ],
        check=True,
        env=env,
    )
    (config,) = (build / "CMakeFiles").glob("*/CMakeCXXCompiler.cmake")
    compiler = re.search(r'set\(CMAKE_CXX_COMPILER_VERSION "([^"]+)"\)', config.read_text())
    if compiler is None or not re.fullmatch(r"19\.(?:3[0-9]|4[0-4])(?:\.\d+)+", compiler[1]):
        raise RuntimeError("Windows Whisper needs MSVC v143 (19.30 through 19.44).")
    print(f"Building Whisper {build.name} with MSVC {compiler[1]} (v143, /MD)", flush=True)
    subprocess.run(
        [
            "cmake",
            "--build",
            str(build),
            "--config",
            "Release",
            "--target",
            "_pywhispercpp",
            "--parallel",
            str(min(os.cpu_count() or 1, 8)),
        ],
        check=True,
        env=env,
    )
    (extension,) = output.glob("_pywhispercpp*.pyd")
    return extension, compiler[1]


class WindowsWhisperHook(BuildHookInterface):
    def initialize(self, version, build_data):
        if sys.platform != "win32" or platform.machine().lower() not in {"amd64", "x86_64"}:
            raise RuntimeError("Build the Windows Whisper wheel on Windows x64 with MSVC.")
        project = Path(self.root)
        pin = tomllib.loads((project / "pyproject.toml").read_text())["tool"]["clipmill"][
            "upstream"
        ]
        self.scratch = tempfile.TemporaryDirectory(prefix="cm-whisper-")
        root = Path(self.scratch.name)
        source = source_archive(pin, root)
        files = build_data["force_include"]
        files[str(source / "pywhispercpp")] = "pywhispercpp"
        files[str(project / "_pywhispercpp.py")] = "_pywhispercpp.py"
        files[str(project / "_clipmill_cpu.py")] = "pywhispercpp/_clipmill_cpu.py"
        for name, license_path in (
            ("pywhispercpp", source / "LICENSE"),
            ("pybind11", source / "pybind11/LICENSE"),
            ("whisper.cpp", project / "whisper.cpp-LICENSE.txt"),
        ):
            files[str(license_path)] = f"pywhispercpp/licenses/{name}-LICENSE.txt"
        for variant, enabled in VARIANTS.items():
            extension, compiler = build_variant(source, root / variant, enabled)
            files[str(extension)] = f"pywhispercpp/_{variant}/{extension.name}"
        provenance = root / "build.json"
        provenance.write_text(
            json.dumps(
                {
                    "source": pin,
                    "msvc": compiler,
                    "python": platform.python_version(),
                    "cmake": subprocess.check_output(
                        ["cmake", "--version"], text=True
                    ).splitlines()[0],
                    "ggml_features": {name: sorted(flags) for name, flags in VARIANTS.items()},
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        files[str(provenance)] = "pywhispercpp/_clipmill_build.json"
        build_data["pure_python"] = False
        build_data["infer_tag"] = True

    def finalize(self, version, build_data, artifact_path):
        self.scratch.cleanup()
