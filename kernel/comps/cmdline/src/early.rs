// SPDX-License-Identifier: MPL-2.0

//! Parses a subset of the kernel command line before the heap is available.
//!
//! OSTD calls the function registered with [`ostd::early_cmdline_parser`] during
//! bootstrap, before serial and logging are initialized. The full cmdline parser
//! in the parent crate runs later, once the heap and component system are ready.

use ostd::{boot::EarlyCmdline, log::LevelFilter};

/// Kernel command-line keys consumed by [`early_cmdline_parser`].
///
/// These are stripped during dispatch so they are not forwarded to init.
const EARLY_PARAMS: &[&str] = &["earlycon", "loglevel"];

/// Returns whether `normalized_key` is handled only by the early parser.
pub(super) fn is_early_param(normalized_key: &str) -> bool {
    EARLY_PARAMS.contains(&normalized_key)
}

/// Registers the kernel's early command-line parser with OSTD.
///
/// Scans space-separated tokens in `cmdline` and returns [`EarlyCmdline`] for
/// OSTD to configure the early serial console and log filter.
///
/// # Recognized tokens
///
/// - `earlycon` — enables the early UART console (exact token match).
/// - `loglevel=N` — sets the OSTD log filter (`N` in `0..=8`, see [`LevelFilter`]).
///
/// Other tokens are ignored. When `loglevel` is absent, the default is `8`
/// ([`LevelFilter::Debug`]). When `earlycon` is absent, the early console stays disabled.
#[ostd::early_cmdline_parser]
const fn early_cmdline_parser(cmdline: &str) -> EarlyCmdline {
    let mut has_early_console = false;
    let mut log_level = LevelFilter::Debug;

    let bytes = cmdline.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index] == b' ' {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }

        let start = index;
        while index < bytes.len() && bytes[index] != b' ' {
            index += 1;
        }
        let end = index;

        if match_token(bytes, start, end, b"earlycon") {
            has_early_console = true;
        } else if starts_with_at(bytes, start, end, b"loglevel=") {
            log_level = parse_loglevel_at(bytes, start + 9, end);
        }
    }

    EarlyCmdline {
        log_level,
        has_early_console,
    }
}

/// Returns whether `bytes[start..end]` equals `target` byte-for-byte.
///
/// Requires equal lengths, so tokens such as `earlyconxxxx` do not match `earlycon`.
const fn match_token(bytes: &[u8], start: usize, end: usize, target: &[u8]) -> bool {
    if end - start != target.len() {
        return false;
    }
    starts_with_at(bytes, start, end, target)
}

/// Returns whether `bytes[start..end]` starts with `prefix`.
const fn starts_with_at(bytes: &[u8], start: usize, end: usize, prefix: &[u8]) -> bool {
    if end - start < prefix.len() {
        return false;
    }
    let mut index = 0;
    while index < prefix.len() {
        if bytes[start + index] != prefix[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// Parses a decimal log level from the suffix of a `loglevel=` token.
///
/// Reads consecutive ASCII digits starting at `start` and stops at the first
/// non-digit or at `end`. Returns [`LevelFilter::Debug`] when no digits are
/// present or the value exceeds `8`, consistent with [`LevelFilter::from_u8`].
const fn parse_loglevel_at(bytes: &[u8], mut start: usize, end: usize) -> LevelFilter {
    let mut result: u8 = 0;
    let mut has_digits = false;

    while start < end {
        let byte = bytes[start];
        if byte.is_ascii_digit() {
            let digit = byte - b'0';
            result = result.saturating_mul(10).saturating_add(digit);
            has_digits = true;
        } else {
            break;
        }
        start += 1;
    }

    if !has_digits || result > 8 {
        LevelFilter::Debug
    } else {
        LevelFilter::from_u8(result)
    }
}

#[cfg(ktest)]
mod tests {
    use ostd::prelude::*;

    use super::*;

    #[ktest]
    fn is_early_param_recognizes_early_keys() {
        assert!(is_early_param("earlycon"));
        assert!(is_early_param("loglevel"));
        assert!(!is_early_param("log_level"));
        assert!(!is_early_param("unknown"));
    }

    #[ktest]
    fn early_cmdline_parser_defaults() {
        let result = early_cmdline_parser("");
        assert_eq!(result.log_level, LevelFilter::Debug);
        assert!(!result.has_early_console);
    }

    #[ktest]
    fn early_cmdline_parser_earlycon() {
        assert!(early_cmdline_parser("earlycon").has_early_console);
        assert!(!early_cmdline_parser("earlyconxxxx").has_early_console);
    }

    #[ktest]
    fn early_cmdline_parser_loglevel() {
        assert_eq!(
            early_cmdline_parser("loglevel=0").log_level,
            LevelFilter::Off
        );
        assert_eq!(
            early_cmdline_parser("loglevel=4").log_level,
            LevelFilter::Error
        );
        assert_eq!(
            early_cmdline_parser("loglevel=3x").log_level,
            LevelFilter::Crit
        );
        assert_eq!(
            early_cmdline_parser("loglevel=9").log_level,
            LevelFilter::Debug
        );
    }

    #[ktest]
    fn early_cmdline_parser_last_loglevel_wins() {
        assert_eq!(
            early_cmdline_parser("loglevel=4 loglevel=0").log_level,
            LevelFilter::Off
        );
    }
}
