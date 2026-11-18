"""Transcribe recorded speech with ClipMill's installed Windows x64 Whisper worker.

Run on a modern x64 Windows host with clang and lld-link. Supply the first-run
worker's Scripts/python.exe, its wheel from the app's engine/wheels directory,
the registered whisper-base model, and these pinned upstream test inputs:

    https://raw.githubusercontent.com/ggml-org/whisper.cpp/2eeeba56e9edd762b4b38467bab96c2517163158/samples/jfk.wav
    https://www.intel.com/content/www/us/en/download/684897/intel-software-development-emulator.html

The Windows Intel SDE 10.13.1 archive SHA-256 is
74e626ede09b0baa5011fc9e51b58627ea92c3fc0bae5fd7db34b490f335f651.
Keep the extracted kit together. Intel documents chip checking in
doc/chip_check.html in that kit. The test itself does not fetch dependencies.
"""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import importlib.metadata
import json
import math
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import tomllib
import wave
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve()
MODEL_SPEC = HERE.parents[2] / "models" / "registry" / "whisper-base.toml"
AUDIO_SHA256 = "59dfb9a4acb36fe2a2affc14bacbee2920ff435cb13cc314a08c13f66ba7860e"
SDE_EXE_SHA256 = "59bededb5c0a0fa73cd94f36f8382efa9b69b78ecb43df9ce151d030d5b420c0"
REFERENCE_TEXT = (
    "and so my fellow americans ask not what your country can do for you "
    "ask what you can do for your country"
)
REFERENCE = REFERENCE_TEXT.split()
CONTROLS = {"avx": "vzeroupper", "avx2": "vpaddd", "fma": "vfmadd132ps", "bmi2": "pext"}
# SDE flags and signatures come from the pinned kit's misc/cpuid/<chip>/cpuid.def.
# Each CPU must select the corresponding compiled extension.
EMULATED = {"ivybridge": ("-ivb", "000306a9", "avx"), "nehalem": ("-nhm", "000106a5", "x64")}
WHEEL_VERSION = "1.5.0+clipmill.1"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def word_edits(transcript: str) -> int:
    """Minimum insertions, deletions and substitutions from the spoken reference."""
    heard = re.findall(r"[a-z]+", transcript.lower())
    previous = list(range(len(heard) + 1))
    for i, word in enumerate(REFERENCE, 1):
        row = [i]
        for j, actual in enumerate(heard, 1):
            row.append(min(previous[j] + 1, row[-1] + 1, previous[j - 1] + (word != actual)))
        previous = row
    return previous[-1]


def installed_payload(wheel: Path, site: Path) -> dict[str, str]:
    """Match the dispatcher and native code in the worker to the installed app's wheel."""
    site = site.resolve(strict=True)
    require(
        wheel.name == f"pywhispercpp-{WHEEL_VERSION}-cp312-cp312-win_amd64.whl",
        f"unexpected staged wheel: {wheel.name}",
    )
    members: dict[str, str] = {}
    with zipfile.ZipFile(wheel) as archive:
        for name in archive.namelist():
            if not name.endswith((".py", ".dll", ".pyd")):
                continue
            installed = (site / name).resolve()
            require(installed.is_relative_to(site), f"invalid wheel path: {name}")
            expected = hashlib.sha256(archive.read(name)).hexdigest()
            require(digest(installed) == expected, f"installed bytes differ: {name}")
            members[name] = expected
    require(
        any(name.endswith("/_clipmill_cpu.py") for name in members),
        "wheel has no Windows CPU dispatcher",
    )
    require(any(name.endswith(".pyd") for name in members), "wheel has no native Whisper extension")
    return members


def build_probe(directory: Path) -> Path:
    def llvm(tool: str) -> str:
        found = shutil.which(tool)
        if found:
            return found
        standard = Path(os.environ["PROGRAMFILES"]) / "LLVM" / "bin" / (tool + ".exe")
        require(standard.is_file(), f"missing {tool}; install the Windows LLVM tools")
        return str(standard)

    obj = directory / "whisper_isa.obj"
    dll = directory / "whisper_isa.dll"
    subprocess.run(
        [
            llvm("clang"),
            "--target=x86_64-pc-windows-msvc",
            "-c",
            HERE.with_name("whisper_windows_isa.S"),
            "-o",
            obj,
        ],
        check=True,
    )
    exports = ["read_cpu", *(f"require_{name}" for name in CONTROLS)]
    subprocess.run(
        [
            llvm("lld-link"),
            "/dll",
            "/noentry",
            "/nodefaultlib",
            "/machine:x64",
            f"/out:{dll}",
            *(f"/export:{entry}" for entry in exports),
            obj,
        ],
        check=True,
    )
    return dll


def probe_cpu(lib: ctypes.CDLL) -> dict:
    words = (ctypes.c_uint32 * 8)()
    lib.read_cpu.argtypes = [ctypes.POINTER(ctypes.c_uint32)]
    lib.read_cpu.restype = None
    lib.read_cpu(words)
    signature, ecx, ebx, edx, extended_ecx = words[3:]
    cpu = {
        "vendor": struct.pack("<III", *words[:3]).decode("ascii"),
        "signature": f"{signature:08x}",
        "sse": bool(edx & (1 << 25)),
        "sse2": bool(edx & (1 << 26)),
        "sse3": bool(ecx & 1),
        "ssse3": bool(ecx & (1 << 9)),
        "sse41": bool(ecx & (1 << 19)),
        "sse42": bool(ecx & (1 << 20)),
        "popcnt": bool(ecx & (1 << 23)),
        "cx16": bool(ecx & (1 << 13)),
        "lahf": bool(extended_ecx & 1),
        "avx": bool(ecx & (1 << 28)),
        "f16c": bool(ecx & (1 << 29)),
        "fma": bool(ecx & (1 << 12)),
        "bmi1": bool(ebx & (1 << 3)),
        "avx2": bool(ebx & (1 << 5)),
        "bmi2": bool(ebx & (1 << 8)),
    }
    v2_features = ("sse", "sse2", "sse3", "ssse3", "sse41", "sse42", "popcnt", "cx16", "lahf")
    cpu["x86_v2"] = all(cpu[name] for name in v2_features)
    return cpu


def infer(profile: str, dll: Path, model: Path, audio: Path) -> dict:
    def offline(event: str, args: tuple) -> None:
        if event in {"socket.connect", "socket.getaddrinfo", "socket.gethostbyname"}:
            raise RuntimeError(f"offline inference attempted {event}")

    sys.addaudithook(offline)
    lib = ctypes.CDLL(str(dll))
    cpu = probe_cpu(lib)
    expected = "avx2"
    if profile in EMULATED:
        _, signature, expected = EMULATED[profile]
        has_avx = profile == "ivybridge"
        require(
            cpu["vendor"] == "GenuineIntel"
            and cpu["signature"] == signature
            and cpu["x86_v2"]
            and cpu["avx"] == has_avx
            and cpu["f16c"] == has_avx
            and not any(cpu[name] for name in ("avx2", "bmi1", "bmi2", "fma")),
            f"worker did not observe SDE's {profile} profile: {cpu}",
        )
    else:
        require(
            all(cpu[name] for name in CONTROLS), f"native control needs a modern x64 CPU: {cpu}"
        )
        for name in CONTROLS:
            getattr(lib, f"require_{name}")()

    from clipmill_worker_asr_whispercpp.engine import WhisperCppRecognizer, decode_pcm16
    from clipmill_worker_sdk.weights import VerifiedModel

    version = importlib.metadata.version("pywhispercpp")
    require(version == WHEEL_VERSION, f"unexpected installed version: {version}")
    weight = tomllib.loads(MODEL_SPEC.read_text(encoding="utf-8"))["files"][0]
    binding = VerifiedModel("whisper-base", "asr", weight["sha256"], model.parent, (model.name,))
    recognizer = WhisperCppRecognizer(binding, weights=model.name, language="en")
    # Let the worker perform its regular admission and native initialization first.
    from pywhispercpp._clipmill_cpu import selected_variant
    from pywhispercpp.model import pw

    variant = selected_variant()
    require(
        variant == expected and pw.__name__ == f"pywhispercpp._{expected}._pywhispercpp",
        f"{profile} selected {variant} and loaded {pw.__name__}",
    )
    with wave.open(str(audio), "rb") as stream:
        require(stream.getparams()[:4] == (1, 2, 16000, 176000), "unexpected recorded audio format")
        samples = decode_pcm16(stream.readframes(stream.getnframes()))
    result = recognizer.decode(samples)
    edits = word_edits(result.text)
    require(edits <= 4, f"{profile} lost the spoken reference ({edits} word edits): {result.text}")
    require(
        len(result.tokens) >= 12
        and all(math.isfinite(t.probability) and 0 <= t.probability <= 1 for t in result.tokens),
        "invalid Whisper token evidence",
    )
    return {
        "cpu": cpu,
        "variant": variant,
        "extension": pw.__file__,
        "worker": sys.modules[WhisperCppRecognizer.__module__].__file__,
        "version": version,
        "transcript": result.text,
        "word_edits": edits,
        "reference_words": len(REFERENCE),
        "tokens": len(result.tokens),
        "venv": sys.prefix,
        "python": sys.version,
    }


def child(argv: list[str]) -> None:
    if argv[0] == "_control":
        lib = ctypes.CDLL(argv[1])
        getattr(lib, f"require_{argv[2]}")()
        print("ISA_CANARY_RETURNED", flush=True)
    else:
        result = infer(argv[1], *(Path(arg) for arg in argv[2:]))
        print("WHISPER_RESULT " + json.dumps(result), flush=True)


def run(options: argparse.Namespace, report: dict) -> None:
    require(sys.platform == "win32" and struct.calcsize("P") == 8, "requires x64 Windows")
    # Preserve the venv launcher path if this installation used a symlink to Python.
    python = options.python.absolute()
    wheel, audio, model, sde = (
        p.resolve(strict=True) for p in (options.wheel, options.audio, options.model, options.sde)
    )
    require(
        python.parent.name.lower() == "scripts" and python.name.lower() == "python.exe",
        f"expected the installed worker's Scripts/python.exe: {python}",
    )
    venv = python.parent.parent.resolve(strict=True)
    site = (venv / "Lib" / "site-packages").resolve(strict=True)
    members = installed_payload(wheel, site)
    weight = tomllib.loads(MODEL_SPEC.read_text(encoding="utf-8"))["files"][0]
    require(model.name == weight["path"] and digest(model) == weight["sha256"], "wrong model")
    require(digest(audio) == AUDIO_SHA256, "wrong recording")
    require(digest(sde) == SDE_EXE_SHA256, "wrong Intel SDE launcher")
    report.update(
        wheel=wheel.name,
        wheel_sha256=digest(wheel),
        model_sha256=weight["sha256"],
        audio_sha256=AUDIO_SHA256,
        sde_exe_sha256=SDE_EXE_SHA256,
        windows_build=sys.getwindowsversion().build,
        emulator="Intel SDE 10.13.1 on native Windows, including loaded packages",
        emulated_profiles={name: row[0] for name, row in EMULATED.items()},
        profiles={},
        rejected_instructions={},
    )

    # A Windows venv launcher can spawn a base Python process outside instrumentation.
    # Start that interpreter directly with the same venv marker CPython's launcher sets.
    # The child verifies both its actual CPUID and its installed module's location.
    base = Path(
        subprocess.check_output(
            [python, "-I", "-c", "import sys; print(sys._base_executable)"], text=True
        ).strip()
    ).resolve(strict=True)
    environment = dict(os.environ, __PYVENV_LAUNCHER__=str(python))
    with tempfile.TemporaryDirectory(prefix="whisper-isa-") as work:
        dll = build_probe(Path(work))

        def launch(profile: str, args: list[str]) -> subprocess.CompletedProcess[str]:
            command = (
                [str(python)]
                if profile == "native"
                else [str(sde), EMULATED[profile][0], "--", str(base)]
            )
            return subprocess.run(
                [*command, "-I", str(HERE), *args],
                cwd=work,
                env=os.environ if profile == "native" else environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                encoding="utf-8",
                errors="replace",
                timeout=options.timeout,
            )

        for name, instruction in CONTROLS.items():
            profile = "nehalem" if name == "avx" else "ivybridge"
            check = launch(profile, ["_control", str(dll), name])
            output = check.stdout.lower()
            require(
                "executed instruction not valid for specified chip" in output
                and instruction in output
                and "whisper_isa.dll" in output
                and "isa_canary_returned" not in output,
                f"{profile} did not trap {instruction} in the loaded DLL: {check.stdout[-2400:]}",
            )
            report["rejected_instructions"].setdefault(profile, {})[name] = instruction

        for profile in ("native", *EMULATED):
            print(f"Transcribing the pinned recording on {profile}", flush=True)
            check = launch(profile, ["_infer", profile, str(dll), str(model), str(audio)])
            require(
                check.returncode == 0 and "SDE-ERROR" not in check.stdout,
                f"{profile} exited {check.returncode}: {check.stdout[-3500:]}",
            )
            lines = [
                s.removeprefix("WHISPER_RESULT ")
                for s in check.stdout.splitlines()
                if s.startswith("WHISPER_RESULT ")
            ]
            require(
                len(lines) == 1, f"{profile} missing completed inference: {check.stdout[-1500:]}"
            )
            result = json.loads(lines[0])
            require(Path(result["venv"]).resolve() == venv, f"{profile} used another Python venv")
            result["worker"] = Path(result["worker"]).resolve().relative_to(site).as_posix()
            extension = Path(result["extension"]).resolve().relative_to(site).as_posix()
            require(
                extension in members, f"{profile} imported an extension outside the staged wheel"
            )
            result.update(extension=extension, extension_sha256=members[extension])
            report["profiles"][profile] = result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    for name in ("python", "wheel", "model", "audio", "sde", "report"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    parser.add_argument(
        "--timeout", type=int, default=1800, help="seconds per instrumented process"
    )
    options = parser.parse_args()
    report: dict = {"ok": False}
    try:
        run(options, report)
        report["ok"] = True
    except (OSError, ValueError, subprocess.SubprocessError, zipfile.BadZipFile) as error:
        report["error"] = str(error)
    options.report.parent.mkdir(parents=True, exist_ok=True)
    output = json.dumps(report, indent=2)
    options.report.write_text(output + "\n", encoding="utf-8")
    print(output)
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    if sys.argv[1:2] in (["_infer"], ["_control"]):
        child(sys.argv[1:])
    else:
        sys.exit(main())
