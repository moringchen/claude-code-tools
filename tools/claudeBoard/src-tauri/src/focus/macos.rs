use super::{FocusAttempt, PlatformCommand};

pub fn command_for(attempt: &FocusAttempt) -> PlatformCommand {
    let (app, note) = match attempt {
        FocusAttempt::Pane {
            app,
            descriptor,
            tab_id,
            pane_id,
            ..
        } => (
            app,
            format!(
                "-- descriptor={} tab={} pane={}",
                descriptor, tab_id, pane_id
            ),
        ),
        FocusAttempt::Tab {
            app,
            descriptor,
            tab_id,
            ..
        } => (app, format!("-- descriptor={} tab={}", descriptor, tab_id)),
        FocusAttempt::AppWindow {
            app, descriptor, ..
        } => (app, format!("-- descriptor={}", descriptor)),
    };

    PlatformCommand {
        program: "osascript".into(),
        args: vec![
            "-e".into(),
            format!(
                "tell application {} to activate",
                applescript_string_literal(app)
            ),
            "-e".into(),
            note,
        ],
    }
}

fn applescript_string_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
