//! Scheduler expression syntax, deliberately independent of invocation dates.

use std::sync::LazyLock;

use regex::Regex;

// CreateSchedule probes, carina-test-008, ap-northeast-1, 2026-11-05:
// an execution-role error proves syntax was accepted before role validation;
// NEVER errors depend on time/StartDate/EndDate and are not syntax failures.
// https://github.com/carina-rs/carina-provider-awscc/issues/525#issuecomment-6655771279
pub(super) fn validate_schedule_expression(value: &str) -> Result<(), String> {
    // SDK custom-type dispatch calls the callback without declarative bounds.
    if !(1..=256).contains(&value.chars().count()) {
        return Err("Schedule expression must contain 1 to 256 characters".to_string());
    }
    let valid = value.split_once('(').is_some_and(|(keyword, rest)| {
        let Some(body) = rest.strip_suffix(')') else {
            return false;
        };
        if keyword.eq_ignore_ascii_case("rate") {
            valid_rate(body)
        } else if keyword.eq_ignore_ascii_case("cron") {
            valid_cron(body)
        } else if keyword.eq_ignore_ascii_case("at") {
            valid_at(body)
        } else {
            false
        }
    });
    if valid {
        Ok(())
    } else {
        Err("Invalid Schedule Expression: expected rate(...), cron(...), or at(...)".to_string())
    }
}

fn digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn positive_integer(value: &str) -> bool {
    digits(value) && value.bytes().any(|byte| byte != b'0')
}

fn valid_rate(body: &str) -> bool {
    let digit_count = body.bytes().take_while(u8::is_ascii_digit).count();
    let (number, unit) = body.split_at(digit_count);
    positive_integer(number)
        && matches!(
            unit.strip_prefix(' ')
                .unwrap_or(unit)
                .to_ascii_lowercase()
                .as_str(),
            "minute" | "minutes" | "hour" | "hours" | "day" | "days"
        )
}

fn valid_at(body: &str) -> bool {
    // AWS accepts Feb 30 / Apr 31, past dates, optional seconds and 24:00.
    // Validate field syntax/ranges, without calendar or current-time checks.
    static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"\A[0-9]{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12][0-9]|3[01])T(?:(?:[01][0-9]|2[0-3]):[0-5][0-9](?::[0-5][0-9])?|24:00(?::00)?)\z",
        )
        .expect("Scheduler at expression pattern must compile")
    });
    PATTERN.is_match(body)
}

#[derive(Clone, Copy)]
enum CronField {
    Minute,
    Hour,
    DayOfMonth,
    Month,
    DayOfWeek,
    Year,
}

impl CronField {
    fn value(self, value: &str) -> bool {
        let (min, max, names): (u8, u8, &[&str]) = match self {
            Self::Minute => (0, 59, &[]),
            Self::Hour => (0, 23, &[]),
            Self::DayOfMonth => (1, 31, &[]),
            Self::Month => (
                1,
                12,
                &[
                    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV",
                    "DEC",
                ],
            ),
            Self::DayOfWeek => (1, 7, &["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"]),
            Self::Year => return digits(value),
        };
        names.contains(&value) || bounded_integer(value, min, max)
    }

    fn special(self, value: &str) -> bool {
        match self {
            Self::DayOfMonth => {
                matches!(value, "L" | "LW")
                    || value.strip_prefix("L-").is_some_and(digits)
                    || value.strip_suffix('W').is_some_and(|day| self.value(day))
            }
            Self::DayOfWeek => {
                value == "L"
                    || value
                        .strip_suffix(['L', 'W'])
                        .is_some_and(|day| self.value(day))
                    || value.split_once('#').is_some_and(|(day, occurrence)| {
                        self.value(day) && bounded_integer(occurrence, 1, 5)
                    })
            }
            _ => false,
        }
    }

    fn item(self, value: &str) -> bool {
        if self.special(value) {
            return true;
        }
        let base = if let Some((base, step)) = value.split_once('/') {
            if !positive_integer(step) {
                return false;
            }
            base
        } else {
            value
        };
        base == "*"
            || self.value(base)
            || base.split_once('-').is_some_and(|(start, end)| {
                // Do not invent a range-order restriction; allow wrapping.
                self.value(start) && self.value(end)
            })
    }

    fn field(self, value: &str) -> bool {
        value
            .to_ascii_uppercase()
            .split(',')
            .all(|item| self.item(item))
    }
}

fn bounded_integer(value: &str, min: u8, max: u8) -> bool {
    if !digits(value) {
        return false;
    }
    let normalized = value.trim_start_matches('0');
    if normalized.is_empty() {
        return min == 0;
    }
    normalized
        .parse::<u8>()
        .is_ok_and(|n| (min..=max).contains(&n))
}

fn valid_cron(body: &str) -> bool {
    let fields = body.split_whitespace().collect::<Vec<_>>();
    if !matches!(fields.len(), 5 | 6) || (fields[2] == "?") == (fields[4] == "?") {
        return false;
    }
    use CronField::*;
    [Minute, Hour, DayOfMonth, Month, DayOfWeek, Year]
        .into_iter()
        .zip(fields)
        .all(|(field, value)| {
            (matches!(field, DayOfMonth | DayOfWeek) && value == "?") || field.field(value)
        })
}
