import pytest
from clipmill_worker_audio_events.events import KINDS, detect, kind_columns

LABELS = [
    "Speech",
    "Laughter",
    "Giggle",
    "Applause",
    "Clapping",
    "Cheering",
    "Crowd",
    "Music",
    "Shout",
    "Gasp",
]
WINDOW = 32_000
HOP = 16_000


def rows(**series):
    """Window rows with the given per-label scores and zero elsewhere."""
    length = max(len(values) for values in series.values())
    table = []
    for position in range(length):
        row = [0.0] * len(LABELS)
        for label, values in series.items():
            if position < len(values):
                row[LABELS.index(label.replace("_", " "))] = values[position]
        table.append(row)
    return table


def events(table, total=None):
    columns = kind_columns(LABELS)
    total = total if total is not None else (len(table) - 1) * HOP + WINDOW
    return detect(table, columns, window_samples=WINDOW, hop_samples=HOP, total_samples=total)


def test_a_laugh_is_one_event_from_its_first_window_to_its_last():
    found = events(rows(Laughter=[0.0, 0.1, 0.3, 0.8, 0.6, 0.25, 0.0, 0.0]))
    assert [(e.kind, e.start_sample, e.end_sample) for e in found] == [
        ("laughter", 2 * HOP, 5 * HOP + WINDOW)
    ]
    assert found[0].peak == 0.8


def test_a_kind_takes_the_loudest_of_its_classes():
    found = events(rows(Laughter=[0.1, 0.1], Giggle=[0.5, 0.0]))
    assert [e.kind for e in found] == ["laughter"]


def test_one_quiet_window_does_not_split_a_run():
    found = events(rows(Applause=[0.9, 0.0, 0.9, 0.0, 0.0, 0.9]))
    assert len(found) == 2
    assert found[0].end_sample == 2 * HOP + WINDOW


def test_scores_that_never_reach_on_are_no_event():
    assert events(rows(Music=[0.4, 0.45, 0.49])) == []


def test_events_are_in_time_order_then_kind():
    found = events(rows(Cheering=[0.0, 0.9], Applause=[0.0, 0.9], Laughter=[0.9, 0.0]))
    assert [e.kind for e in found] == ["laughter", "applause", "cheering"]


def test_the_last_window_ends_at_the_recording():
    found = events(rows(Shout=[0.0, 0.9]), total=HOP + 20_000)
    assert found[0].end_sample == HOP + 20_000


def test_a_tagger_missing_a_kind_is_refused():
    with pytest.raises(ValueError, match="gasp"):
        kind_columns([label for label in LABELS if label != "Gasp"])


def test_every_kind_has_thresholds():
    from clipmill_worker_audio_events.events import THRESHOLDS

    for kind in KINDS:
        on, off = THRESHOLDS[kind]
        assert 0 < off < on < 1
