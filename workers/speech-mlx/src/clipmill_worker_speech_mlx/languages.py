"""Language codes on the contract, language names in the model.

Every artifact in the chain carries a BCP-47 code, because that is what
consumers, caption files, and the edit document speak. Qwen3 was trained on
English prose naming its languages. Somewhere the two have to meet, and doing
it in one table beats doing it at four call sites.

The table only proposes. What a model actually supports is a property of the
weights, so the caller checks each name against the loaded model's own list and
refuses a language it would otherwise silently mistranscribe — a recognizer
asked for Welsh and quietly given English produces text that reads fine and is
wrong.
"""

from __future__ import annotations

# Qwen3-ASR's languages, by the ISO 639-1 code the contract uses. Kept short
# on purpose: a code here that the weights do not support is caught at load,
# and a code missing from here is refused rather than guessed.
LANGUAGE_NAMES = {
    "ar": "Arabic",
    "de": "German",
    "en": "English",
    "es": "Spanish",
    "fr": "French",
    "it": "Italian",
    "ja": "Japanese",
    "ko": "Korean",
    "pt": "Portuguese",
    "ru": "Russian",
    "zh": "Chinese",
}

LANGUAGE_CODES = {name.lower(): code for code, name in LANGUAGE_NAMES.items()}

#: What the contract's `language` field says when nothing is known yet.
UNDETERMINED = "und"


class UnsupportedLanguage(ValueError):
    """A language this implementation cannot honestly claim to recognize."""


def to_model_name(code: str, supported: frozenset[str] | None) -> str:
    """The model's name for a language code, or a refusal naming what is.

    `supported` is the loaded model's own lower-cased list, or None when the
    weights do not publish one — in which case the table is taken at its word,
    since there is nothing better to check against.
    """

    name = LANGUAGE_NAMES.get(code.lower())
    if name is None:
        raise UnsupportedLanguage(
            f"{code} is not a language this implementation maps; "
            f"mapped codes are {', '.join(sorted(LANGUAGE_NAMES))}"
        )
    if supported is not None and name.lower() not in supported:
        raise UnsupportedLanguage(
            f"the pinned weights do not support {name}; they support {', '.join(sorted(supported))}"
        )
    return name


def to_language_code(name: str) -> str:
    """The contract's code for a language the model named.

    Falls back to `und` rather than inventing a code: a detected language this
    table does not know is a fact about the recording that the transcript
    should carry as unknown, not as English.
    """

    return LANGUAGE_CODES.get(name.strip().lower(), UNDETERMINED)


__all__ = [
    "LANGUAGE_CODES",
    "LANGUAGE_NAMES",
    "UNDETERMINED",
    "UnsupportedLanguage",
    "to_language_code",
    "to_model_name",
]
