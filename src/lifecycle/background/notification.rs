//! Desktop and system toast notification utilities.
//!
//! **Taxonomy Classification**: Execution State (Lifecycle - Background) + Platform (Native).

#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(all(target_os = "windows", feature = "notification"))]
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Trigger a native Windows Toast Notification (on Windows) or desktop notification (on Linux).
pub fn show_toast_notification(title: &str, message: &str) {
    let exe_name = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()))
        .unwrap_or_else(|| "rApp".to_string());
    let exe_clean = exe_name.strip_suffix(".exe").unwrap_or(&exe_name);
    let app_id = format!("Local76.{}", exe_clean);
    show_toast_notification_with_id(&app_id, title, message);
}

/// Trigger a native notification with a custom App ID (useful for Action Center grouping).
pub fn show_toast_notification_with_id(app_id: &str, title: &str, message: &str) {
    #[cfg(all(target_os = "windows", feature = "notification"))]
    {
        let _ = (|| -> Result<(), Box<dyn std::error::Error>> {
            use windows::Data::Xml::Dom::XmlDocument;
            use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};

            let escaped_title = escape_xml(title);
            let escaped_message = escape_xml(message);
            let toast_xml = format!(
                "<toast><visual><binding template='ToastText02'><text id='1'>{}</text><text id='2'>{}</text></binding></visual></toast>",
                escaped_title, escaped_message
            );
            let doc = XmlDocument::new()?;
            doc.LoadXml(&windows::core::HSTRING::from(toast_xml))?;

            let toast = ToastNotification::CreateToastNotification(&doc)?;
            let notifier = ToastNotificationManager::CreateToastNotifierWithId(&windows::core::HSTRING::from(app_id))?;
            notifier.Show(&toast)?;
            Ok(())
        })();
    }

    #[cfg(all(target_os = "windows", not(feature = "notification")))]
    {
        let _ = (app_id, title, message);
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("notify-send")
            .arg(title)
            .arg(message)
            .status();
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = (title, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escaping() {
        show_toast_notification("<test>&", "\"message'\"");
    }
}
