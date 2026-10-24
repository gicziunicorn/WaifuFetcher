#[warn(unused)]
use std::fmt::Write;
use anyhow::{Context, Result};
use tracing::{Level, Subscriber, field::{Field, Visit}};
use tracing_appender::non_blocking::WorkerGuard;
use std::{fs, io};
use dirs::{data_local_dir, home_dir};
use time::{UtcOffset, macros::format_description as timefmt};
use tracing_subscriber::{EnvFilter, Layer, fmt::{self, FormatEvent, FormatFields, FormattedFields, time::OffsetTime}, layer::SubscriberExt, registry::LookupSpan, util::SubscriberInitExt};

use crate::APP_NAME;


// Custom formatter (instead of doing something better)
// I wanted the terminal logs to look nice so I implemented my own formatter
struct CustomFormatter {
    ansi: bool,
    time_format: &'static str
}

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result
    {
        // ANSI colors
        let black = "\x1b[38;5;0m";
        let white = "\x1b[38;5;15m";
        let gray = "\x1b[38;5;251m";
        let lightgray = "\x1b[38;5;253m";
        let blue = "\x1b[38;5;12m";
        let red = "\x1b[38;5;9m";
        let yellow = "\x1b[38;5;3m";
        let green = "\x1b[38;5;64m";

        let timestamp = chrono::Local::now()
            .format(self.time_format);

        let metadata = event.metadata();
        let level = metadata.level();

        let level_color = match *level {
            Level::DEBUG => green,
            Level::INFO => blue,
            Level::WARN => yellow,
            Level::ERROR => red,
            Level::TRACE => gray,
        };
        if self.ansi {
            write!(&mut writer, "{gray}{timestamp} {level_color}{}{} {gray}{}: ", level, " ".repeat(5-level.to_string().len()), metadata.target())?;
            write!(writer, "{white}")?;
        } else {
            write!(&mut writer, "{timestamp} {}{} {}: ", level, " ".repeat(5-level.to_string().len()), metadata.target())?;
        }

        let mut visitor = CustomVisitor {
            data: &mut String::new(),
            level: metadata.level(),
            ansi: self.ansi,
        };
        event.record(&mut visitor);

        writeln!(writer, "{}", visitor.data)
    }
}

// Custom visitor to print fields
struct CustomVisitor<'a> {
    data: &'a mut String,
    level: &'a Level,
    ansi: bool,
}

impl<'a> Visit for CustomVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let debug_str = format!("{value:?}");
        let formatted = debug_str.strip_circumfix("\"", "\"").unwrap_or(&debug_str);
        if formatted == "" { return; }

        let mut lines = formatted.lines();
        let mut message = lines.next().unwrap_or_default().to_string();
        for line in lines {
            if self.ansi {
                write!(message, "\n            \x1b[38;5;240m|\x1b[38;5;15m   {line}");
            } else {
                write!(message, "\n            |   {line}");
            }
        }


        match *self.level {
            Level::ERROR => {
                // The "main" field
                if field.name() == "error" {
                    write!(self.data, "{message}");
                }
                else if field.name() == "causes" {
                    let causes = message.split("|");
                    for cause in causes {
                        self.data.push_str(&format!("\n\t\tCaused by: {cause}"));
                    }
                    self.data.push_str("\n");
                }
                else {
                    self.data.push_str(&format!("\n\t\t{}: {message}", field.name()));
                }
            },
            _ => {
                // The "main" field
                if field.name() == "message" {
                    write!(self.data, "{message}");
                }
                // Include other fields
                else {
                    self.data.push_str(&format!("\n\t{}: {message}", field.name()));
                }
            }
        }
    }
}



pub fn setup_logger() -> Result<Option<WorkerGuard>> {
    // Get the local time offset for formatting since of course that's my job too
    let local_offset = UtcOffset::current_local_offset()
        .unwrap_or(UtcOffset::UTC);

    // Set 'error' level for crates, 'debug/warn' for this app
    let stdout_filter = EnvFilter::try_from_default_env() // allow user to set the filter in via an env variable
        .unwrap_or_else(|_| {
            // Stdout will have a different log level in debug mode than in release mode
            if cfg!(debug_assertions) {
                EnvFilter::new("error,WaifuFetcher=debug")
            } else {
                EnvFilter::new("error,WaifuFetcher=info")
            }
        });

    let terminal_formatter = CustomFormatter {
        ansi: true,
        time_format: "%H:%M:%S",
    };
    // Create a the stdout layer with a nice format
    let stdout_layer = fmt::layer()
        .with_timer(OffsetTime::new(local_offset, timefmt!("[hour]:[minute]:[second]")))
        .event_format(terminal_formatter)
        .with_filter(stdout_filter);

    // Set the path for the log file
    let log_path = data_local_dir()
        .or_else(home_dir)
        .map(|path| path.join(APP_NAME));

    // Create the file layer and organize log files if the path was found (the layer can be None, tracing ignores it)
    // Also keep _guard alive, so logs will be flushed when the app exits
    let (file_layer, _guard) = if let Some(log_path) = log_path {
        // Try renaming the previous config to previous.log
        let mv_result = fs::rename(log_path.join("latest.log"), log_path.join("previous.log"));
        if let Err(err) = mv_result && err.kind() != io::ErrorKind::NotFound {
            // Or try deleting the previous log (because the FileAppender doesn't overwrite it)
            fs::remove_file(log_path.join("latest.log"))
                .context("Failed to backup or remove previous log")?;
        };

        let timer_format = OffsetTime::new(local_offset, timefmt!("[year]-[month]-[day] [hour]:[minute]:[second]"));

        // A non-rolling file appender (meaning it won't create backup)
        // (since the backup is handled already)
        let file_appender = tracing_appender::rolling::never(log_path, "latest.log");
        let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

        // The formatter
        let file_formatter = CustomFormatter {
            ansi: false,
            time_format: "%Y-%M-%d %H:%M:%S",
        };

        // Construct the file layer
        let layer = fmt::layer()
            .with_writer(file_writer)
            .with_timer(timer_format)
            .event_format(file_formatter)
            .with_filter(EnvFilter::new("error,WaifuFetcher=info"));

        ( Some(layer), Some(_guard) )
    }
    else { (None, None) };

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // Return the guard to keep it alive
    return Ok(_guard);
}