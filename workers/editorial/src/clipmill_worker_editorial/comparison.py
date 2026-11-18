"""Identify a recorded measurement that a conclusion still compares after an edit."""

from __future__ import annotations

import re
from decimal import Decimal

# This check requires a numerical pair, explicit comparative source wording and
# a conclusion referring back to it. Other joins receive the normal source review.
COMPARES = re.compile(
    r"\b(?:(?:same|identical) (?:workload|inputs?|conditions?|runs?|tests?|trip|journey|"
    r"distance|task|sample|batch|service|equipment)|second (?:run|test|group)|"
    r"other (?:run|option|group)|compared (?:with|to)|by contrast|in contrast|"
    r"(?:in|for) comparison|instead|whereas|versus|"
    r"fell to|rose to|dropped to|increased to|reduced to)\b",
    re.IGNORECASE,
)
REFERS_BACK = re.compile(r"^(?:(?:and|so|but)[, ]+)?(?:this|that|these|those|it|which)\b", re.I)
RESULT = re.compile(
    r"\b(?:differences?|gaps?|rounding errors?|savings?|improvements?|reductions?|"
    r"increases?|decreases?|drops?|margins?|ratios?|hal(?:f|ved)|doubl(?:e|ed)|"
    r"faster|slower|cheaper|more expensive|times (?:as|the))\b",
    re.IGNORECASE,
)

UNITS = {
    unit: (dimension, scale)
    for dimension, scale, symbols in (
        ("usd", 1, ("$", "dollar", "dollars")),
        ("eur", 1, ("€", "euro", "euros")),
        ("gbp", 1, ("£", "pound", "pounds")),
        ("percent", 1, ("%", "percent", "per cent")),
        ("milliseconds", 1, ("ms", "millisecond", "milliseconds")),
        ("milliseconds", 1000, ("s", "sec", "secs", "second", "seconds")),
        ("milliseconds", 60_000, ("min", "mins", "minute", "minutes")),
    )
    for unit in symbols
}
AMOUNT = r"\d+(?:,\d{3})*(?:\.\d+)?"
SUFFIX = "|".join(sorted((re.escape(u) for u in UNITS if u[0].isalnum() or u == "%"), reverse=True))
MEASURE = re.compile(
    rf"(?<![\w.])(?:([$€£])\s*({AMOUNT})|({AMOUNT})\s*({SUFFIX}))(?!\w)", re.IGNORECASE
)


def measurements(text):
    for match in MEASURE.finditer(text):
        prefix, amount, number, suffix = match.groups()
        dimension, scale = UNITS[(prefix or suffix).casefold()]
        yield dimension, Decimal((amount or number).replace(",", "")) * scale, match.group()


def missing_comparison(join):
    """Return the source-grounded finding only if a distinct comparator disappeared."""
    removed = " ".join(line["text"] for line in join.get("removed", []))
    omitted = [
        quantity
        for sentence in re.split(r"(?<=[.!?])\s+", removed)
        if COMPARES.search(sentence)
        for quantity in measurements(sentence)
    ]
    if not omitted:
        return None
    right = " ".join(line["text"] for line in join["heard_right"]).strip()
    conclusion = re.split(r"(?<=[.!?])\s+", right, maxsplit=1)[0]
    if not REFERS_BACK.search(conclusion) or not RESULT.search(conclusion):
        return None
    left = " ".join(line["text"] for line in join["heard_left"])
    kept = list(measurements(left + " " + conclusion))
    for dimension, amount, quote in omitted:
        peers = {n: text for unit, n, text in kept if unit == dimension}
        if len(peers) == 1 and amount not in peers:
            baseline = next(iter(peers.values()))
            return f"The edit omits {quote} from the speaker's comparison with {baseline}."
    return None
