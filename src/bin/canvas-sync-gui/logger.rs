use std::sync::Mutex;

use log::Record;

pub struct LogBuffer {
    pub logs: Mutex<Vec<LogItem>>,
}

impl LogBuffer {
    pub const fn new() -> Self {
        Self {
            logs: Mutex::new(Vec::new()),
        }
    }
}

pub struct LogItem {
    pub time: std::time::Instant,
    pub level: log::Level,
    pub message: String,
}

impl log::Log for LogBuffer {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        let mut logs = self.logs.lock().unwrap();
        eprintln!("{} - {}", record.level(), record.args());
        logs.push(LogItem {
            time: std::time::Instant::now(),
            level: record.level(),
            message: record.args().to_string(),
        });
    }
    fn flush(&self) {}
}
