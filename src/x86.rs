//! Pretty print logs.

use crate::{Femme, Logger};
use log::{kv, Level, Log, Metadata, Record};
use std::io::{self, StdoutLock, Write};

// ANSI term codes.
const RESET: &'static str = "\x1b[0m";
const BOLD: &'static str = "\x1b[1m";
const RED: &'static str = "\x1b[31m";
const GREEN: &'static str = "\x1b[32m";
const YELLOW: &'static str = "\x1b[33m";

/// Format Key/Value pairs that have been passed to a `Log` macro (such as `info!`)
///
/// # Arguments
/// * `handle` - Exclusive handle to `stdout`
/// * `record` - Record to write
fn format_kv_pairs<'b>(mut handle: &mut StdoutLock<'b>, record: &Record) {
    struct Visitor<'a, 'b> {
        stdout: &'a mut StdoutLock<'b>,
    }

    impl<'kvs, 'a, 'b> kv::Visitor<'kvs> for Visitor<'a, 'b> {
        fn visit_pair(
            &mut self,
            key: kv::Key<'kvs>,
            val: kv::Value<'kvs>,
        ) -> Result<(), kv::Error> {
            write!(self.stdout, "\n    {}{}{} {}", BOLD, key, RESET, val).unwrap();
            Ok(())
        }
    }

    let mut visitor = Visitor {
        stdout: &mut handle,
    };
    record.key_values().visit(&mut visitor).unwrap();
}

/// Uses a pretty-print format to print to stdout
///
/// # Arguments
/// * `handle` - Exclusive handle to `stdout`
/// * `record` - Record to write
fn write_pretty(handle: &mut StdoutLock, record: &Record) {
    // Format lines
    let msg = record.target();
    match record.level() {
        Level::Trace | Level::Debug | Level::Info => {
            write!(handle, "{}{}{}{}", GREEN, BOLD, msg, RESET).unwrap();
        }
        Level::Warn => write!(handle, "{}{}{}{}", YELLOW, BOLD, msg, RESET).unwrap(),
        Level::Error => write!(handle, "{}{}{}{}", RED, BOLD, msg, RESET).unwrap(),
    }
    write!(handle, " {}", record.args()).unwrap();

    // Format Key/Value pairs
    format_kv_pairs(handle, record);
    writeln!(handle, "").unwrap();
}

/// Uses a pretty-print format to print to stdout using the
/// Newline Delimited JSON format
///
/// # Arguments
/// * `handle` - Exclusive handle to `stdout`
/// * `record` - Record to write
fn write_ndjson(handle: &mut StdoutLock, record: &Record) {
    fn get_level(level: log::Level) -> u8 {
        use log::Level::*;
        match level {
            Trace => 10,
            Debug => 20,
            Info => 30,
            Warn => 40,
            Error => 50,
        }
    }

    write!(handle, "{}", '{').unwrap();
    write!(handle, "\"level\":{}", get_level(record.level())).unwrap();

    let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_millis();

    write!(handle, ",\"time\":{}", now).unwrap();
    write!(handle, ",\"msg\":\"{}\"", record.args()).unwrap();

    format_kv_pairs(handle, record);
    writeln!(handle, "{}", "}").unwrap();
}

impl Log for Femme {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record<'_>) {
        let level = self.module_level(record);

        if record.level() <= *level {
            // acquire stdout lock
            let stdout = io::stdout();
            let mut handle = stdout.lock();

            match self.logger {
                Logger::Pretty => write_pretty(&mut handle, &record),
                Logger::NDJson => write_ndjson(&mut handle, &record),
            }
        }
    }
    fn flush(&self) {}
}
