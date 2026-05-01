use claude_task_window::{
    focus::{macos, resolve_focus, windows, FocusAttempt, FocusRequest, HostActivator},
    model::WindowTarget,
};
use std::cell::RefCell;

#[derive(Default)]
struct StubActivator {
    attempts: RefCell<Vec<FocusAttempt>>,
}

impl HostActivator for StubActivator {
    fn activate(&self, attempt: &FocusAttempt) -> bool {
        self.attempts.borrow_mut().push(attempt.clone());
        matches!(attempt, FocusAttempt::AppWindow { .. })
    }
}

#[test]
fn resolve_focus_tries_pane_then_tab_then_app_window() {
    let activator = StubActivator::default();
    let request = FocusRequest {
        task_id: "task-8".into(),
        window_target: WindowTarget {
            host_kind: "tmux".into(),
            app: "Ghostty".into(),
            descriptor: "dev".into(),
            tab_id: Some("dev".into()),
            pane_id: Some("1.2".into()),
        },
    };

    let focused = resolve_focus(&activator, &request);

    assert!(focused);
    assert_eq!(
        activator.attempts.borrow().as_slice(),
        &[
            FocusAttempt::Pane {
                task_id: "task-8".into(),
                host_kind: "tmux".into(),
                app: "Ghostty".into(),
                descriptor: "dev".into(),
                tab_id: "dev".into(),
                pane_id: "1.2".into(),
            },
            FocusAttempt::Tab {
                task_id: "task-8".into(),
                host_kind: "tmux".into(),
                app: "Ghostty".into(),
                descriptor: "dev".into(),
                tab_id: "dev".into(),
            },
            FocusAttempt::AppWindow {
                task_id: "task-8".into(),
                host_kind: "tmux".into(),
                app: "Ghostty".into(),
                descriptor: "dev".into(),
            },
        ]
    );
}

#[test]
fn macos_command_uses_separate_program_and_escaped_applescript_args() {
    let command = macos::command_for(&FocusAttempt::AppWindow {
        task_id: "task-8".into(),
        host_kind: "terminal".into(),
        app: "Ghost\\\"ty".into(),
        descriptor: "main".into(),
    });

    assert_eq!(command.program, "osascript");
    assert_eq!(command.args[0], "-e");
    assert_eq!(
        command.args[1],
        "tell application \"Ghost\\\\\\\"ty\" to activate"
    );
}

#[test]
fn windows_command_uses_separate_program_and_escaped_powershell_args() {
    let command = windows::command_for(&FocusAttempt::AppWindow {
        task_id: "task-8".into(),
        host_kind: "terminal".into(),
        app: "Ghost'ty".into(),
        descriptor: "dev's".into(),
    });

    assert_eq!(command.program, "powershell");
    assert_eq!(command.args[0], "-NoProfile");
    assert_eq!(command.args[1], "-Command");
    assert_eq!(
        command.args[2],
        "Write-Output 'activate Ghost''ty descriptor=dev''s'"
    );
}
