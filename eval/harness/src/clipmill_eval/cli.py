"""Command-line entry points for reproducible local evaluation."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

from .annotation import load_annotation, load_annotations
from .attestation import (
    build_phase0_attestation,
    load_private_signing_key,
    verify_phase0_attestation,
    write_phase0_attestation,
)
from .client import DaemonClient
from .corpus import verify_corpus
from .fetch import (
    build_documents,
    fetch_item,
    load_spec,
    refuse_tracked_destination,
    write_documents,
)
from .mlx import build_mlx_attestation, verify_mlx_attestation, write_mlx_attestation
from .runner import (
    analyze_and_read,
    run_corpus,
    verify_candidates,
    verify_ranking,
    write_run_manifest,
)
from .scoring import clips_from_ranking, meets_bar, score_run
from .smoke import build_smoke_corpus
from .worksheet import sentences_from_index, write_worksheet


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
        if options.command == "fetch-corpus":
            corpus_id, items = load_spec(options.spec)
            refuse_tracked_destination(options.corpus_dir, options.repository)
            fetched = [
                (item, fetch_item(item, options.corpus_dir, options.downloader.split()))
                for item in items
            ]
            manifest, attestation = build_documents(corpus_id, fetched, options.corpus_dir)
            manifest_path, attestation_path = write_documents(
                options.output_dir, manifest, attestation
            )
            print(
                f"fetched {len(fetched)} items for {corpus_id}; "
                f"sign {manifest_path.name} and {attestation_path.name} before running"
            )
            return 0
        if options.command == "annotate":
            corpus = verify_corpus(
                options.corpus_dir,
                options.manifest,
                options.license_attestation,
                _public_key(options.public_key),
            )
            client = DaemonClient(options.socket)
            project = client.create_project(f"annotate-{corpus.corpus_id}")
            written = 0
            for item in corpus.items:
                if item.expected_result != "success":
                    continue
                source = client.register_source(project.project_id, corpus.path_for(item)).source
                documents = analyze_and_read(
                    client,
                    project.project_id,
                    source.source_id,
                    timeout_seconds=options.timeout_seconds,
                )
                index = documents.get("index.transcript.v1")
                if index is None:
                    raise RuntimeError(f"{item.item_id} produced no evidence index to annotate")
                write_worksheet(
                    options.output_dir,
                    item.item_id,
                    str(index["source_fingerprint"]),
                    options.annotator,
                    0,
                    sentences_from_index(index),
                )
                written += 1
            print(f"wrote {written} worksheets to {options.output_dir}")
            return 0
        if options.command == "recall":
            corpus = verify_corpus(
                options.corpus_dir,
                options.manifest,
                options.license_attestation,
                _public_key(options.public_key),
            )
            annotations = load_annotations(options.annotations)
            by_item = {annotation.item_id: annotation for annotation in annotations}
            client = DaemonClient(options.socket)
            project = client.create_project(f"recall-{corpus.corpus_id}")
            pairs = []
            for item in corpus.items:
                annotation = by_item.get(item.item_id)
                if annotation is None:
                    continue
                source = client.register_source(project.project_id, corpus.path_for(item)).source
                documents = analyze_and_read(
                    client,
                    project.project_id,
                    source.source_id,
                    timeout_seconds=options.timeout_seconds,
                )
                candidates = documents["discovery.candidates.v1"]
                ranking = documents["ranking.set.v1"]
                verify_candidates(candidates)
                verify_ranking(ranking, candidates)
                pairs.append((annotation, clips_from_ranking(ranking, candidates)))
            if not pairs:
                raise RuntimeError("no corpus item carried an annotation to score against")
            return _write_recall(score_run(pairs).report(), options)
        if options.command == "score-recall":
            annotation = load_annotation(options.annotation)
            candidates = json.loads(options.candidates.read_text(encoding="utf-8"))
            ranking = json.loads(options.ranking.read_text(encoding="utf-8"))
            verify_candidates(candidates)
            verify_ranking(ranking, candidates)
            clips = clips_from_ranking(ranking, candidates)
            return _write_recall(score_run([(annotation, clips)]).report(), options)
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


def _write_recall(report: dict, options: argparse.Namespace) -> int:
    """Write the report, then hold it to a bar if one was named.

    The report is written before the bar is checked, always. A failing gate
    that produced no evidence is a gate nobody can act on, and the first thing
    anybody asks after a recall failure is which moments were missed.
    """

    options.output.parent.mkdir(parents=True, exist_ok=True)
    options.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(
        f"recall {report['recall']:.3f} "
        f"({report['moments_recalled']}/{report['moments_total']} moments), "
        f"duplicate rate {report['duplicate_rate']:.3f}, "
        f"median boundary error {report['boundary_edge_error_millis']['median']} ms"
    )
    if options.bar is None:
        return 0
    bar = json.loads(options.bar.read_text(encoding="utf-8"))
    failures = meets_bar(report, bar)
    for failure in failures:
        print(f"clipmill-eval: {failure}", file=sys.stderr)
    return 1 if failures else 0


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

    fetch = subcommands.add_parser("fetch-corpus")
    fetch.add_argument("--spec", type=Path, required=True)
    fetch.add_argument("--corpus-dir", type=Path, required=True)
    fetch.add_argument("--output-dir", type=Path, required=True)
    fetch.add_argument("--repository", type=Path, default=Path.cwd())
    # Named rather than assumed on PATH, and passed as one string so a pinned
    # build can be given as `uvx yt-dlp@2025.1.1`.
    fetch.add_argument("--downloader", default="yt-dlp")

    annotate = subcommands.add_parser("annotate")
    _corpus_arguments(annotate)
    annotate.add_argument("--socket", type=Path, required=True)
    annotate.add_argument("--output-dir", type=Path, required=True)
    annotate.add_argument("--annotator", required=True)
    annotate.add_argument("--timeout-seconds", type=float, default=1800.0)

    recall = subcommands.add_parser("recall")
    _corpus_arguments(recall)
    recall.add_argument("--socket", type=Path, required=True)
    recall.add_argument("--annotations", type=Path, required=True)
    recall.add_argument("--output", type=Path, required=True)
    recall.add_argument("--bar", type=Path)
    recall.add_argument("--timeout-seconds", type=float, default=1800.0)

    score = subcommands.add_parser("score-recall")
    score.add_argument("--annotation", type=Path, required=True)
    score.add_argument("--candidates", type=Path, required=True)
    score.add_argument("--ranking", type=Path, required=True)
    score.add_argument("--output", type=Path, required=True)
    score.add_argument("--bar", type=Path)

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
