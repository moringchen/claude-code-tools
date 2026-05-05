use claude_board::startup_hooks::ensure_hook_setup_in_home;

#[test]
fn daemon_startup_hook_setup_installs_current_hook_events() {
    let temp_home = std::env::temp_dir().join(format!(
        "claude-board-startup-hooks-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp_home);
    std::fs::create_dir_all(&temp_home).unwrap();

    let paths = ensure_hook_setup_in_home(&temp_home).unwrap();
    let settings = std::fs::read_to_string(paths.settings_path).unwrap();

    assert!(settings.contains("UserPromptSubmit"));
    assert!(settings.contains("PostConversationTurn"));
    assert!(settings.contains("PreToolUse"));
    assert!(settings.contains("Stop"));
    assert!(settings.contains(&paths.dispatch_script_path.to_string_lossy().to_string()));

    let _ = std::fs::remove_dir_all(&temp_home);
}

#[test]
fn daemon_startup_hook_setup_preserves_invalid_settings_file() {
    let temp_home = std::env::temp_dir().join(format!(
        "claude-board-invalid-settings-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp_home);
    let claude_dir = temp_home.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    let settings_path = claude_dir.join("settings.json");
    std::fs::write(&settings_path, "{ invalid json").unwrap();

    let result = ensure_hook_setup_in_home(&temp_home);

    assert!(result.is_err());
    assert_eq!(std::fs::read_to_string(settings_path).unwrap(), "{ invalid json");

    let _ = std::fs::remove_dir_all(&temp_home);
}
