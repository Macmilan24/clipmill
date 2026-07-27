"""What this family says on a machine that cannot run it.

The interesting case is Linux, where `mlx` is not installed at all — and where
this suite still runs, because the lockfile's platform marker resolves the
dependency to nothing rather than to a broken environment. What must be true
there is that asking for MLX produces a named refusal rather than an
ImportError the scheduler would have to interpret.
"""

from __future__ import annotations

import sys

import pytest
from clipmill_worker_speech_mlx import CAPABILITIES
from clipmill_worker_speech_mlx.runtime import (
    ALIGNER_MODEL_TYPE,
    ASR_MODEL_TYPE,
    MlxUnavailable,
    require_mlx,
    supported_languages,
)

ON_APPLE_SILICON = sys.platform == "darwin"


def test_the_family_serves_both_speech_stages_that_run_a_qwen3_model():
    """One environment, two capabilities: they are one model family."""

    assert CAPABILITIES == ("speech-asr", "speech-align")


def test_the_two_architectures_are_named_rather_than_inferred():
    """Loading must not depend on what the weights directory is called."""

    assert ASR_MODEL_TYPE != ALIGNER_MODEL_TYPE
    assert ASR_MODEL_TYPE == "qwen3_asr"
    assert ALIGNER_MODEL_TYPE == "qwen3_forced_aligner"


@pytest.mark.skipif(ON_APPLE_SILICON, reason="MLX is installed here")
def test_a_machine_without_mlx_refuses_by_name():
    with pytest.raises(MlxUnavailable, match="Apple silicon"):
        require_mlx()


def test_weights_that_publish_no_language_list_say_so_rather_than_claiming_none():
    """None and the empty set mean different things.

    None is "these weights make no claim", which leaves the mapping table in
    charge. An empty set would mean "these weights support nothing", which
    would refuse every language.
    """

    class Silent:
        pass

    class Empty:
        def get_supported_languages(self):
            return []

    class Speaks:
        def get_supported_languages(self):
            return ["English", "chinese"]

    assert supported_languages(Silent()) is None
    assert supported_languages(Empty()) is None
    assert supported_languages(Speaks()) == frozenset({"english", "chinese"})
