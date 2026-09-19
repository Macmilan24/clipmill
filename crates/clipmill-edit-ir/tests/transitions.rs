#![allow(clippy::unwrap_used)]

use clipmill_edit_ir::{EditCommand, EditDocument};

#[test]
fn legacy_documents_keep_their_bytes_and_soft_cuts_undo_exactly() {
    let mut document = EditDocument::default();
    let legacy = document.to_canonical_json().unwrap();
    assert!(!String::from_utf8_lossy(&legacy).contains("transition_ticks"));
    document = EditDocument::from_canonical_json(&legacy).unwrap();
    assert_eq!(document.video.transition_ticks, 0);
    assert_eq!(document.to_canonical_json().unwrap(), legacy);
    let command = EditCommand::SetTransition {
        duration_ticks: 10_800,
    };
    let inverse = command.apply(&mut document).unwrap();
    let enabled = document.to_canonical_json().unwrap();
    assert_eq!(document.video.transition_ticks, 10_800);
    assert_eq!(document.program_duration_ticks(), 0);
    assert_eq!(inverse.apply(&mut document).unwrap(), command);
    assert_eq!(document.to_canonical_json().unwrap(), legacy);
    let replay = EditCommand::from_canonical_json(&command.to_canonical_json().unwrap()).unwrap();
    replay.apply(&mut document).unwrap();
    assert_eq!(document.to_canonical_json().unwrap(), enabled);
}

#[test]
fn invalid_durations_leave_the_document_untouched() {
    for duration_ticks in [-1, 22_501, i64::MAX] {
        let mut document = EditDocument::default();
        let before = document.to_canonical_json().unwrap();
        assert!(
            EditCommand::SetTransition { duration_ticks }
                .apply(&mut document)
                .is_err()
        );
        assert_eq!(document.to_canonical_json().unwrap(), before);
    }
}
