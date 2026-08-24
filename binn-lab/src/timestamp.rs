//! When a cell was produced, recorded in the cell itself.
//!
//! # Why a cell needs this
//!
//! The campaign's central claim is that a rule was registered *before* the data
//! it governs existed. Until now a cell could not attest to that: the schema
//! carried `wall_secs`, a **duration**, and nothing saying *when*. The ordering
//! therefore rested on S3 upload times and git commit times — evidence outside
//! the artefact, and evidence that disappears when a bucket is emptied.
//!
//! That is not hypothetical. An audit on 2026-08-23 found wave 14's analyser was
//! committed 22 minutes *after* its first cell, contradicting three places that
//! said otherwise. It was only checkable because the bucket still existed.
//!
//! # Two fields, deliberately
//!
//! `emitted_unix_s` is an integer for machines: unambiguous, sortable, and with
//! no format to parse wrongly. `emitted_utc` is the same instant as ISO 8601 for
//! people, because a reader opening a cell should see a date rather than
//! 1,756,000,000.
//!
//! # No new dependency
//!
//! `std::time` gives the epoch second; the civil-date conversion below is the
//! standard days-from-civil algorithm, which is exact for every date this will
//! ever see and is pinned against known instants in the tests. Adding a date
//! crate for two fields would be a dependency this workspace does not otherwise
//! need.
//!
//! # This field is not a measurement
//!
//! It must never enter a bit-identity comparison: it differs on every run by
//! construction. `scripts/gate_f_rust.py` compares an explicit field list and
//! this is not on it, which `binn-lab/tests/timestamp_is_not_compared.rs` pins.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch, or 0 if the clock is before it.
///
/// A clock behind 1970 is a broken machine rather than a condition to model, and
/// returning 0 keeps the field present and obviously wrong instead of absent.
pub fn unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `YYYY-MM-DDTHH:MM:SSZ` for an epoch second.
///
/// Proleptic Gregorian, UTC, no leap seconds — which matches what every other
/// timestamp in this repository means.
pub fn iso8601_utc(epoch_seconds: i64) -> String {
    let days = epoch_seconds.div_euclid(86_400);
    let seconds_of_day = epoch_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds_of_day / 3600,
        (seconds_of_day % 3600) / 60,
        seconds_of_day % 60,
    )
}

/// Days since 1970-01-01 to `(year, month, day)`.
///
/// Howard Hinnant's `civil_from_days`, which is exact over the whole range of
/// `i64` days and has no branches for leap years beyond the era arithmetic.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64; // [0, 146096]
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153; // [0, 11], March-based
    let day = (day_of_year - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Instants with independently known civil dates, including the cases the
    /// era arithmetic is most likely to get wrong.
    #[test]
    fn known_instants_convert_exactly() {
        for (epoch, expected) in [
            (0_i64, "1970-01-01T00:00:00Z"),
            (1, "1970-01-01T00:00:01Z"),
            (86_399, "1970-01-01T23:59:59Z"),
            (86_400, "1970-01-02T00:00:00Z"),
            // 2000 is a leap year (divisible by 400); 1900 was not.
            (951_782_400, "2000-02-29T00:00:00Z"),
            (946_684_800, "2000-01-01T00:00:00Z"),
            // A year boundary, and a 31 December, where an off-by-one shows.
            (1_735_689_599, "2024-12-31T23:59:59Z"),
            (1_735_689_600, "2025-01-01T00:00:00Z"),
            // 2024 is a leap year; 2023 is not.
            (1_709_164_800, "2024-02-29T00:00:00Z"),
            (1_677_628_800, "2023-03-01T00:00:00Z"),
            // The morning this file was written. The first value written here
            // was a guessed epoch and the test caught it: 1_755_940_800 is
            // 2025-08-23, a year earlier. Computed rather than guessed now.
            (1_787_472_000, "2026-08-23T08:00:00Z"),
        ] {
            assert_eq!(iso8601_utc(epoch), expected, "epoch {epoch}");
        }
    }

    /// Before 1970 the arithmetic must still be right — `div_euclid` rather than
    /// truncating division is what makes that true, and a `/` here would put
    /// 1969-12-31T23:59:59Z one day out.
    #[test]
    fn instants_before_the_epoch_are_exact() {
        assert_eq!(iso8601_utc(-1), "1969-12-31T23:59:59Z");
        assert_eq!(iso8601_utc(-86_400), "1969-12-31T00:00:00Z");
    }

    /// The clock is real: it is after this file was written and before a date
    /// far enough out that a wrong unit — milliseconds for seconds, say — would
    /// land beyond it.
    #[test]
    fn the_clock_returns_a_plausible_instant() {
        let now = unix_seconds();
        assert!(now > 1_787_356_800, "clock reads {now}, before 2026-08-22");
        assert!(
            now < 4_102_444_800,
            "clock reads {now}, after the year 2100"
        );
    }

    /// Every field of the rendering is fixed width, so cells sort by string.
    #[test]
    fn the_rendering_is_sortable_as_text() {
        let epochs = [1_735_689_600_i64, 946_684_800, 1_755_940_800];
        let mut by_text: Vec<String> = epochs.iter().map(|e| iso8601_utc(*e)).collect();
        by_text.sort();
        let mut chronological = epochs;
        chronological.sort();
        let expected: Vec<String> = chronological.iter().map(|e| iso8601_utc(*e)).collect();
        assert_eq!(by_text, expected, "text order must be chronological order");
        for stamp in &by_text {
            assert_eq!(stamp.len(), 20, "{stamp} is not fixed width");
        }
    }
}
