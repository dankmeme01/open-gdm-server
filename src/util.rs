use chrono::Local;
use colored::Colorize;
use log::Level;

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        if !metadata.target().starts_with("open_gdm_server") {
            metadata.level() <= Level::Warn
        } else if cfg!(debug_assertions) {
            true
        } else {
            metadata.level() <= Level::Info
        }
    }

    fn log(&self, record: &log::Record) {
        let time = Local::now();
        let formatted_time = time.format("%Y-%m-%d %H:%M:%S%.3f");
        if self.enabled(record.metadata()) {
            let (level, args) = match record.level() {
                Level::Error => (
                    record.level().to_string().bright_red(),
                    record.args().to_string().bright_red(),
                ),
                Level::Warn => (
                    record.level().to_string().bright_yellow(),
                    record.args().to_string().bright_yellow(),
                ),
                Level::Info => (
                    record.level().to_string().cyan(),
                    record.args().to_string().cyan(),
                ),
                Level::Debug => (
                    record.level().to_string().normal(),
                    record.args().to_string().normal(),
                ),
                Level::Trace => (
                    record.level().to_string().black(),
                    record.args().to_string().black(),
                ),
            };

            println!(
                "[{}] [{}] - {}",
                formatted_time,
                level,
                args,
            )
        }
    }

    fn flush(&self) {}
}
