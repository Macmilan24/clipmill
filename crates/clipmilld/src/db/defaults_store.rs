//! The look a person's clips start with, kept where the engine can read it.
//!
//! The Studio chooses a caption look per project and a brand kit for the
//! installation, and used to keep both to itself. A clip an agent or the
//! command line made then started plain. The Studio now sends each choice
//! here when it changes, and the engine starts every clip it makes from it.
//! One row per scope: the installation's kit, and each project's look.

use clipmill_contracts::proto::ipc::v1::ClipDefaultsV1;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Value, json};

pub(super) const CREATE_V18_TABLES: &str = "
    CREATE TABLE clip_defaults (
        scope TEXT NOT NULL PRIMARY KEY CHECK(length(scope) > 0),
        defaults_json TEXT NOT NULL CHECK(json_valid(defaults_json)),
        updated_unix_millis INTEGER NOT NULL CHECK(updated_unix_millis >= 0)
    ) STRICT, WITHOUT ROWID;
";

/// The scope of the installation's kit. Project ids start `prj_`.
const KIT: &str = "kit";

fn to_json(defaults: &ClipDefaultsV1) -> String {
    let mut value = json!({
        "style_ref": defaults.style_ref,
        "caption_options_json": defaults.caption_options_json,
        "brand_json": defaults.brand_json,
        "shape": defaults.shape,
    });
    if let Some(highlight) = defaults.highlight_spoken_word {
        value["highlight_spoken_word"] = json!(highlight);
    }
    value.to_string()
}

fn from_json(text: &str) -> ClipDefaultsV1 {
    let value: Value = serde_json::from_str(text).unwrap_or_default();
    let text = |name: &str| value[name].as_str().unwrap_or_default().to_owned();
    ClipDefaultsV1 {
        style_ref: text("style_ref"),
        highlight_spoken_word: value["highlight_spoken_word"].as_bool(),
        caption_options_json: text("caption_options_json"),
        brand_json: text("brand_json"),
        shape: text("shape"),
    }
}

/// Replace one scope's defaults: a project's look, or with no project the kit.
pub(super) fn set(
    connection: &Connection,
    project_id: &str,
    defaults: &ClipDefaultsV1,
    now: u64,
) -> rusqlite::Result<()> {
    let scope = if project_id.is_empty() {
        KIT
    } else {
        project_id
    };
    connection.execute(
        "INSERT INTO clip_defaults (scope, defaults_json, updated_unix_millis)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(scope) DO UPDATE SET
             defaults_json = excluded.defaults_json,
             updated_unix_millis = excluded.updated_unix_millis",
        params![
            scope,
            to_json(defaults),
            i64::try_from(now).unwrap_or(i64::MAX)
        ],
    )?;
    Ok(())
}

/// What a new clip in this project starts with: the project's own look,
/// else the look new projects start with, and the kit's brand and frame.
pub(super) fn for_project(
    connection: &Connection,
    project_id: &str,
) -> rusqlite::Result<ClipDefaultsV1> {
    let read = |scope: &str| {
        connection
            .query_row(
                "SELECT defaults_json FROM clip_defaults WHERE scope = ?1",
                [scope],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map(|found| found.map(|text| from_json(&text)))
    };
    let kit = read(KIT)?.unwrap_or_default();
    let Some(project) = read(project_id)? else {
        return Ok(ClipDefaultsV1 {
            caption_options_json: String::new(),
            highlight_spoken_word: None,
            ..kit
        });
    };
    Ok(ClipDefaultsV1 {
        style_ref: if project.style_ref.is_empty() {
            kit.style_ref
        } else {
            project.style_ref
        },
        highlight_spoken_word: project.highlight_spoken_word,
        caption_options_json: project.caption_options_json,
        brand_json: kit.brand_json,
        shape: kit.shape,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn a_project_starts_from_its_own_look_and_the_installations_kit() {
        let connection = Connection::open_in_memory().expect("memory database");
        connection.execute_batch(CREATE_V18_TABLES).expect("tables");
        assert_eq!(
            for_project(&connection, "prj_a").expect("read"),
            ClipDefaultsV1::default()
        );
        let kit = ClipDefaultsV1 {
            style_ref: "clipmill.captions.boxed.v1".to_owned(),
            brand_json: r##"{"progress":{"colour":"#FFD65C","edge":"bottom","thickness":10}}"##
                .to_owned(),
            shape: "square".to_owned(),
            ..ClipDefaultsV1::default()
        };
        set(&connection, "", &kit, 1).expect("kit");
        // A project nobody chose a look for starts with the starting look.
        let plain = for_project(&connection, "prj_a").expect("read");
        assert_eq!(plain.style_ref, kit.style_ref);
        assert_eq!(
            (plain.brand_json.as_str(), plain.shape.as_str()),
            (kit.brand_json.as_str(), "square")
        );
        let look = ClipDefaultsV1 {
            style_ref: "clipmill.captions.minimal.v1".to_owned(),
            highlight_spoken_word: Some(false),
            caption_options_json: r#"{"font_size":64}"#.to_owned(),
            ..ClipDefaultsV1::default()
        };
        set(&connection, "prj_a", &look, 2).expect("look");
        let chosen = for_project(&connection, "prj_a").expect("read");
        assert_eq!(chosen.style_ref, look.style_ref);
        assert_eq!(chosen.highlight_spoken_word, Some(false));
        assert_eq!(chosen.caption_options_json, look.caption_options_json);
        assert_eq!(chosen.brand_json, kit.brand_json);
        // Another project still starts from the kit alone.
        assert_eq!(
            for_project(&connection, "prj_b").expect("read").style_ref,
            kit.style_ref
        );
    }
}
