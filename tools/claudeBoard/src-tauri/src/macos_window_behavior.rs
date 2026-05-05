pub const MACOS_FOREGROUND_WINDOW_LEVEL: i32 = 5;
pub const MACOS_BACKGROUND_WINDOW_LEVEL: i32 = 0;
pub const MACOS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
pub const MACOS_COLLECTION_BEHAVIOR_STATIONARY: u64 = 1 << 4;
pub const MACOS_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY: u64 = 1 << 8;

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

pub fn macos_window_level_for_mode(mode: OverlayZOrderMode) -> i32 {
    match mode {
        OverlayZOrderMode::Background => MACOS_BACKGROUND_WINDOW_LEVEL,
        OverlayZOrderMode::Foreground => MACOS_FOREGROUND_WINDOW_LEVEL,
    }
}

pub fn macos_window_collection_behavior() -> u64 {
    MACOS_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES
        | MACOS_COLLECTION_BEHAVIOR_STATIONARY
        | MACOS_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY
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
