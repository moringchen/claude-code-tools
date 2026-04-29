use super::{FocusAttempt, PlatformCommand};

pub fn command_for(attempt: &FocusAttempt) -> PlatformCommand {
    let script = match attempt {
        FocusAttempt::Pane {
            app,
            descriptor,
            tab_id,
            pane_id,
            ..
        } => format!(
            "Write-Output 'activate {} descriptor={} tab={} pane={}'",
            powershell_single_quoted_literal(app),
            powershell_single_quoted_literal(descriptor),
            powershell_single_quoted_literal(tab_id),
            powershell_single_quoted_literal(pane_id)
        ),
        FocusAttempt::Tab {
            app,
            descriptor,
            tab_id,
            ..
        } => format!(
            "Write-Output 'activate {} descriptor={} tab={}'",
            powershell_single_quoted_literal(app),
            powershell_single_quoted_literal(descriptor),
            powershell_single_quoted_literal(tab_id)
        ),
        FocusAttempt::AppWindow {
            app, descriptor, ..
        } => format!(
            "Write-Output 'activate {} descriptor={}'",
            powershell_single_quoted_literal(app),
            powershell_single_quoted_literal(descriptor)
        ),
    };

    PlatformCommand {
        program: "powershell".into(),
        args: vec!["-NoProfile".into(), "-Command".into(), script],
    }
}

fn powershell_single_quoted_literal(value: &str) -> String {
    value.replace('\'', "''")
}
