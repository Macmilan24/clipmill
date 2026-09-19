import json
from types import SimpleNamespace

import clipmill_youtube_import as importer
import pytest


@pytest.mark.parametrize(
    "path",
    [
        "https://youtu.be/NYFGCESmikA?si=share",
        "https://www.youtube.com/watch?v=NYFGCESmikA&list=ignored&t=30",
        "https://m.youtube.com/shorts/NYFGCESmikA",
        "https://youtube.com/embed/NYFGCESmikA",
        "https://www.youtube.com/live/NYFGCESmikA",
    ],
)
def test_share_links_resolve_exactly_one_identity(path):
    assert importer.canonical_url(path) == (
        "https://www.youtube.com/watch?v=NYFGCESmikA",
        "NYFGCESmikA",
    )


@pytest.mark.parametrize(
    "path",
    [
        "file:///etc/passwd",
        "http://www.youtube.com/watch?v=NYFGCESmikA",
        "https://youtube.com.evil.invalid/watch?v=NYFGCESmikA",
        "https://youtube.com@evil.invalid/watch?v=NYFGCESmikA",
        "https://user:password@youtube.com/watch?v=NYFGCESmikA",
        "https://youtube.com:8443/watch?v=NYFGCESmikA",
        "https://youtube.com/playlist?list=anything",
        "https://youtube.com/@channel",
        "https://youtu.be/NYFGCESmikA/extra",
        "https://youtube.com/watch?v=NYFGCESmikA&v=OTHER______",
        "https://youtu.be/NYFGCESmikA\n--exec=oops",
        "https://youtu.be/%2e%2e",
    ],
)
def test_hostile_or_non_video_urls_are_rejected_before_network(path):
    with pytest.raises(importer.ImportFailure, match="YouTube"):
        importer.canonical_url(path)


def metadata(**extra):
    return {"id": "NYFGCESmikA", "title": "Creator title", "duration": 90, **extra}


@pytest.mark.parametrize(
    "extra",
    [
        {"_type": "playlist"},
        {"id": "mismatched"},
        {"is_live": True},
        {"live_status": "is_upcoming"},
        {"live_status": "post_live"},
        {"has_drm": True},
        {"duration": None},
        {"duration": float("nan")},
        {"duration": 0},
        {"duration": 21601},
        {"requested_formats": [{"filesize": importer.MAX_BYTES}, {"filesize": 100}]},
    ],
)
def test_bad_metadata_cannot_advance_to_downloading(extra):
    with pytest.raises(importer.ImportFailure):
        importer.inspect_metadata(metadata(**extra), "NYFGCESmikA")


def test_unknown_length_transfer_still_stops_at_observed_total_limit(tmp_path, monkeypatch):
    monkeypatch.setattr(importer, "MAX_BYTES", 100)
    progress = importer.Progress(tmp_path)
    progress({"filename": "video", "downloaded_bytes": 70})
    with pytest.raises(importer.ImportFailure, match="8 GB"):
        progress({"filename": "audio", "downloaded_bytes": 31})


def test_stream_total_is_not_reported_as_whole_video_percentage(tmp_path, capsys):
    progress = importer.Progress(tmp_path)
    progress({"filename": "video", "downloaded_bytes": 50, "total_bytes": 100})
    progress({"filename": "audio", "downloaded_bytes": 10, "status": "finished"})
    records = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert records[-1] == {"event": "progress", "downloaded_bytes": 60}


def test_known_combined_total_covers_sequential_audio_and_video(tmp_path, capsys):
    progress = importer.Progress(tmp_path)
    progress.expected_total = 120
    progress({"filename": "video", "downloaded_bytes": 100, "status": "finished"})
    progress({"filename": "audio", "downloaded_bytes": 20, "status": "finished"})
    records = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    assert records[-1] == {"event": "progress", "downloaded_bytes": 120, "total_bytes": 120}


def test_temp_budget_counts_both_streams_and_merged_file(tmp_path, monkeypatch):
    monkeypatch.setattr(importer, "MAX_BYTES", 10)
    monkeypatch.setattr(importer, "RESERVE_BYTES", 5)
    for name in ("source.video.part", "source.audio.part", "source.mkv"):
        (tmp_path / name).write_bytes(b"x" * 9)
    assert importer.storage_failure(tmp_path)[0] == "size_limit"


def test_low_space_is_checked_even_during_merging(tmp_path, monkeypatch):
    monkeypatch.setattr(importer.shutil, "disk_usage", lambda _: SimpleNamespace(free=0))
    assert importer.storage_failure(tmp_path)[0] == "disk_full"


def test_storage_io_failure_fails_closed_instead_of_disabling_watchdog(tmp_path, monkeypatch):
    def failure(_):
        raise PermissionError("unreadable mount")

    monkeypatch.setattr(importer, "storage_failure", failure)
    assert importer.checked_storage_failure(tmp_path)[0] == "storage_error"


def test_destination_refuses_reuse_and_symlinks(tmp_path):
    (tmp_path / "partial").write_text("old")
    with pytest.raises(importer.ImportFailure):
        importer.prepare_destination(str(tmp_path))
    link = tmp_path / "escape"
    link.symlink_to(tmp_path, target_is_directory=True)
    with pytest.raises(importer.ImportFailure):
        importer.prepare_destination(str(link))
    assert importer.storage_failure(tmp_path)[0] == "storage_error"


def test_options_disable_ambient_inputs_and_partial_media(tmp_path):
    opts = importer.options(tmp_path, "/pinned/ffmpeg", "/pinned/node", lambda _: None)
    assert opts["skip_unavailable_fragments"] is False
    assert opts["cookiefile"] is None and opts["cookiesfrombrowser"] is None
    assert opts["usenetrc"] is False and opts["proxy"] == ""
    assert opts["remote_components"] == [] and opts["enable_file_urls"] is False
    assert opts["js_runtimes"] == {"node": {"path": "/pinned/node"}}
    assert opts["noplaylist"] and opts["allowed_extractors"] == ["youtube"]


@pytest.mark.parametrize("quality", [360, 720, 1080])
def test_real_extractor_format_selection_respects_quality_limit(tmp_path, quality):
    from yt_dlp import YoutubeDL
    from yt_dlp.globals import plugin_dirs

    plugin_dirs.value = []
    formats = [
        {
            "format_id": str(height),
            "height": height,
            "width": height * 16 // 9,
            "vcodec": "avc1",
            "acodec": "none",
            "ext": "mp4",
            "protocol": "https",
            "url": f"https://media.invalid/{height}",
        }
        for height in [360, 720, 1080]
    ] + [
        {
            "format_id": "audio",
            "vcodec": "none",
            "acodec": "opus",
            "ext": "webm",
            "protocol": "https",
            "url": "https://media.invalid/audio",
        }
    ]
    opts = importer.options(tmp_path, "/ffmpeg", "/node", lambda _: None, quality)
    with YoutubeDL(opts) as ydl:
        selected = ydl.process_video_result(
            {"id": "fixture", "title": "Fixture", "formats": formats}, download=False
        )
    assert selected["requested_formats"][0]["height"] == quality


def test_completed_output_requires_actual_final_file_not_extractor_success(tmp_path, monkeypatch):
    import yt_dlp
    from yt_dlp.globals import plugin_dirs

    class FakeDownloader:
        def __init__(self, opts):
            assert plugin_dirs.value == []
            self.opts = opts

        def __enter__(self):
            return self

        def __exit__(self, *_):
            pass

        def extract_info(self, url, download):
            assert not download
            return metadata()

        def process_info(self, info):
            pass  # A provider returned success without a complete file.

    monkeypatch.setattr(yt_dlp, "YoutubeDL", FakeDownloader)
    monkeypatch.setattr(importer, "monitor_storage", lambda _: None)
    with pytest.raises(importer.ImportFailure, match="complete playable"):
        importer.download("https://youtu.be/NYFGCESmikA", str(tmp_path), "/ffmpeg", "/node")


def test_provider_failure_never_echoes_signed_urls_or_credentials(monkeypatch, capsys):
    monkeypatch.setattr(importer, "watch_parent", lambda: None)
    monkeypatch.setattr(importer, "check_runtime", lambda *_: "/node")
    monkeypatch.setattr(
        "sys.argv", ["import", "--url", "url", "--destination", "/tmp", "--ffmpeg", "/ffmpeg"]
    )

    def fail(*_):
        raise RuntimeError("secret path /Users/private, https://media.invalid?token=SECRET")

    monkeypatch.setattr(importer, "download", fail)
    assert importer.main() == 1
    output = capsys.readouterr().out
    assert "SECRET" not in output and "private" not in output
    assert json.loads(output)["code"] == "download_failed"


def test_runtime_check_has_no_network_and_requires_compatible_node(tmp_path, monkeypatch):
    binary = tmp_path / "ffmpeg"
    binary.touch(mode=0o700)
    monkeypatch.setattr(
        importer.subprocess, "run", lambda *_args, **_kw: SimpleNamespace(stdout="v20.1.0")
    )
    with pytest.raises(importer.ImportFailure, match=r"Node\.js 22"):
        importer.check_runtime(str(binary), "/node")
