use claude_board::macos_window_behavior::{
    macos_window_collection_behavior, macos_window_level_for_mode, OverlayZOrderMode,
    MACOS_BACKGROUND_WINDOW_LEVEL, MACOS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES,
    MACOS_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY, MACOS_COLLECTION_BEHAVIOR_STATIONARY,
    MACOS_FOREGROUND_WINDOW_LEVEL,
};

#[test]
fn foreground_overlay_uses_floating_window_level() {
    assert_eq!(MACOS_FOREGROUND_WINDOW_LEVEL, 5);
    assert_eq!(
        macos_window_level_for_mode(OverlayZOrderMode::Foreground),
        MACOS_FOREGROUND_WINDOW_LEVEL
    );
}

#[test]
fn background_overlay_uses_normal_window_level() {
    assert_eq!(MACOS_BACKGROUND_WINDOW_LEVEL, 0);
    assert_eq!(
        macos_window_level_for_mode(OverlayZOrderMode::Background),
        MACOS_BACKGROUND_WINDOW_LEVEL
    );
}

#[test]
fn dock_icon_is_visible_so_clicking_it_can_reopen_claudeboard() {
    let config = include_str!("../tauri.conf.json");
    assert!(config.contains("\"skipTaskbar\": false"));
}

#[test]
fn overlay_window_joins_all_spaces_and_stays_stationary() {
    assert_eq!(MACOS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES, 1 << 0);
    assert_eq!(MACOS_COLLECTION_BEHAVIOR_STATIONARY, 1 << 4);
    assert_eq!(MACOS_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY, 1 << 8);
    assert_eq!(
        macos_window_collection_behavior(),
        MACOS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES
            | MACOS_COLLECTION_BEHAVIOR_STATIONARY
            | MACOS_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
    );
}
