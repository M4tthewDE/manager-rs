pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<i32> for LogLevel {
    fn from(level: i32) -> Self {
        match level {
            0 => Self::Trace,
            1 => Self::Debug,
            2 => Self::Info,
            3 => Self::Warn,
            4 => Self::Error,
            _ => panic!("oops"),
        }
    }
}

pub struct LogLine {
    pub level: LogLevel,
    pub text: String,
}

#[derive(Default)]
pub struct ServerLog {
    pub logs: Vec<LogLine>,
}

impl ServerLog {
    pub fn push(&mut self, log_line: LogLine) {
        self.logs.push(log_line);
    }
}
