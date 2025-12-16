use std::process::Command;

pub enum NotificationType {
    Success,
    Error,
    Info,
}

pub struct Notification;

impl Notification {
    pub fn send(notification_type: NotificationType, title: &str, message: &str) {
        let urgency = match notification_type {
            NotificationType::Success => "normal",
            NotificationType::Error => "critical",
            NotificationType::Info => "low",
        };

        let icon = match notification_type {
            NotificationType::Success => "dialog-information",
            NotificationType::Error => "dialog-error",
            NotificationType::Info => "dialog-information",
        };

        // Utiliser notify-send pour les notifications système
        let result = Command::new("notify-send")
            .arg("--urgency")
            .arg(urgency)
            .arg("--icon")
            .arg(icon)
            .arg("--app-name")
            .arg("NDownloader")
            .arg(title)
            .arg(message)
            .spawn();

        if let Err(error) = result {
            tracing::warn!("Failed to send system notification: {}", error);
        }
    }

    pub fn success(title: &str, message: &str) {
        Self::send(NotificationType::Success, title, message);
    }

    pub fn error(title: &str, message: &str) {
        Self::send(NotificationType::Error, title, message);
    }

    pub fn info(title: &str, message: &str) {
        Self::send(NotificationType::Info, title, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_methods_dont_panic() {
        // Vérifie que les méthodes ne paniquent pas
        Notification::success("Test", "Success");
        Notification::error("Test", "Error");
        Notification::info("Test", "Info");
    }
}
