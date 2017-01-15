//! This tiny crate checks that the running or installed `rustc` meets some
//! version requirements. The version is queried by calling the Rust compiler
//! with `--version`. The path to the compiler is determined first via the
//! `RUSTC` environment variable. If it is not set, then `rustc` is used. If
//! that fails, no determination is made, and calls return `None`.
//!
//! # Example
//!
//! Check that the running compiler is a nightly release:
//!
//! ```rust
//! extern crate version_check;
//!
//! match version_check::is_nightly() {
//!     Some(true) => "running a nightly",
//!     Some(false) => "not nightly",
//!     None => "couldn't figure it out"
//! };
//! ```
//!
//! Check that the running compiler is at least version `1.13.0`:
//!
//! ```rust
//! extern crate version_check;
//!
//! match version_check::is_min_version("1.13.0") {
//!     Some((true, version)) => format!("Yes! It's: {}", version),
//!     Some((false, version)) => format!("No! {} is too old!", version),
//!     None => "couldn't figure it out".into()
//! };
//! ```
//!
//! Check that the running compiler was released on or after `2016-12-18`:
//!
//! ```rust
//! extern crate version_check;
//!
//! match version_check::is_min_date("2016-12-18") {
//!     Some((true, date)) => format!("Yes! It's: {}", date),
//!     Some((false, date)) => format!("No! {} is too long ago!", date),
//!     None => "couldn't figure it out".into()
//! };
//! ```
//!
//! # Alternatives
//!
//! This crate is dead simple with no dependencies. If you need something more
//! and don't care about panicking if the version cannot be obtained or adding
//! dependencies, see [rustc_version](https://crates.io/crates/rustc_version).

use std::env;
use std::process::Command;

// Convert a string of %Y-%m-%d to a single u32 maintaining ordering.
fn str_to_ymd(ymd: &str) -> Option<u32> {
    let ymd: Vec<u32> = ymd.split("-").filter_map(|s| s.parse::<u32>().ok()).collect();
    if ymd.len() != 3 {
        return None
    }

    let (y, m, d) = (ymd[0], ymd[1], ymd[2]);
    Some((y << 9) | (m << 5) | d)
}

// Convert a string with prefix major-minor-patch to a single u64 maintaining
// ordering. Assumes none of the components are > 1048576.
fn str_to_mmp(mmp: &str) -> Option<u64> {
    let mmp: Vec<u16> = mmp.split('-')
        .nth(0)
        .unwrap_or("")
        .split('.')
        .filter_map(|s| s.parse::<u16>().ok())
        .collect();

    if mmp.len() != 3 {
        return None
    }

    let (maj, min, patch) = (mmp[0] as u64, mmp[1] as u64, mmp[2] as u64);
    Some((maj << 32) | (min << 16) | patch)
}

fn get_version_and_date() -> Option<(String, String)> {
    let output = env::var("RUSTC").ok()
        .and_then(|rustc| Command::new(rustc).arg("--version").output().ok())
        .or_else(|| Command::new("rustc").arg("--version").output().ok())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| {
            let mut components = s.split(" ");
            let version = components.nth(1);
            let date = components.nth(1).map(|s| s.trim_right().trim_right_matches(")"));
            (version.map(|s| s.to_string()), date.map(|s| s.to_string()))
        });

    match output {
        Some((Some(version), Some(date))) => Some((version, date)),
        _ => None
    }
}

/// Checks that the running or installed `rustc` was released no earlier than
/// some date.
///
/// The format of `min_date` must be YYYY-MM-DD. For instance: `2016-12-20` or
/// `2017-01-09`.
///
/// If the date cannot be retrieved or parsed, or if `min_date` could not be
/// parsed, returns `None`. Otherwise returns a tuple where the first value is
/// `true` if the installed `rustc` is at least from `min_data` and the second
/// value is the date (in YYYY-MM-DD) of the installed `rustc`.
pub fn is_min_date(min_date: &str) -> Option<(bool, String)> {
    if let Some((_, actual_date_str)) = get_version_and_date() {
        str_to_ymd(&actual_date_str)
            .and_then(|actual| str_to_ymd(min_date).map(|min| (min, actual)))
            .map(|(min, actual)| (actual >= min, actual_date_str))
    } else {
        None
    }
}

/// Checks that the running or installed `rustc` is at least some minimum
/// version.
///
/// The format of `min_version` is a semantic version: `1.15.0-beta`, `1.14.0`,
/// `1.16.0-nightly`, etc.
///
/// If the version cannot be retrieved or parsed, or if `min_version` could not
/// be parsed, returns `None`. Otherwise returns a tuple where the first value
/// is `true` if the installed `rustc` is at least `min_version` and the second
/// value is the version (semantic) of the installed `rustc`.
pub fn is_min_version(min_version: &str) -> Option<(bool, String)> {
    if let Some((actual_version_str, _)) = get_version_and_date() {
        str_to_mmp(&actual_version_str)
            .and_then(|actual| str_to_mmp(min_version).map(|min| (min, actual)))
            .map(|(min, actual)| (actual >= min, actual_version_str))
    } else {
        None
    }
}

/// Determines whether the running or installed `rustc` is on the nightly
/// channel.
///
/// If the version could not be determined, returns `None`. Otherwise returns
/// `Some(true)` if the running version is a nightly release, and `Some(false)`
/// otherwise.
pub fn is_nightly() -> Option<bool> {
    get_version_and_date()
        .map(|(actual_version_str, _)| actual_version_str.contains("nightly"))
}

/// Determines whether the running or installed `rustc` is on the beta channel.
///
/// If the version could not be determined, returns `None`. Otherwise returns
/// `Some(true)` if the running version is a beta release, and `Some(false)`
/// otherwise.
pub fn is_beta() -> Option<bool> {
    get_version_and_date()
        .map(|(actual_version_str, _)| actual_version_str.contains("beta"))
}
