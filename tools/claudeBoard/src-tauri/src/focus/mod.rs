use crate::model::WindowTarget;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FocusAttempt {
    Pane {
        task_id: String,
        host_kind: String,
        app: String,
        descriptor: String,
        tab_id: String,
        pane_id: String,
    },
    Tab {
        task_id: String,
        host_kind: String,
        app: String,
        descriptor: String,
        tab_id: String,
    },
    AppWindow {
        task_id: String,
        host_kind: String,
        app: String,
        descriptor: String,
    },
}

#[derive(Clone, Debug)]
pub struct FocusRequest {
    pub task_id: String,
    pub window_target: WindowTarget,
}

pub trait HostActivator {
    fn activate(&self, attempt: &FocusAttempt) -> bool;
}

pub fn resolve_focus(activator: &impl HostActivator, request: &FocusRequest) -> bool {
    let target = &request.window_target;

    if let (Some(tab_id), Some(pane_id)) = (target.tab_id.as_ref(), target.pane_id.as_ref()) {
        let attempt = FocusAttempt::Pane {
            task_id: request.task_id.clone(),
            host_kind: target.host_kind.clone(),
            app: target.app.clone(),
            descriptor: target.descriptor.clone(),
            tab_id: tab_id.clone(),
            pane_id: pane_id.clone(),
        };
        if activator.activate(&attempt) {
            return true;
        }
    }

    if let Some(tab_id) = target.tab_id.as_ref() {
        let attempt = FocusAttempt::Tab {
            task_id: request.task_id.clone(),
            host_kind: target.host_kind.clone(),
            app: target.app.clone(),
            descriptor: target.descriptor.clone(),
            tab_id: tab_id.clone(),
        };
        if activator.activate(&attempt) {
            return true;
        }
    }

    let attempt = FocusAttempt::AppWindow {
        task_id: request.task_id.clone(),
        host_kind: target.host_kind.clone(),
        app: target.app.clone(),
        descriptor: target.descriptor.clone(),
    };

    activator.activate(&attempt)
}

pub mod macos;
pub mod windows;
