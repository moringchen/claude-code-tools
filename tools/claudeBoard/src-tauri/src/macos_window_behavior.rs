#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacosWindowBehavior {
    pub focusable: bool,
    pub visible_on_all_workspaces: bool,
    pub always_on_bottom: bool,
    pub always_on_top: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayZOrderMode {
    Background,
    Foreground,
}

impl OverlayZOrderMode {
    pub fn opposite(self) -> Self {
        match self {
            Self::Background => Self::Foreground,
            Self::Foreground => Self::Background,
        }
    }
}

pub fn overlay_behavior_for_mode(mode: OverlayZOrderMode) -> MacosWindowBehavior {
    match mode {
        OverlayZOrderMode::Background => background_overlay_behavior(),
        OverlayZOrderMode::Foreground => foreground_overlay_behavior(),
    }
}

pub fn background_overlay_behavior() -> MacosWindowBehavior {
    MacosWindowBehavior {
        focusable: true,
        visible_on_all_workspaces: true,
        always_on_bottom: true,
        always_on_top: false,
    }
}

pub fn foreground_overlay_behavior() -> MacosWindowBehavior {
    MacosWindowBehavior {
        focusable: true,
        visible_on_all_workspaces: true,
        always_on_bottom: false,
        always_on_top: true,
    }
}
