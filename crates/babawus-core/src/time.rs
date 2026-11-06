//! Timestamps in the log: local time with the UTC offset, for example
//! `2026-11-03 17:54:00 +0100`.

use std::mem::MaybeUninit;
use std::time::{Duration, SystemTime};

const SECONDS_PER_DAY: u64 = 24 * 60 * 60;

/// `YYYY-MM-DD HH:MM:SS +HHMM`
const TIMESTAMP_FORM: &[u8] = b"dddd-dd-dd dd:dd:dd sdddd";

/// A time in local time with the UTC offset, for the log.
pub fn log_timestamp(time: SystemTime) -> String {
    let seconds = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let seconds = libc::time_t::try_from(seconds).expect("the time fits in a time_t");
    let mut fields = MaybeUninit::<libc::tm>::uninit();
    // SAFETY: `seconds` is valid for reads, and `fields` is valid for writes.
    let result = unsafe { libc::localtime_r(&seconds, fields.as_mut_ptr()) };
    assert!(!result.is_null(), "a time since 1970 has a local time");
    // SAFETY: `localtime_r` succeeded, so it has filled in `fields`.
    let fields = unsafe { fields.assume_init() };
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
