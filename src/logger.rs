// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

use std::sync::atomic::{AtomicU8, Ordering};

static LOG_LEVEL: AtomicU8 = AtomicU8::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Error = 2,
}

impl LogLevel {
    fn from_u8(val: u8) -> Self {
        match val {
            0 => LogLevel::Debug,
            2 => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

pub fn init_log_level() {
    let level_str = std::env::var("RMECHREVO_LOG_LEVEL").unwrap_or_default();
    let level = match level_str.to_uppercase().as_str() {
        "DEBUG" => 0,
        "ERROR" => 2,
        _ => 1,
    };
    LOG_LEVEL.store(level, Ordering::Relaxed);
}

pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn current_log_level() -> LogLevel {
    LogLevel::from_u8(LOG_LEVEL.load(Ordering::Relaxed))
}

pub fn should_log(level: LogLevel) -> bool {
    level >= current_log_level()
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if $crate::logger::should_log($crate::logger::LogLevel::Debug) {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if $crate::logger::should_log($crate::logger::LogLevel::Info) {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if $crate::logger::should_log($crate::logger::LogLevel::Error) {
            eprintln!($($arg)*);
        }
    };
}
