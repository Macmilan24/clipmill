"""Language codes crossing into a model that names its languages in English.

These run everywhere, including on a machine with no MLX at all: the mapping is
the part that can be wrong without any weights present, and it is the part
whose failure produces text that reads fine and is in the wrong language.
"""

from __future__ import annotations

import pytest
from clipmill_worker_speech_mlx.languages import (
    LANGUAGE_NAMES,
    UNDETERMINED,
    UnsupportedLanguage,
    to_language_code,
    to_model_name,
)

SUPPORTED = frozenset(name.lower() for name in LANGUAGE_NAMES.values())


def test_a_code_becomes_the_name_the_model_was_trained_to_read():
    assert to_model_name("en", SUPPORTED) == "English"
    assert to_model_name("JA", SUPPORTED) == "Japanese", "the code's case is not meaning"


def test_a_code_nothing_maps_is_refused_with_what_is_mapped():
    with pytest.raises(UnsupportedLanguage, match="not a language this implementation maps"):
        to_model_name("cy", SUPPORTED)


def test_weights_that_do_not_claim_a_language_override_the_table():
    """The table proposes; the loaded model disposes.

    A recognizer asked for Korean by weights that were never trained on it
    would answer anyway, in something else, with no sign that it had.
    """

    with pytest.raises(UnsupportedLanguage, match="do not support Korean"):
        to_model_name("ko", frozenset({"english", "chinese"}))


def test_weights_that_publish_no_language_list_leave_the_table_alone():
    assert to_model_name("ko", None) == "Korean"


def test_a_detected_language_nobody_maps_is_unknown_rather_than_english():
    assert to_language_code("English") == "en"
    assert to_language_code("  chinese ") == "zh"
    assert to_language_code("Welsh") == UNDETERMINED


def test_every_mapped_name_maps_back_to_its_own_code():
    for code, name in LANGUAGE_NAMES.items():
        assert to_language_code(name) == code
