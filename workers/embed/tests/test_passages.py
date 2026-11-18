from pathlib import Path

import numpy as np
import pytest
from clipmill_worker_embed.passages import MAX_WORDS, passages

S = 90_000


def sentence(index, text, start):
    return {"index": index, "text": text, "start_ticks": start * S, "end_ticks": start * S + S}


def test_fragments_join_until_they_say_something():
    found = passages([sentence(i, "three words here", i) for i in range(20)])
    assert [(p.first, p.last) for p in found] == [(0, 6), (7, 13), (14, 19)]
    assert found[0].text.startswith("three words here three words here")


def test_a_long_silence_ends_a_passage():
    found = passages([sentence(0, "before the break", 0), sentence(1, "after it", 10)])
    assert [(p.first, p.last) for p in found] == [(0, 0), (1, 1)]


def test_no_passage_grows_past_the_limit():
    long = " ".join(["word"] * 50)
    found = passages([sentence(i, long, i) for i in range(4)])
    assert all(len(p.text.split()) <= MAX_WORDS for p in found)


MODEL = Path(__file__).resolve().parents[3] / ".cache" / "models" / "multilingual-e5-small"


@pytest.mark.skipif(
    not (MODEL / "tokenizer.json").exists(), reason="the pinned model is not fetched"
)
def test_a_question_lands_nearest_the_passage_it_means():
    from clipmill_worker_embed.encoder import Encoder

    class Pinned:
        def path(self, name):
            return MODEL / name

    encoder = Encoder(Pinned())
    docs = encoder.encode(
        [
            "passage: The subscription costs twelve dollars a month after the free trial.",
            "passage: We hiked up the mountain and watched the sun rise over the valley.",
            "passage: The model forgets earlier parts of a long conversation.",
        ]
    )
    query = encoder.encode(["query: how much does it cost"])[0]
    assert np.allclose(np.linalg.norm(docs, axis=1), 1.0, atol=1e-4)
    assert int(np.argmax(docs @ query)) == 0
