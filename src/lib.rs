//! Not just a pretty (inter)face.
//!
//! A pretty-printer and [ndjson](http://ndjson.org/) logger for the [log](https://docs.rs/log) crate.
//!
//! ## Examples
//! ```
//! femme::start();
//! log::warn!("Unauthorized access attempt on /login");
//! log::info!("Listening on port 8080");
//! ```

pub use log::LevelFilter;

use std::{borrow::Cow, collections::HashMap, default::Default};

#[cfg(not(target_arch = "wasm32"))]
mod x86;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub enum Logger {
    #[cfg(not(target_arch = "wasm32"))]
    Pretty,

    #[cfg(not(target_arch = "wasm32"))]
    NDJson,

    #[cfg(target_arch = "wasm32")]
    Wasm,
}

impl Default for Logger {
    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Logger::Wasm
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        Logger::Pretty
    }
}

/// Starts logging depending on current environment.
///
/// # Log output
///
/// - when compiling with `--release` uses ndjson.
/// - pretty-prints otherwise.
/// - works in WASM out of the box.
///
/// # Examples
///
/// ```
/// femme::start();
/// log::warn!("Unauthorized access attempt on /login");
/// log::info!("Listening on port 8080");
/// ```
pub fn start() {
    with_level(LevelFilter::Info)
}

/// Shortcut for building a pretty-printed Logger
#[cfg(not(target_arch = "wasm32"))]
pub fn pretty() -> Femme {
    Femme::default().logger(Logger::Pretty)
}

/// Shortcut for building a ndjson Logger
#[cfg(not(target_arch = "wasm32"))]
pub fn ndjson() -> Femme {
    Femme::default().logger(Logger::NDJson)
}

/// Shortcut for building a ndjson Logger
#[cfg(target_arch = "wasm32")]
pub fn wasm() -> Femme {
    Femme::default().logger(Logger::Wasm)
}

/// Start logging with a log level.
///
/// All messages under the specified log level will statically be filtered out.
///
/// # Examples
/// ```
/// femme::with_level(log::LevelFilter::Trace);
/// ```
pub fn with_level(level: log::LevelFilter) {
    Femme::default()
        .level(level)
        .finish()
        .expect("failed to start logger")
}

pub struct Femme {
    /// Type of logger in use
    logger: Logger,

    /// The default log level
    ///
    /// If a module / crate / target  is not specifically called out
    /// via `level_for ` then this is the level we will log a
    level: LevelFilter,

    /// Per module / crate log levels
    targets: HashMap<Cow<'static, str>, LevelFilter>,
}

impl Default for Femme {
    fn default() -> Self {
        Femme {
            logger: Logger::default(),
            level: LevelFilter::Info,
            targets: HashMap::new(),
        }
    }
}

impl Femme {
    /// Set the type of logger
    ///
    /// Different logger types include `Pretty`, 'NDJson`, or 'Wasm'
    pub fn logger(mut self, logger: Logger) -> Self {
        self.logger = logger;
        self
    }

    /// Set the log level to use
    ///
    /// This is the default log level if a specific one is not defined
    /// for a module / crate using `level_for`
    pub fn level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// Sets a log level for a specific module or crate
    ///
    /// # Arguments
    /// * `module` - The fully-qualified module or crate name
    /// * `level` - The level to log at for this module or crate
    pub fn level_for(mut self, module: impl Into<Cow<'static, str>>, level: LevelFilter) -> Self {
        let module = module.into();
        self.targets
            .entry(module)
            .and_modify(|l| *l = level)
            .or_insert(level);
        self
    }

    /// What level to log at for a given module
    ///
    /// # Arguments
    /// * `record` - The record to extract the module name from
    fn module_level(&self, record: &log::Record) -> &LevelFilter {
        record
            .module_path()
            .and_then(|module| module.split("::").nth(0))
            .and_then(|module| self.targets.get(module))
            .unwrap_or_else(|| &self.level)
    }

    /// Finish building and start the logger
    pub fn finish(self) -> Result<(), log::SetLoggerError> {
        // compute the max log level
        let max_level = std::cmp::max(
            self.level,
            self.targets
                .values()
                .max()
                .unwrap_or_else(|| &LevelFilter::Off)
                .clone(),
        );

        let logger = Box::new(self);
        log::set_boxed_logger(logger)?;
        log::set_max_level(max_level);

        Ok(())
    }
}
