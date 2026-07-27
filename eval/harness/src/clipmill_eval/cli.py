"""Command-line entry points for reproducible local evaluation."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

from .attestation import (
    build_phase0_attestation,
    load_private_signing_key,
    verify_phase0_attestation,
    write_phase0_attestation,
)
from .client import DaemonClient
from .corpus import verify_corpus
from .mlx import build_mlx_attestation, verify_mlx_attestation, write_mlx_attestation
from .runner import run_corpus, write_run_manifest
from .smoke import build_smoke_corpus


def main(arguments: list[str] | None = None) -> int:
    parser = _parser()
    options = parser.parse_args(arguments)
    try:
        if options.command == "verify-corpus":
            corpus = verify_corpus(
                options.corpus_dir,
                options.manifest,
                options.license_attestation,
                _public_key(options.public_key),
            )
            print(f"verified {corpus.corpus_id}: {len(corpus.items)} items")
            return 0
        if options.command == "run":
            corpus = verify_corpus(
                options.corpus_dir,
                options.manifest,
                options.license_attestation,
                _public_key(options.public_key),
            )
            manifest = run_corpus(
                DaemonClient(options.socket),
                options.data_dir,
                corpus,
            )
            write_run_manifest(options.output, manifest)
            print(f"wrote run manifest sha256:{_sha256(options.output)}")
            return 0
        if options.command == "seed40":
            corpus = verify_corpus(
                options.corpus_dir,
                options.manifest,
                options.license_attestation,
                _public_key(options.public_key),
            )
            run_manifest = run_corpus(
                DaemonClient(options.socket),
                options.data_dir,
                corpus,
            )
            bundle = build_phase0_attestation(
                corpus,
                run_manifest,
                load_private_signing_key(options.signing_key),
            )
            write_phase0_attestation(options.output_dir, bundle)
            verify_phase0_attestation(options.output_dir)
            digest = _sha256(options.output_dir / "run-attestation.json")
            print(f"Seed-40 passed; wrote Phase 0 attestation sha256:{digest}")
            return 0
        if options.command == "verify-attestation":
            bundle = verify_phase0_attestation(options.attestation_dir)
            print(
                "verified Phase 0 attestation: "
                f"{bundle.corpus_metadata['items_total']} Seed-40 items"
            )
            return 0
        if options.command == "attest-mlx":
            bundle = build_mlx_attestation(
                json.loads(options.profile.read_text(encoding="utf-8")),
                json.loads(options.timing.read_text(encoding="utf-8")),
                load_private_signing_key(options.signing_key),
            )
            write_mlx_attestation(options.output_dir, bundle)
            verify_mlx_attestation(options.output_dir)
            digest = _sha256(options.output_dir / "mlx-attestation.json")
            print(f"MLX selection attested; wrote sha256:{digest}")
            return 0
        if options.command == "verify-mlx-attestation":
            bundle = verify_mlx_attestation(options.attestation_dir)
            bound = ", ".join(
                f"{binding['capability']}={binding['model']}"
                for binding in bundle.attestation["bindings"]
            )
            print(f"verified MLX selection attestation: {bound}")
            return 0
        if options.command == "smoke":
            root, manifest_path, license_path = build_smoke_corpus(
                options.work_dir,
                options.ffmpeg,
            )
            corpus = verify_corpus(root, manifest_path, license_path)
            manifest = run_corpus(
                DaemonClient(options.socket),
                options.data_dir,
                corpus,
            )
            write_run_manifest(options.output, manifest)
            print(f"smoke run passed; manifest sha256:{_sha256(options.output)}")
            return 0
    except (OSError, RuntimeError, ValueError) as error:
        print(f"clipmill-eval: {error}", file=sys.stderr)
        return 1
    parser.error("unknown command")
    return 2


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="clipmill-eval")
    subcommands = parser.add_subparsers(dest="command", required=True)

    verify = subcommands.add_parser("verify-corpus")
    _corpus_arguments(verify)

    run = subcommands.add_parser("run")
    _corpus_arguments(run)
    _daemon_arguments(run)

    seed40 = subcommands.add_parser("seed40")
    _corpus_arguments(seed40)
    seed40.add_argument("--socket", type=Path, required=True)
    seed40.add_argument("--data-dir", type=Path, required=True)
    seed40.add_argument("--signing-key", type=Path, required=True)
    seed40.add_argument("--output-dir", type=Path, required=True)

    verify_attestation = subcommands.add_parser("verify-attestation")
    verify_attestation.add_argument("--attestation-dir", type=Path, required=True)

    attest_mlx = subcommands.add_parser("attest-mlx")
    attest_mlx.add_argument("--profile", type=Path, required=True)
    attest_mlx.add_argument("--timing", type=Path, required=True)
    attest_mlx.add_argument("--signing-key", type=Path, required=True)
    attest_mlx.add_argument("--output-dir", type=Path, required=True)

    verify_mlx = subcommands.add_parser("verify-mlx-attestation")
    verify_mlx.add_argument("--attestation-dir", type=Path, required=True)

    smoke = subcommands.add_parser("smoke")
    _daemon_arguments(smoke)
    smoke.add_argument("--ffmpeg", type=Path, required=True)
    smoke.add_argument("--work-dir", type=Path, required=True)
    return parser


def _corpus_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--corpus-dir", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--license-attestation", type=Path, required=True)
    parser.add_argument("--public-key", type=Path)


def _daemon_arguments(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--socket", type=Path, required=True)
    parser.add_argument("--data-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)


def _public_key(path: Path | None) -> bytes | None:
    if path is None:
        return None
    encoded = path.read_text(encoding="utf-8").strip()
    try:
        public_key = bytes.fromhex(encoded)
    except ValueError as error:
        raise ValueError("public key file is not hexadecimal") from error
    if len(public_key) != 32:
        raise ValueError("public key must contain exactly 32 bytes")
    return public_key


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
