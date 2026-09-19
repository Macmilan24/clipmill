#![allow(clippy::unwrap_used)]
use super::*;
use fs2::FileExt;

#[test]
fn import_quality_accepts_only_supported_choices_and_preserves_the_legacy_default() {
    assert_eq!(import_height(0), Some(1080));
    for value in [360, 720, 1080] {
        assert_eq!(import_height(value), Some(value));
    }
    for value in [1, 480, 2160, u32::MAX] {
        assert_eq!(import_height(value), None);
    }
}

#[test]
fn accepts_only_single_video_ids_at_exact_https_youtube_hosts() {
    for value in [
        "https://youtu.be/NYFGCESmikA?si=tracking",
        "https://www.youtube.com/watch?v=NYFGCESmikA&list=ignored",
        "https://m.youtube.com/shorts/NYFGCESmikA",
        "https://www.youtube.com/embed/NYFGCESmikA",
    ] {
        assert_eq!(youtube_id(value).as_deref(), Some("NYFGCESmikA"));
    }
    for value in [
        "http://youtu.be/NYFGCESmikA",
        "https://youtube.com.evil/watch?v=NYFGCESmikA",
        "https://user@youtube.com/watch?v=NYFGCESmikA",
        "https://www.youtube.com/watch?v=NYFGCESmikA&v=aaaaaaaaaaa",
        "https://www.youtube.com/playlist?list=abc",
        "https://youtu.be/../../etc",
        "file:///etc/passwd",
        "https://youtu.be/NYFGCESmikA\n",
        "https://youtu.be/NYFGCESmikA/more",
    ] {
        assert!(youtube_id(value).is_none(), "{value}");
    }
}

#[test]
fn orphan_cleanup_respects_helper_locks_and_never_follows_links() {
    let root = tempfile::tempdir().unwrap();
    let external = tempfile::tempdir().unwrap();
    fs::write(external.path().join("keep"), b"user source").unwrap();
    let held = root.path().join("attempt-held");
    let abandoned = root.path().join("attempt-stopped");
    fs::create_dir(&held).unwrap();
    fs::create_dir(&abandoned).unwrap();
    let lock = fs::File::create(held.join(".import.lock")).unwrap();
    lock.lock_exclusive().unwrap();
    fs::File::create(abandoned.join(".import.lock")).unwrap();
    fs::write(abandoned.join("partial"), b"data").unwrap();
    std::os::unix::fs::symlink(external.path(), root.path().join("attempt-link")).unwrap();
    cleanup_abandoned(root.path());
    assert!(held.exists());
    assert!(!abandoned.exists());
    assert!(external.path().join("keep").exists());
    drop(lock);
    cleanup_abandoned(root.path());
    assert!(!held.exists());
}

#[test]
fn completed_download_must_be_a_single_owned_regular_file() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("source.mkv");
    fs::write(&path, b"video").unwrap();
    let mut video = DownloadedVideo {
        path: path.clone(),
        video_id: "NYFGCESmikA".into(),
        title: "Title".into(),
        channel: "Channel".into(),
        duration_seconds: 10.0,
        byte_size: 5,
    };
    assert!(valid_download(root.path(), &video, "NYFGCESmikA"));
    video.duration_seconds = f64::NAN;
    assert!(!valid_download(root.path(), &video, "NYFGCESmikA"));
    video.duration_seconds = 10.0;
    fs::hard_link(&path, root.path().join("other")).unwrap();
    assert!(!valid_download(root.path(), &video, "NYFGCESmikA"));
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(root.path().join("other"), &path).unwrap();
    assert!(!valid_download(root.path(), &video, "NYFGCESmikA"));
}

#[test]
fn project_cleanup_preserves_a_locked_attempt_until_the_helper_exits() {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    let attempt = project.join("import/attempt-live");
    fs::create_dir_all(&attempt).unwrap();
    let lock = fs::File::create(attempt.join(".import.lock")).unwrap();
    lock.lock_exclusive().unwrap();
    fs::write(attempt.join("partial"), b"pending bytes").unwrap();
    cleanup_project(&project);
    assert!(attempt.join("partial").exists());
    drop(lock);
    cleanup_project(&project);
    assert!(!project.exists());
}
