#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(target_os = "windows")]
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Trigger a native Windows Toast Notification (on Windows) or desktop notification (on Linux).
pub fn show_toast_notification(title: &str, message: &str) {
    #[cfg(target_os = "windows")]
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
            let notifier = ToastNotificationManager::CreateToastNotifierWithId(&windows::core::HSTRING::from("tourian.dynamics.rtemplate"))?;
            notifier.Show(&toast)?;
            Ok(())
        })();
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
