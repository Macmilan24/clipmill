//! The editorial brief: what a person wants made from a recording.
//!
//! This is the deterministic half: what the person chose, completed with
//! defaults that suit the genre and the platforms, and limited by what the
//! platforms accept. Every default is named in `inferred`, so the Studio can
//! show it as a suggestion. Reading a typed prompt into content goals is the
//! model's half, layered on top.

use clipmill_contracts::schemas::mind_brief::{
    MindBrief, MindBriefAspect as Aspect, MindBriefFormsItem as Form, MindBriefGenre as Genre,
    MindBriefGoal as Goal, MindBriefPlatformsItem as Platform,
};
use serde_json::{Value, json};

use crate::SECOND;

/// What the person chose. Everything left `None` or empty is filled in.
#[derive(Clone, Debug, Default)]
pub struct Chosen {
    pub goal: Option<Goal>,
    pub genre: Option<Genre>,
    pub platforms: Vec<Platform>,
    pub aspect: Option<Aspect>,
    pub language: Option<String>,
    /// `(min, max)` ticks.
    pub length: Option<(u64, u64)>,
    pub count: Option<u32>,
    pub forms: Vec<Form>,
    pub topics: Vec<String>,
    pub people: Vec<String>,
    pub moments: Vec<String>,
    pub include: Vec<String>,
    pub avoid: Vec<String>,
    pub caption_style_ref: Option<String>,
    pub prompt: Option<String>,
}

/// Why a brief could not be made.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum BriefError {
    #[error("a clip cannot be shorter than {min} s and longer than {max} s at once")]
    Length { min: u64, max: u64 },
    #[error("the brief could not be assembled: {0}")]
    Invalid(String),
}

/// The length a genre's clips usually work at, as `(min, target, max)` seconds.
const fn genre_length(genre: Genre) -> (u64, u64, u64) {
    match genre {
        Genre::Tutorial => (30, 60, 120),
        Genre::Vlog | Genre::Music => (15, 30, 60),
        Genre::Sports | Genre::Gaming => (10, 25, 60),
        Genre::Podcast
        | Genre::Interview
        | Genre::Talk
        | Genre::Scripted
        | Genre::News
        | Genre::Other => (20, 45, 90),
    }
}

/// The longest clip a platform accepts in its short-video feed, in seconds.
const fn platform_max(platform: Platform) -> u64 {
    match platform {
        Platform::YoutubeShorts | Platform::InstagramReels | Platform::Facebook => 180,
        Platform::Threads | Platform::X => 140,
        Platform::Tiktok | Platform::Linkedin => 600,
        Platform::Youtube | Platform::Other => 3_600,
    }
}

/// Whether a platform's feed is landscape first.
const fn landscape_first(platform: Platform) -> bool {
    matches!(platform, Platform::Youtube | Platform::Linkedin)
}

/// The brief for what the person chose, with every gap filled and named.
pub fn compile(chosen: &Chosen) -> Result<MindBrief, BriefError> {
    let mut inferred: Vec<&str> = Vec::new();
    let mut infer = |field: &'static str, was_chosen: bool| {
        if !was_chosen {
            inferred.push(field);
        }
    };

    let genre = chosen.genre.unwrap_or(Genre::Podcast);
    infer("genre", chosen.genre.is_some());
    let goal = chosen.goal.unwrap_or(match genre {
        Genre::Tutorial => Goal::Tutorial,
        _ => Goal::Highlights,
    });
    infer("goal", chosen.goal.is_some());
    let platforms = if chosen.platforms.is_empty() {
        vec![
            Platform::YoutubeShorts,
            Platform::Tiktok,
            Platform::InstagramReels,
        ]
    } else {
        chosen.platforms.clone()
    };
    infer("platforms", !chosen.platforms.is_empty());
    let aspect = chosen
        .aspect
        .unwrap_or(if platforms.iter().copied().all(landscape_first) {
            Aspect::Landscape
        } else {
            Aspect::Vertical
        });
    infer("aspect", chosen.aspect.is_some());
    infer("language", chosen.language.is_some());

    let (min, target, max) = length(chosen.length, genre, &platforms)?;
    infer("length", chosen.length.is_some());

    let target_count = chosen.count.unwrap_or(10).clamp(1, 100);
    infer("count", chosen.count.is_some());
    let forms: Vec<Form> = if chosen.forms.is_empty() {
        vec![
            Form::Straight,
            Form::Tightened,
            Form::Stitch,
            Form::ColdOpen,
            Form::ThemeMontage,
        ]
    } else {
        chosen.forms.clone()
    };
    infer("forms", !chosen.forms.is_empty());
    infer("production", false);
    infer("generation", false);

    let mut brief = json!({
        "schema_version": "clipmill.mind.brief.v1",
        "goal": goal,
        "genre": genre,
        "platforms": platforms,
        "aspect": aspect,
        "length": {"min_ticks": min, "max_ticks": max, "target_ticks": target},
        "count": {"target": target_count, "all_worthwhile": chosen.count.is_none()},
        "forms": forms,
        "content": content(chosen),
        "production": {"hook_style": "punch_in", "effects": "light", "broll": "none", "music": "none"},
        "generation": {"voice": false, "images": false, "video": false},
        "inferred": inferred,
    });
    if let Some(language) = &chosen.language {
        brief["language"] = json!(language);
    }
    if let Some(style) = &chosen.caption_style_ref {
        brief["production"]["caption_style_ref"] = json!(style);
    }
    if let Some(prompt) = chosen
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        brief["prompt"] = json!(prompt.chars().take(1000).collect::<String>());
    }
    serde_json::from_value(brief).map_err(|error| BriefError::Invalid(error.to_string()))
}

/// `(min, target, max)` ticks: what the person chose or the genre's usual
/// length, capped by the platforms, which bind a choice as much as a default.
fn length(
    chosen: Option<(u64, u64)>,
    genre: Genre,
    platforms: &[Platform],
) -> Result<(u64, u64, u64), BriefError> {
    let ceiling = platforms
        .iter()
        .copied()
        .map(platform_max)
        .min()
        .unwrap_or(u64::MAX)
        .saturating_mul(SECOND);
    let (genre_min, genre_target, genre_max) = genre_length(genre);
    let (min, max) = chosen.unwrap_or((genre_min * SECOND, genre_max * SECOND));
    let max = max.min(ceiling);
    if min == 0 || min > max {
        return Err(BriefError::Length {
            min: min / SECOND,
            max: max / SECOND,
        });
    }
    Ok((min, (genre_target * SECOND).clamp(min, max), max))
}

fn content(chosen: &Chosen) -> Value {
    let mut content = serde_json::Map::new();
    for (key, phrases) in [
        ("topics", &chosen.topics),
        ("people", &chosen.people),
        ("moments", &chosen.moments),
        ("include", &chosen.include),
        ("avoid", &chosen.avoid),
    ] {
        let kept: Vec<String> = phrases
            .iter()
            .map(|phrase| phrase.trim().chars().take(200).collect::<String>())
            .filter(|phrase| !phrase.is_empty())
            .take(20)
            .collect();
        if !kept.is_empty() {
            content.insert(key.to_owned(), json!(kept));
        }
    }
    Value::Object(content)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;
    use clipmill_contracts::schemas::mind_brief::MindBriefInferredItem as Inferred;

    #[test]
    fn nothing_chosen_is_a_podcast_highlight_for_the_short_feeds() {
        let brief = compile(&Chosen::default()).unwrap();
        assert_eq!(brief.genre, Genre::Podcast);
        assert_eq!(brief.goal, Goal::Highlights);
        assert_eq!(brief.aspect, Aspect::Vertical);
        assert_eq!(brief.length.min_ticks.get(), 20 * SECOND);
        assert_eq!(brief.length.max_ticks.get(), 90 * SECOND);
        assert_eq!(brief.length.target_ticks.get(), 45 * SECOND);
        assert_eq!(brief.forms.len(), 5);
        assert_eq!(brief.count.all_worthwhile, Some(true));
        assert!(brief.inferred.contains(&Inferred::Genre));
        assert!(brief.inferred.contains(&Inferred::Length));
    }

    #[test]
    fn a_tutorial_runs_longer_and_a_game_clip_shorter() {
        let tutorial = compile(&Chosen {
            genre: Some(Genre::Tutorial),
            ..Chosen::default()
        })
        .unwrap();
        assert_eq!(tutorial.goal, Goal::Tutorial);
        assert_eq!(tutorial.length.max_ticks.get(), 120 * SECOND);
        let gaming = compile(&Chosen {
            genre: Some(Genre::Gaming),
            ..Chosen::default()
        })
        .unwrap();
        assert_eq!(gaming.length.target_ticks.get(), 25 * SECOND);
    }

    #[test]
    fn the_platforms_cap_what_a_person_chose() {
        let brief = compile(&Chosen {
            platforms: vec![Platform::YoutubeShorts, Platform::Tiktok],
            length: Some((30 * SECOND, 400 * SECOND)),
            ..Chosen::default()
        })
        .unwrap();
        assert_eq!(brief.length.max_ticks.get(), 180 * SECOND);
        assert!(!brief.inferred.contains(&Inferred::Length));
        assert!(!brief.inferred.contains(&Inferred::Platforms));
    }

    #[test]
    fn landscape_feeds_get_a_landscape_frame() {
        let brief = compile(&Chosen {
            platforms: vec![Platform::Youtube],
            ..Chosen::default()
        })
        .unwrap();
        assert_eq!(brief.aspect, Aspect::Landscape);
    }

    #[test]
    fn an_impossible_length_is_refused() {
        let error = compile(&Chosen {
            platforms: vec![Platform::YoutubeShorts],
            length: Some((200 * SECOND, 300 * SECOND)),
            ..Chosen::default()
        })
        .unwrap_err();
        assert_eq!(error, BriefError::Length { min: 200, max: 180 });
    }

    #[test]
    fn content_and_prompt_are_kept_as_typed_and_bounded() {
        let brief = compile(&Chosen {
            topics: vec!["  pricing ".to_owned(), String::new()],
            avoid: vec!["sponsor reads".to_owned()],
            prompt: Some("Find every time they disagree about pricing".to_owned()),
            count: Some(5),
            ..Chosen::default()
        })
        .unwrap();
        let topics: Vec<String> = brief
            .content
            .topics
            .as_ref()
            .unwrap()
            .0
            .iter()
            .map(|t| t.to_string())
            .collect();
        assert_eq!(topics, ["pricing"]);
        assert_eq!(brief.count.target.get(), 5);
        assert_eq!(brief.count.all_worthwhile, Some(false));
        assert!(brief.prompt.is_some());
    }
}
