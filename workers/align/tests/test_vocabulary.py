"""Text the acoustic model can score, and text it cannot.

The alphabet has thirty-two labels; recognized text has rather more than that.
Every character outside the alphabet forces a choice, and the two silent
options are both wrong: dropping a word loses what was said, and aligning
around it invents a measurement. These pin the third option — say so.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from clipmill_worker_align.vocabulary import Vocabulary

# The real wav2vec2 CTC alphabet: uppercase English, an apostrophe, a word
# delimiter, a blank, and the sentence markers.
LABELS = {
    "<pad>": 0,
    "<s>": 1,
    "</s>": 2,
    "<unk>": 3,
    "|": 4,
    **{character: 5 + index for index, character in enumerate("ETAONIHSRDLUMWCFGYPBVKXJQZ")},
    "'": 31,
}


@pytest.fixture
def vocabulary() -> Vocabulary:
    return Vocabulary(LABELS)


def test_ordinary_words_are_scoreable(vocabulary: Vocabulary) -> None:
    scoreable, unscoreable = vocabulary.encode(["the", "first", "slice"])
    assert [word.normalized for word in scoreable] == ["THE", "FIRST", "SLICE"]
    assert unscoreable == []


def test_the_displayed_text_is_never_the_normalized_text(vocabulary: Vocabulary) -> None:
    """A caption shows what the recognizer wrote, not what the model scored.

    Normalization exists so the acoustic model has something in its alphabet
    to align against; letting it leak into the output would strip the
    punctuation and casing that make a transcript readable.
    """

    scoreable, _ = vocabulary.encode(["Hello,", "world!"])
    assert [word.text for word in scoreable] == ["Hello,", "world!"]
    assert [word.normalized for word in scoreable] == ["HELLO", "WORLD"]


def test_an_apostrophe_survives_because_the_model_knows_it(vocabulary: Vocabulary) -> None:
    scoreable, _ = vocabulary.encode(["don't"])
    assert scoreable[0].normalized == "DON'T"


def test_a_word_with_nothing_in_the_alphabet_is_reported_not_dropped(
    vocabulary: Vocabulary,
) -> None:
    """Digits are the common case, and the reason this is a stated limit.

    Spelling them out is language-specific — "101" is three words in one
    language and two in another — and a wrong expansion aligns the wrong
    sounds to the wrong word rather than admitting it could not.
    """

    scoreable, unscoreable = vocabulary.encode(["we", "shipped", "101", "clips"])
    assert [word.text for word in scoreable] == ["we", "shipped", "clips"]
    assert len(unscoreable) == 1
    assert unscoreable[0].text == "101"
    assert unscoreable[0].reason == "out_of_vocabulary"
    # And its position in the original text is kept, so assembly can put the
    # word back where it belongs with interpolated timing.
    assert unscoreable[0].index == 2


def test_words_are_separated_by_the_delimiter_the_model_was_trained_with(
    vocabulary: Vocabulary,
) -> None:
    """Without it, "a cat" and "acat" score identically — and the first word
    would have no end distinct from the second word's start."""

    scoreable, _ = vocabulary.encode(["a", "cat"])
    labels, spans = vocabulary.label_sequence(scoreable)
    assert labels.count(vocabulary.delimiter_id) == 1
    assert labels[spans[0][1]] == vocabulary.delimiter_id
    # Each span addresses exactly its own word's characters.
    assert [labels[start:end] for start, end in spans] == [
        list(scoreable[0].labels),
        list(scoreable[1].labels),
    ]


def test_a_single_word_needs_no_delimiter(vocabulary: Vocabulary) -> None:
    scoreable, _ = vocabulary.encode(["alone"])
    labels, spans = vocabulary.label_sequence(scoreable)
    assert vocabulary.delimiter_id not in labels
    assert spans == [(0, len("ALONE"))]


def test_an_alphabet_without_a_blank_or_delimiter_is_refused() -> None:
    with pytest.raises(ValueError, match="blank or no word delimiter"):
        Vocabulary({"A": 0, "B": 1})


def test_the_published_alphabet_loads(tmp_path: Path) -> None:
    """The file is pinned by digest and read rather than assumed.

    Hardcoding the label ordering would silently mis-time every word the day a
    model with a different alphabet is pinned.
    """

    path = tmp_path / "vocab.json"
    path.write_text(json.dumps(LABELS), encoding="utf-8")
    loaded = Vocabulary.load(path)
    assert loaded.blank_id == 0
    assert loaded.delimiter_id == 4
