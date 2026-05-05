use claude_board::{
    alerts::{announcement_for, Preferences},
    model::TaskStatus,
};

fn preferences(
    notify_completed: bool,
    notify_needs_user: bool,
    speak_completed: bool,
    speak_needs_user: bool,
) -> Preferences {
    Preferences {
        notify_completed,
        notify_needs_user,
        speak_completed,
        speak_needs_user,
    }
}

#[test]
fn completed_notify_only_shows_notification_without_voice() {
    let announcement = announcement_for(
        TaskStatus::Completed,
        &preferences(true, false, false, false),
    )
    .unwrap();

    assert!(announcement.show_notification);
    assert_eq!(announcement.voice_text, None);
}

#[test]
fn completed_speak_only_sets_voice_without_notification() {
    let announcement = announcement_for(
        TaskStatus::Completed,
        &preferences(false, false, true, false),
    )
    .unwrap();

    assert!(!announcement.show_notification);
    assert_eq!(
        announcement.voice_text.as_deref(),
        Some("Claude task completed")
    );
}

#[test]
fn completed_neither_returns_none() {
    let announcement = announcement_for(
        TaskStatus::Completed,
        &preferences(false, false, false, false),
    );

    assert_eq!(announcement, None);
}

#[test]
fn needs_user_notify_only_shows_notification_without_voice() {
    let announcement = announcement_for(
        TaskStatus::NeedsUser,
        &preferences(false, true, false, false),
    )
    .unwrap();

    assert!(announcement.show_notification);
    assert_eq!(announcement.voice_text, None);
}

#[test]
fn needs_user_speak_only_sets_voice_without_notification() {
    let announcement = announcement_for(
        TaskStatus::NeedsUser,
        &preferences(false, false, false, true),
    )
    .unwrap();

    assert!(!announcement.show_notification);
    assert_eq!(
        announcement.voice_text.as_deref(),
        Some("Claude task needs your attention")
    );
}

#[test]
fn needs_user_neither_returns_none() {
    let announcement = announcement_for(
        TaskStatus::NeedsUser,
        &preferences(false, false, false, false),
    );

    assert_eq!(announcement, None);
}

#[test]
fn both_speaking_statuses_keep_distinct_voice_text() {
    let preferences = preferences(true, true, true, true);

    let completed = announcement_for(TaskStatus::Completed, &preferences).unwrap();
    let needs_user = announcement_for(TaskStatus::NeedsUser, &preferences).unwrap();

    assert_eq!(
        completed.voice_text.as_deref(),
        Some("Claude task completed")
    );
    assert_eq!(
        needs_user.voice_text.as_deref(),
        Some("Claude task needs your attention")
    );
    assert_ne!(completed.voice_text, needs_user.voice_text);
    assert!(completed.show_notification);
    assert!(needs_user.show_notification);
}
