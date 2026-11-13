//! Timestamps in the log, in local time with the UTC offset, for example
//! `2026-11-03 17:54:00 +0100`, and in the names of older versions, in UTC, for example
//! `2026-11-03_165400`.

use std::mem::MaybeUninit;
use std::time::{Duration, SystemTime};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// `YYYY-MM-DD HH:MM:SS +HHMM`
const TIMESTAMP_FORM: &[u8] = b"dddd-dd-dd dd:dd:dd sdddd";

/// Splits `time` into its fields, using `localtime_r` or `gmtime_r`.
fn broken_down(
    time: SystemTime,
    convert: unsafe extern "C" fn(*const libc::time_t, *mut libc::tm) -> *mut libc::tm,
) -> libc::tm {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let seconds = libc::time_t::try_from(seconds).expect("the time fits in a time_t");
    let mut fields = MaybeUninit::<libc::tm>::uninit();
    // SAFETY: `seconds` is valid for reads, and `fields` is valid for writes.
    let result = unsafe { convert(&seconds, fields.as_mut_ptr()) };
    assert!(!result.is_null(), "a time since 1970 can be split up");
    // SAFETY: the conversion succeeded, so it has filled in `fields`.
    unsafe { fields.assume_init() }
}

/// A time in UTC, for the name of an older version: `YYYY-MM-DD_HHMMSS`.
///
/// This must not be local time, which can go backwards, because pruning relies on the order of
/// these names.
pub fn version_timestamp(time: SystemTime) -> String {
    let fields = broken_down(time, libc::gmtime_r);
    format!(
        "{:04}-{:02}-{:02}_{:02}{:02}{:02}",
        fields.tm_year + 1900,
        fields.tm_mon + 1,
        fields.tm_mday,
        fields.tm_hour,
        fields.tm_min,
        fields.tm_sec
    )
}

/// A date and a time of day, as a clock shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockTime {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
}

impl ClockTime {
    fn of(fields: &libc::tm) -> Self {
        Self {
            year: fields.tm_year + 1900,
            month: fields.tm_mon + 1,
            day: fields.tm_mday,
            hour: fields.tm_hour,
            minute: fields.tm_min,
        }
    }

    /// The number of calendar days from `self` to `later`, whatever the times of day.
    pub fn days_until(self, later: Self) -> i64 {
        let days =
            |time: Self| days_from_civil(time.year.into(), time.month.into(), time.day.into());
        days(later) - days(self)
    }
}

/// `time` in local time, for the user to read.
pub fn local_clock(time: SystemTime) -> ClockTime {
    ClockTime::of(&broken_down(time, libc::localtime_r))
}

/// `time` in UTC. The tests use this in place of [`local_clock`], so that they don't depend on
/// the time zone.
pub fn utc_clock(time: SystemTime) -> ClockTime {
    ClockTime::of(&broken_down(time, libc::gmtime_r))
}

/// A time in UTC, as the configuration notes when it was written: `YYYY-MM-DD HH:MM:SS UTC`.
pub fn utc_timestamp(time: SystemTime) -> String {
    let fields = broken_down(time, libc::gmtime_r);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        fields.tm_year + 1900,
        fields.tm_mon + 1,
        fields.tm_mday,
        fields.tm_hour,
        fields.tm_min,
        fields.tm_sec
    )
}

/// Reads a time written by [`utc_timestamp`].
pub fn parse_utc_timestamp(text: &str) -> Option<SystemTime> {
    parse_log_timestamp(&format!("{} +0000", text.strip_suffix(" UTC")?))
}

/// A time in local time with the UTC offset, for the log.
pub fn log_timestamp(time: SystemTime) -> String {
    let fields = broken_down(time, libc::localtime_r);
    let date = (fields.tm_year + 1900, fields.tm_mon + 1, fields.tm_mday);
    let clock = (fields.tm_hour, fields.tm_min, fields.tm_sec);
    format_timestamp(date, clock, fields.tm_gmtoff)
}

fn format_timestamp(
    (year, month, day): (i32, i32, i32),
    (hour, minute, second): (i32, i32, i32),
    offset_seconds: i64,
) -> String {
    let sign = if offset_seconds < 0 { '-' } else { '+' };
    let offset_minutes = offset_seconds.abs() / 60;
    let (offset_hours, offset_minutes) = (offset_minutes / 60, offset_minutes % 60);
    format!(
        "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} \
         {sign}{offset_hours:02}{offset_minutes:02}"
    )
}

/// Days since 1970-01-01 of a date in the Gregorian calendar.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    // Counting from March puts the leap day at the end of the year.
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// Reads a timestamp from the log. Returns `None` for anything else, which is how a garbled line
/// is recognised.
pub fn parse_log_timestamp(text: &str) -> Option<SystemTime> {
    let bytes = text.as_bytes();
    let has_form = bytes.len() == TIMESTAMP_FORM.len()
        && bytes
            .iter()
            .zip(TIMESTAMP_FORM)
            .all(|(byte, form)| match form {
                b'd' => byte.is_ascii_digit(),
                b's' => matches!(byte, b'+' | b'-'),
                form => byte == form,
            });
    if !has_form {
        return None;
    }
    let number = |from: usize, to: usize| text[from..to].parse::<i64>().ok();
    let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    let offset = number(21, 23)? * 3600 + number(23, 25)? * 60;
    let offset = if bytes[20] == b'-' { -offset } else { offset };
    let in_range = (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && hour < 24
        && minute < 60
        && second < 60;
    if !in_range {
        return None;
    }
    let days = days_from_civil(year, month, day);
    let seconds = days * 86_400 + hour * 3600 + minute * 60 + second - offset;
    let seconds = u64::try_from(seconds).ok()?;
    SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds))
}

/// The number of whole days from `then` to `now`. A time in the future counts as 0, because a
/// clock may be wrong.
pub fn days_since(then: SystemTime, now: SystemTime) -> u64 {
    now.duration_since(then)
        .map_or(0, |since| since.as_secs() / SECONDS_PER_DAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    /// Runs the test `name` again in a process in another time zone, which is the only safe way
    /// to change it. Returns whether this already is that process. The zone is written out in
    /// full, so that it doesn't depend on the system's time zone files.
    fn in_time_zone_of_india(name: &str) -> bool {
        const ZONE: &str = "IST-5:30";
        if std::env::var("TZ").is_ok_and(|zone| zone == ZONE) {
            return true;
        }
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([&format!("time::tests::{name}"), "--exact"])
            .env("TZ", ZONE)
            .status()
            .unwrap();
        assert!(status.success());
        false
    }

    #[test]
    fn version_timestamps_are_in_utc_whatever_the_time_zone() {
        if in_time_zone_of_india("version_timestamps_are_in_utc_whatever_the_time_zone") {
            assert_eq!(version_timestamp(at(1_793_724_840)), "2026-11-03_165400");
        }
    }

    #[test]
    fn log_timestamps_are_in_local_time_with_its_offset() {
        if in_time_zone_of_india("log_timestamps_are_in_local_time_with_its_offset") {
            let text = log_timestamp(at(1_793_724_840));

            assert_eq!(text, "2026-11-03 22:24:00 +0530");
        }
    }

    #[test]
    fn version_timestamps_have_the_form_in_the_requirements() {
        assert_eq!(version_timestamp(at(0)), "1970-01-01_000000");
        // Worked out separately, with Python's `datetime`.
        assert_eq!(version_timestamp(at(1_709_199_304)), "2024-02-29_093504");
    }

    #[test]
    fn version_names_sort_in_order_across_the_autumn_clock_change() {
        // In Central Europe, these are both 02:30 on 2026-10-25: the clocks went back in between.
        let before = version_timestamp(at(1_792_888_200));
        let after = version_timestamp(at(1_792_891_800));

        assert_eq!(before, "2026-10-25_003000");
        assert_eq!(after, "2026-10-25_013000");
        assert!(before < after);
    }

    #[test]
    fn pruning_understands_version_timestamps() {
        use std::ffi::{OsStr, OsString};

        use crate::versions::{Format, older_version, versions_to_remove};

        let name = OsStr::new("report.pdf");
        let older = older_version(name, &version_timestamp(at(1_792_888_200)), 1);
        let newer = older_version(name, &version_timestamp(at(1_792_891_800)), 1);
        let entries: Vec<OsString> = vec![newer, older.clone()];

        assert_eq!(versions_to_remove(&entries, name, 1, Format::Ext4), [older]);
    }

    #[test]
    fn timestamps_have_the_form_in_the_requirements() {
        let text = format_timestamp((2026, 11, 3), (17, 54, 0), 3600);

        assert_eq!(text, "2026-11-03 17:54:00 +0100");
    }

    #[test]
    fn offsets_can_be_negative_and_have_minutes() {
        assert_eq!(
            format_timestamp((2024, 2, 29), (6, 5, 4), -12_600),
            "2024-02-29 06:05:04 -0330"
        );
        assert_eq!(
            format_timestamp((2024, 2, 29), (6, 5, 4), 20_700),
            "2024-02-29 06:05:04 +0545"
        );
        assert_eq!(
            format_timestamp((2024, 2, 29), (6, 5, 4), 0),
            "2024-02-29 06:05:04 +0000"
        );
    }

    #[test]
    fn reading_a_timestamp_takes_the_offset_into_account() {
        // Both worked out separately, with Python's `datetime`.
        let cases = [
            ("2026-11-03 17:54:00 +0100", 1793724840),
            ("2024-02-29 06:05:04 -0330", 1709199304),
            ("1970-01-01 00:00:00 +0000", 0),
        ];
        for (text, seconds) in cases {
            assert_eq!(parse_log_timestamp(text), Some(at(seconds)), "{text}");
        }
    }

    #[test]
    fn the_same_moment_in_two_time_zones_is_the_same_time() {
        assert_eq!(
            parse_log_timestamp("2026-11-03 17:54:00 +0100"),
            parse_log_timestamp("2026-11-03 11:54:00 -0500")
        );
    }

    #[test]
    fn a_written_timestamp_reads_back_in_this_time_zone() {
        for seconds in [0, 951_782_400, 1_700_000_000, 1_793_000_000, 4_000_000_000] {
            let text = log_timestamp(at(seconds));

            assert_eq!(parse_log_timestamp(&text), Some(at(seconds)), "{text}");
        }
    }

    #[test]
    fn anything_else_is_not_a_timestamp() {
        for text in [
            "",
            "2026-11-03",
            "2026-11-03 17:54:00",
            "2026-11-03 17:54:00 0100",
            "2026-11-03 17:54:00 +010",
            "2026-13-03 17:54:00 +0100",
            "2026-00-03 17:54:00 +0100",
            "2026-11-32 17:54:00 +0100",
            "2026-11-00 17:54:00 +0100",
            "2026-11-03 24:00:00 +0100",
            "2026-11-03 17:60:00 +0100",
            "2026-11-03 17:54:60 +0100",
            "2026-11-03T17:54:00 +0100",
            "20a6-11-03 17:54:00 +0100",
            "1960-01-01 00:00:00 +0000",
            "2026-11-03 17:54:00 +01é",
        ] {
            assert_eq!(parse_log_timestamp(text), None, "{text}");
        }
    }

    #[test]
    fn days_are_whole_days() {
        let then = at(1_000_000);

        assert_eq!(days_since(then, then), 0);
        assert_eq!(days_since(then, at(1_000_000 + 86_399)), 0);
        assert_eq!(days_since(then, at(1_000_000 + 86_400)), 1);
        assert_eq!(days_since(then, at(1_000_000 + 47 * 86_400 + 5)), 47);
    }

    #[test]
    fn a_time_in_the_future_is_no_days_ago() {
        assert_eq!(days_since(at(2_000), at(1_000)), 0);
    }
}
