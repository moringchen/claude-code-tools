use claude_board::sound::sound_path_for_type;

#[test]
fn maps_waiting_and_completed_sound_paths() {
    assert_eq!(
        sound_path_for_type("waiting"),
        Some("/Users/moringchen/Downloads/待回复.mp3")
    );
    assert_eq!(
        sound_path_for_type("completed"),
        Some("/Users/moringchen/Downloads/任务完成.mp3")
    );
}

#[test]
fn rejects_unknown_sound_type() {
    assert_eq!(sound_path_for_type("other"), None);
}
