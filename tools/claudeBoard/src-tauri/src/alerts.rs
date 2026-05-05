use crate::model::TaskStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub notify_completed: bool,
    pub notify_needs_user: bool,
    pub speak_completed: bool,
    pub speak_needs_user: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Announcement {
    pub show_notification: bool,
    pub voice_text: Option<String>,
}

pub fn announcement_for(status: TaskStatus, preferences: &Preferences) -> Option<Announcement> {
    match status {
        TaskStatus::Completed => {
            if !preferences.notify_completed && !preferences.speak_completed {
                return None;
            }

            Some(Announcement {
                show_notification: preferences.notify_completed,
                voice_text: preferences
                    .speak_completed
                    .then(|| "Claude task completed".to_string()),
            })
        }
        TaskStatus::NeedsUser => {
            if !preferences.notify_needs_user && !preferences.speak_needs_user {
                return None;
            }

            Some(Announcement {
                show_notification: preferences.notify_needs_user,
                voice_text: preferences
                    .speak_needs_user
                    .then(|| "Claude task needs your attention".to_string()),
            })
        }
        _ => None,
    }
}
