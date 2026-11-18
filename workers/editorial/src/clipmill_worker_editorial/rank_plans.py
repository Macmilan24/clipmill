"""Counterbalanced pairwise editorial ranking, regularised strengths and diversity."""

from __future__ import annotations

from collections import Counter
from typing import Literal

from .planning import Reply, ShortText, Transcript


class Comparison(Reply):
    winner: Literal["A", "B", "tie"]
    reason: ShortText


def comparison_pairs(count):
    """A connected comparison graph bounded to one full cycle on a long recording."""
    if count < 2:
        return []
    edges = [(i, i + 1) for i in range(count - 1)]
    if count > 2:
        edges.append((count - 1, 0))
    return edges


def strengths(count, comparisons):
    """Bradley-Terry minorization-maximization with one virtual even opponent.

    The fixed-strength opponent contributes half a win and half a loss to
    everyone; unbeaten cuts have finite strength and an unjudged cut is neutral.
    """
    scores = [1.0] * count
    wins = [0.5] * count
    opponents = [[] for _ in scores]
    for a, b, share in comparisons:
        wins[a] += share
        wins[b] += 1.0 - share
        opponents[a].append(b)
        opponents[b].append(a)
    for _ in range(200):
        updated = [
            wins[i] / (1.0 / (score + 1.0) + sum(1.0 / (score + scores[j]) for j in opponents[i]))
            for i, score in enumerate(scores)
        ]
        delta = max((abs(a - b) for a, b in zip(scores, updated, strict=True)), default=0)
        scores = updated
        if delta < 1e-9:
            break
    return scores


def rank_batch(ctx, report, script, requested):
    transcript = Transcript(script)
    rows = []
    seen = set()
    for row in report["reviews"]:
        plan_id = row["plan"]["plan_id"]
        if row["quality"]["verdict"] == "accept" and plan_id not in seen:
            rows.append(row)
            seen.add(plan_id)
    # Sort by content identity before scheduling pairs: input position is no vote.
    rows.sort(key=lambda row: row["plan"]["plan_id"])
    cards = [
        {"title": row["plan"]["title"], "playback": transcript.evidence(row["plan"])["playback"]}
        for row in rows
    ]
    pairs = comparison_pairs(len(rows))
    evidence = []
    votes = []
    for index, (a, b) in enumerate(pairs):
        ctx.progress("compare_pairs", index, len(pairs))
        ballot = []
        reasons = []
        for first, second in ((a, b), (b, a)):
            answer, failure = ctx.ask(
                "compare",
                {
                    "brief": ctx.brief,
                    "previous_decisions": ctx.taste,
                    "A": cards[first],
                    "B": cards[second],
                },
                Comparison,
            )
            if failure:
                continue
            winner = answer["winner"]
            ballot.append(0.5 if winner == "tie" else float((winner == "A") == (first == a)))
            reasons.append(answer["reason"])
        # A one-sided result would reintroduce presentation order bias.
        if len(ballot) == 2:
            share = sum(ballot) / 2
            votes.append((a, b, share))
            evidence.append(
                {
                    "a": rows[a]["plan"]["plan_id"],
                    "b": rows[b]["plan"]["plan_id"],
                    "share_a": share,
                    "reasons": reasons,
                }
            )
    ctx.progress("compare_pairs", len(pairs), len(pairs))
    scores = strengths(len(rows), votes)
    remaining = set(range(len(rows)))
    chapters, forms = Counter(), Counter()
    clips = []
    while remaining and len(clips) < requested:

        def priority(i):
            row = rows[i]
            weight = scores[i] / (
                1 + 0.30 * chapters[row["chapter"]] + 0.15 * forms[row["plan"]["form"]]
            )
            return -weight, row["plan"]["plan_id"]

        i = min(remaining, key=priority)
        remaining.remove(i)
        row = rows[i]
        quality = row["quality"]
        chapters[row["chapter"]] += 1
        forms[row["plan"]["form"]] += 1
        hook_points = {"strong": 12, "adequate": 6, "weak": 0}[quality["hook"]]
        keep = round(50 + 35 * scores[i] / (1 + scores[i]) + hook_points)
        clips.append(
            {
                "plan": row["plan"],
                "chapter": row["chapter"],
                "quality": quality,
                "keep_score": keep,
                "strength": round(scores[i], 6),
                "reasons": list(dict.fromkeys([quality["summary"], *row["reasons"]])),
            }
        )
    result = {
        "clips": clips,
        "comparisons": evidence,
        "requested_count": requested,
        "editorial_prompt": ctx.brief,
    }
    if len(clips) < requested:
        noun = "cut" if requested == 1 else "cuts"
        result["cutoff_reason"] = (
            f"{len(clips)} of {requested} requested {noun} met the brief and passed review."
        )
    return result
