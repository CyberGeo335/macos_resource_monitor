use std::process::Command;

fn escape_osascript_text(text: &str) -> String {
    // Простая экранизация кавычек
    text.replace('"', "\\\"")
}

fn send_osascript_notification(title: &str, subtitle: &str, body: &str) {
    let title = escape_osascript_text(title);
    let subtitle = escape_osascript_text(subtitle);
    let body = escape_osascript_text(body);

    let script = format!(
        "display notification \"{}\" with title \"{}\" subtitle \"{}\"",
        body, title, subtitle
    );

    let _ = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status();
}

#[derive(Clone)]
pub struct Notifier;

impl Notifier {
    pub fn new() -> Self {
        Notifier
    }

    pub fn info(&self, message: &str) {
        send_osascript_notification("MacResourceMonitor", "Info", message);
    }

    pub fn warning(&self, message: &str) {
        send_osascript_notification("MacResourceMonitor", "Warning", message);
    }

    pub fn error(&self, message: &str) {
        send_osascript_notification("MacResourceMonitor", "Error", message);
    }
}
