#!/usr/bin/env python3
"""Stage everything a packaged ClipMill carries, for one target.

Writes two folders the release Tauri config reads (both ignored by Git):

    apps/desktop/src-tauri/binaries/   the pinned executables beside the app:
        clipmilld, ffmpeg, ffprobe and uv, each named <name>-<target triple>
    apps/desktop/src-tauri/resources/  read-only files the app reads:
        fonts/, emoji/, models/registry/, engine/, licenses/

Every download is checked against its pin in bom.toml before it is used, and
cached under .cache/release/ by digest. The engine folder holds ClipMill's own
worker wheels, each worker's third-party requirements exported from its
uv.lock with every line pinned by hash, and engine.json, which the daemon reads
(crates/clipmilld/src/engine/manifest.rs).

    uv run --with packaging==25.0 python tools/release/stage.py [--target TRIPLE]
        [--skip-daemon] [--verify-engine]

--verify-engine installs every component with the staged uv exactly as the
daemon would, into a scratch folder, and imports each worker: a component
that cannot install or start on this platform fails the release here rather
than on someone's computer.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform as host
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path

from packaging.requirements import Requirement
from packaging.tags import Tag, compatible_tags, cpython_tags, mac_platforms
from packaging.utils import canonicalize_name, parse_wheel_filename

ROOT = Path(__file__).resolve().parents[2]
TAURI = ROOT / "apps" / "desktop" / "src-tauri"
BINARIES = TAURI / "binaries"
RESOURCES = TAURI / "resources"
CACHE = ROOT / ".cache" / "release"
ENGINE_SCHEMA = "clipmill.engine.v1"
# What a download of the pinned Python costs, roughly; counted once.
PYTHON_DOWNLOAD_BYTES = 34 * 1024 * 1024


@dataclass(frozen=True)
class Target:
    triple: str
    # Its name in bom.toml, and in the daemon's engine manifest.
    bom: str
    engine: str
    exe: str
    # What `packaging` needs to choose the wheel uv would.
    platforms: tuple[str, ...]
    environment: dict[str, str]


def manylinux(arch: str) -> tuple[str, ...]:
    return (
        *(f"manylinux_2_{minor}_{arch}" for minor in range(39, 16, -1)),
        f"manylinux2014_{arch}",
        f"manylinux2010_{arch}",
        f"manylinux1_{arch}",
        f"linux_{arch}",
    )


def python_environment(system: str, machine: str, os_name: str) -> dict[str, str]:
    return {
        "implementation_name": "cpython",
        "implementation_version": "3.12.13",
        "os_name": os_name,
        "platform_machine": machine,
        "platform_python_implementation": "CPython",
        "platform_release": "",
        "platform_system": system,
        "platform_version": "",
        "python_full_version": "3.12.13",
        "python_version": "3.12",
        "sys_platform": {"Darwin": "darwin", "Linux": "linux", "Windows": "win32"}[system],
    }


TARGETS = {
    "aarch64-apple-darwin": Target(
        "aarch64-apple-darwin",
        "macos-arm64",
        "macos-arm64",
        "",
        tuple(mac_platforms((14, 0), "arm64")),
        python_environment("Darwin", "arm64", "posix"),
    ),
    "x86_64-unknown-linux-gnu": Target(
        "x86_64-unknown-linux-gnu",
        "linux-amd64",
        "linux-x86_64",
        "",
        manylinux("x86_64"),
        python_environment("Linux", "x86_64", "posix"),
    ),
    "x86_64-pc-windows-msvc": Target(
        "x86_64-pc-windows-msvc",
        "windows-amd64",
        "windows-x86_64",
        ".exe",
        ("win_amd64",),
        python_environment("Windows", "AMD64", "nt"),
    ),
}


def host_triple() -> str:
    system, machine = host.system(), host.machine().lower()
    if system == "Darwin" and machine == "arm64":
        return "aarch64-apple-darwin"
    if system == "Linux" and machine in {"x86_64", "amd64"}:
        return "x86_64-unknown-linux-gnu"
    if system == "Windows" and machine in {"amd64", "x86_64"}:
        return "x86_64-pc-windows-msvc"
    raise SystemExit(f"stage: no release target for {system}-{machine}")


def say(message: str) -> None:
    print(f"stage: {message}", flush=True)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fetch(url: str, want: str) -> Path:
    """A pinned download, cached by its digest and checked before use."""
    if not url.startswith("https://"):
        raise SystemExit(f"stage: refusing a download that is not HTTPS: {url}")
    cached = CACHE / "downloads" / want
    if cached.is_file() and sha256(cached) == want:
        return cached
    cached.parent.mkdir(parents=True, exist_ok=True)
    partial = cached.with_suffix(".part")
    say(f"downloading {url}")
    # Some hosts refuse Python's default user agent; say who is asking.
    request = urllib.request.Request(
        url, headers={"User-Agent": "clipmill-release (+https://github.com/Macmilan24/clipmill)"}
    )
    with urllib.request.urlopen(request, timeout=120) as response, partial.open("wb") as out:
        shutil.copyfileobj(response, out)
    got = sha256(partial)
    if got != want:
        partial.unlink()
        raise SystemExit(f"stage: sha256 mismatch for {url}\n  want {want}\n  got  {got}")
    partial.replace(cached)
    return cached


def extract(archive: Path, url: str, member: str, destination: Path) -> None:
    """Take one pinned member out of an archive, and nothing else."""
    destination.parent.mkdir(parents=True, exist_ok=True)
    if url.endswith(".zip"):
        with (
            zipfile.ZipFile(archive) as bundle,
            bundle.open(member) as source,
            destination.open("wb") as out,
        ):
            shutil.copyfileobj(source, out)
    else:
        with tarfile.open(archive) as bundle:
            source = bundle.extractfile(member)
            if source is None:
                raise SystemExit(f"stage: {member} is not a file in {url}")
            with destination.open("wb") as out:
                shutil.copyfileobj(source, out)
    destination.chmod(0o755)


def sidecar(target: Target, name: str) -> Path:
    return BINARIES / f"{name}-{target.triple}{target.exe}"


def run(command: list[str], **options: object) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, check=True, text=True, **options)  # type: ignore[call-overload]


# ---- executables ----------------------------------------------------------


def stage_daemon(target: Target, skip: bool) -> None:
    built = ROOT / "target" / target.triple / "release" / f"clipmilld{target.exe}"
    if not skip:
        say("building clipmilld (release)")
        run(
            [
                "cargo",
                "build",
                "--release",
                "--locked",
                "-p",
                "clipmilld",
                "--bin",
                "clipmilld",
                "--target",
                target.triple,
            ],
            cwd=ROOT,
        )
    if not built.is_file():
        raise SystemExit(f"stage: {built} was not built")
    shutil.copy2(built, sidecar(target, "clipmilld"))


def stage_ffmpeg(target: Target, bom: dict) -> None:
    entry = bom["ffmpeg"][target.bom]
    if entry.get("redistributable") is not True:
        raise SystemExit(f"stage: the pinned FFmpeg for {target.bom} may not be redistributed")
    if "archive_url" in entry:
        archive = fetch(entry["archive_url"], entry["archive_sha256"])
        for tool in ("ffmpeg", "ffprobe"):
            extract(archive, entry["archive_url"], entry[f"{tool}_member"], sidecar(target, tool))
    else:
        for tool in ("ffmpeg", "ffprobe"):
            url = entry[f"{tool}_url"]
            archive = fetch(url, entry[f"{tool}_sha256"])
            extract(archive, url, f"{tool}{target.exe}", sidecar(target, tool))


def stage_uv(target: Target, bom: dict) -> None:
    entry = bom["uv"][target.bom]
    archive = fetch(entry["url"], entry["sha256"])
    extract(archive, entry["url"], entry["member"], sidecar(target, "uv"))


# ---- resources ------------------------------------------------------------


def stage_fonts(bom: dict) -> None:
    fonts = RESOURCES / "fonts"
    fonts.mkdir(parents=True, exist_ok=True)
    captions = bom["fonts"]["captions"]
    archive = fetch(captions["archive_url"], captions["archive_sha256"])
    family, style = captions["family"], captions["style"]
    for member, name, want in (
        (captions["member"], f"{family}-{style}.ttf", captions["member_sha256"]),
        (captions["license_member"], f"{family}-LICENSE.txt", captions["license_sha256"]),
    ):
        with zipfile.ZipFile(archive) as bundle:
            (fonts / name).write_bytes(bundle.read(member))
        if sha256(fonts / name) != want:
            raise SystemExit(f"stage: {member} does not match its pin")
    for face in [value for value in bom["fonts"]["faces"]["ids"].split(",") if value]:
        pin = bom["fonts"]["face"][face]
        shutil.copy2(fetch(pin["url"], pin["sha256"]), fonts / pin["file"])
        shutil.copy2(fetch(pin["license_url"], pin["license_sha256"]), fonts / pin["license_file"])


def stage_emoji(bom: dict) -> None:
    emoji = bom["emoji"]
    folder = RESOURCES / "emoji"
    folder.mkdir(parents=True, exist_ok=True)
    for code in [value for value in emoji["ids"].split(",") if value]:
        picture = fetch(f"{emoji['base_url']}/emoji_u{code}.png", emoji["sha256"][code])
        shutil.copy2(picture, folder / f"emoji_u{code}.png")
    shutil.copy2(fetch(emoji["license_url"], emoji["license_sha256"]), folder / "LICENSE.txt")
    shutil.copy2(fetch(emoji["notice_url"], emoji["notice_sha256"]), folder / "NOTICE-README.md")


def stage_registry() -> None:
    registry = RESOURCES / "models" / "registry"
    registry.mkdir(parents=True, exist_ok=True)
    for manifest in sorted((ROOT / "models" / "registry").glob("*.toml")):
        shutil.copy2(manifest, registry / manifest.name)


# ---- engine ---------------------------------------------------------------


def supported_tags(target: Target) -> list[Tag]:
    tags = list(cpython_tags((3, 12), platforms=list(target.platforms)))
    tags += list(compatible_tags((3, 12), "cp312", platforms=list(target.platforms)))
    return tags


def wheel_downloads(target: Target, lock: Path, requirements: Path) -> dict[str, int]:
    """What installing these requirements downloads here, wheel by wheel: for
    every line that applies to the target, the size uv.lock records for the
    wheel uv would choose, keyed by name and version."""
    ranking = {tag: index for index, tag in enumerate(supported_tags(target))}
    sizes: dict[str, list[tuple[int, int]]] = {}
    for package in tomllib.loads(lock.read_text(encoding="utf-8")).get("package", []):
        choices = []
        for wheel in package.get("wheels", []):
            filename = wheel["url"].rsplit("/", 1)[-1]
            try:
                _, _, _, tags = parse_wheel_filename(filename)
            except ValueError:
                continue
            best = min((ranking[tag] for tag in tags if tag in ranking), default=None)
            if best is not None and "size" in wheel:
                choices.append((best, int(wheel["size"])))
        if choices:
            sizes[canonicalize_name(package["name"])] = choices
    downloads = {}
    for line in requirements.read_text(encoding="utf-8").splitlines():
        line = line.split("\\")[0].strip()
        if not line or line.startswith(("#", "-")):
            continue
        requirement = Requirement(line)
        if requirement.marker is not None and not requirement.marker.evaluate(target.environment):
            continue
        choices = sizes.get(canonicalize_name(requirement.name))
        if choices:
            key = f"{canonicalize_name(requirement.name)}{requirement.specifier}"
            downloads[key] = min(choices)[1]
    return downloads


def project_dir(part: dict) -> Path:
    """Where a component's Python project is: workers/<name> unless named."""
    return ROOT / part.get("project", f"workers/{part['name']}")


def uses_sdk(project: dict) -> bool:
    dependencies = project["project"].get("dependencies", [])
    names = {canonicalize_name(Requirement(dependency).name) for dependency in dependencies}
    return "clipmill-worker-sdk" in names


def build_wheel(project: Path, out: Path) -> Path:
    before = set(out.glob("*.whl"))
    run(["uv", "build", "--wheel", "--out-dir", str(out), str(project)], cwd=ROOT)
    made = sorted(set(out.glob("*.whl")) - before)
    if len(made) != 1:
        raise SystemExit(f"stage: building {project} made {len(made)} wheels")
    return made[0]


def stage_engine(target: Target, bom: dict) -> dict:
    engine = RESOURCES / "engine"
    wheels = engine / "wheels"
    requirements = engine / "requirements"
    wheels.mkdir(parents=True, exist_ok=True)
    requirements.mkdir(parents=True, exist_ok=True)
    table = tomllib.loads((ROOT / "tools" / "release" / "engine.toml").read_text(encoding="utf-8"))
    say("building the worker SDK wheel")
    sdk = build_wheel(ROOT / "workers" / "sdk", wheels)
    parts = []
    # uv downloads a wheel once and links it into every environment that
    # needs it, so each is counted against the first component that does:
    # the estimates then add up to what setting up everything downloads.
    counted: dict[str, set[str]] = {other.engine: set() for other in TARGETS.values()}
    for part in table["part"]:
        name = part["name"]
        directory = project_dir(part)
        project = tomllib.loads((directory / "pyproject.toml").read_text(encoding="utf-8"))
        sdk_needed = uses_sdk(project)
        scripts = project["project"].get("scripts", {})
        if scripts.get(part["command"], "").split(":", 1)[0] != part["module"]:
            raise SystemExit(
                f"stage: engine.toml names {part['command']} -> {part['module']}, "
                f"which {directory.relative_to(ROOT)}/pyproject.toml does not define"
            )
        say(f"building {name}")
        wheel = build_wheel(directory, wheels)
        exported = requirements / f"{name}.txt"
        run(
            [
                "uv",
                "export",
                "--frozen",
                "--no-dev",
                "--no-emit-project",
                # ClipMill's SDK arrives as a wheel from the app, never an index.
                *(["--no-emit-package", "clipmill-worker-sdk"] if sdk_needed else []),
                "--format",
                "requirements-txt",
                "--no-header",
                "--project",
                str(directory),
                "-o",
                str(exported),
            ],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
        )
        own = [{"file": f"wheels/{sdk.name}", "sha256": sha256(sdk)}] if sdk_needed else []
        estimates = {}
        for other in TARGETS.values():
            if other.engine not in part["platforms"]:
                continue
            seen = counted[other.engine]
            downloads = wheel_downloads(other, directory / "uv.lock", exported)
            fresh = {key: size for key, size in downloads.items() if key not in seen}
            # The first component also brings Python itself.
            python = PYTHON_DOWNLOAD_BYTES if not seen else 0
            seen.update(downloads)
            seen.add("python")
            estimates[other.engine] = sum(fresh.values()) + python
        parts.append(
            {
                "name": name,
                "kind": part.get("kind", "worker"),
                "title": part["title"],
                "family": part["family"],
                "command": part["command"],
                "module": part["module"],
                "platforms": part["platforms"],
                "requirements": f"requirements/{name}.txt",
                "requirements_sha256": sha256(exported),
                "wheels": [*own, {"file": f"wheels/{wheel.name}", "sha256": sha256(wheel)}],
                "download_bytes": estimates,
            }
        )
    manifest = {"schema_version": ENGINE_SCHEMA, "python": bom["python"]["version"], "parts": parts}
    (engine / "engine.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def verify_engine(target: Target, manifest: dict) -> None:
    """Install every component here exactly as the daemon does, and start it."""
    uv = sidecar(target, "uv")
    engine = RESOURCES / "engine"
    with tempfile.TemporaryDirectory(prefix="clipmill-engine-") as scratch:
        root = Path(scratch)
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.upper().startswith(("PYTHON", "UV_", "PIP_", "VIRTUAL_ENV", "CONDA_"))
        }
        environment |= {
            "UV_PYTHON_INSTALL_DIR": str(root / "python"),
            "UV_CACHE_DIR": str(root / "cache"),
            "UV_NO_CONFIG": "1",
            "UV_PYTHON_PREFERENCE": "only-managed",
            "UV_PYTHON_DOWNLOADS": "automatic",
            "PYTHONNOUSERSITE": "1",
        }
        quiet = ["--no-progress", "--color", "never"]
        python = manifest["python"]
        run([str(uv), "python", "install", python, *quiet], env=environment)
        for part in manifest["parts"]:
            if target.engine not in part["platforms"]:
                continue
            say(f"verifying {part['name']}")
            env_dir = root / "envs" / part["name"]
            run(
                [str(uv), "venv", str(env_dir), "--python", python, "--no-project", *quiet],
                env=environment,
            )
            interpreter = env_dir / ("Scripts/python.exe" if target.exe else "bin/python")
            run(
                [
                    str(uv),
                    "pip",
                    "install",
                    "--python",
                    str(interpreter),
                    "--require-hashes",
                    "--only-binary",
                    ":all:",
                    "-r",
                    str(engine / part["requirements"]),
                    *quiet,
                ],
                env=environment,
            )
            run(
                [
                    str(uv),
                    "pip",
                    "install",
                    "--python",
                    str(interpreter),
                    "--no-deps",
                    "--no-index",
                    "--offline",
                    *[str(engine / wheel["file"]) for wheel in part["wheels"]],
                    *quiet,
                ],
                env=environment,
            )
            run([str(interpreter), "-I", "-c", f"import {part['module']}"], env=environment)
            scripts = env_dir / ("Scripts" if target.exe else "bin")
            command = scripts / f"{part['command']}{target.exe}"
            if not command.is_file():
                raise SystemExit(f"stage: {part['name']} installed no {command.name}")


# ---- licences ---------------------------------------------------------------


def stage_licenses(bom: dict, target: Target) -> None:
    licenses = RESOURCES / "licenses"
    licenses.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / "LICENSE", licenses / "ClipMill-LICENSE.txt")
    for notice in (ROOT / "apps" / "desktop" / "public" / "licenses").glob("*.txt"):
        shutil.copy2(notice, licenses / notice.name)
    ffmpeg = bom["ffmpeg"][target.bom]
    version = ffmpeg.get("version", bom["ffmpeg"]["version"])
    source = ffmpeg.get("archive_url") or ffmpeg.get("ffmpeg_url")
    (licenses / "FFmpeg-NOTICE.txt").write_text(
        f"ClipMill ships FFmpeg {version} (build {ffmpeg['build']}) as separate programs,\n"
        "ffmpeg and ffprobe, which it runs as subprocesses. FFmpeg is licensed under the\n"
        "GNU General Public License, version 3 (this build enables GPL components).\n\n"
        f"The build was taken from {source}\n"
        f"and its provider is {ffmpeg.get('provider', bom['ffmpeg']['provider'])}.\n"
        f"FFmpeg's source code: https://ffmpeg.org/releases/ffmpeg-{version}.tar.xz\n"
        "and https://git.ffmpeg.org/ffmpeg.git. The GPL text is ClipMill-LICENSE.txt's\n"
        "companion at https://www.gnu.org/licenses/gpl-3.0.txt.\n",
        encoding="utf-8",
    )
    (licenses / "uv-NOTICE.txt").write_text(
        f"ClipMill ships uv {bom['uv']['version']} (https://github.com/astral-sh/uv), dual\n"
        "licensed under MIT or Apache-2.0, to install its components' Python packages.\n",
        encoding="utf-8",
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--target", default=None, help="Rust target triple (default: this machine)")
    parser.add_argument("--skip-daemon", action="store_true", help="use an already built daemon")
    parser.add_argument("--verify-engine", action="store_true", help="install every component once")
    options = parser.parse_args()
    target = TARGETS.get(options.target or host_triple())
    if target is None:
        raise SystemExit(f"stage: unknown target {options.target}")
    bom = tomllib.loads((ROOT / "bom.toml").read_text(encoding="utf-8"))

    for folder in (BINARIES, RESOURCES):
        shutil.rmtree(folder, ignore_errors=True)
        folder.mkdir(parents=True)
    stage_daemon(target, options.skip_daemon)
    stage_ffmpeg(target, bom)
    stage_uv(target, bom)
    stage_fonts(bom)
    stage_emoji(bom)
    stage_registry()
    manifest = stage_engine(target, bom)
    stage_licenses(bom, target)
    if options.verify_engine:
        verify_engine(target, manifest)
    here = [part["name"] for part in manifest["parts"] if target.engine in part["platforms"]]
    say(f"staged {target.triple}: {len(here)} components ({', '.join(here)})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
