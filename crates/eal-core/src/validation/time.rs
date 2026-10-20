//! Python-compatible explicit ISO instants, normalised to microsecond UTC time.
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, Utc, Weekday};

/// Parse a date/time separated by `T`, requiring an explicit timezone.
/// Fractions beyond Python datetime's microsecond precision are truncated.
pub fn parse_time(value: &str) -> Result<DateTime<Utc>, String> {
    let (date, time) = value
        .split_once('T')
        .ok_or("Timestamp requires a date and time separated by T")?;
    let date = date_value(date).ok_or("Invalid ISO-8601 date")?;
    let (clock, offset) = if let Some(clock) = time.strip_suffix('Z') {
        (clock, 0i64)
    } else if let Some(index) = time
        .char_indices()
        .find_map(|(i, c)| matches!(c, '+' | '-').then_some(i))
    {
        let sign = if time.as_bytes()[index] == b'-' {
            -1
        } else {
            1
        };
        let (h, m, s, micro) = time_parts(&time[index + 1..]).ok_or("Invalid ISO-8601 timezone")?;
        // Python treats an all-zero integer offset as UTC even with a fraction.
        let seconds = h as i64 * 3600 + m as i64 * 60 + s as i64;
        if seconds >= 86400 {
            return Err("Invalid ISO-8601 timezone".into());
        }
        (
            &time[..index],
            sign * (seconds * 1_000_000 + if seconds == 0 { 0 } else { micro as i64 }),
        )
    } else {
        return Err("Timestamp must include a timezone".into());
    };
    let (h, m, s, micro) = time_parts(clock).ok_or("Invalid ISO-8601 time")?;
    let clock = NaiveTime::from_hms_micro_opt(h, m, s, micro).ok_or("Invalid ISO-8601 time")?;
    let naive = date
        .and_time(clock)
        .checked_sub_signed(Duration::microseconds(offset))
        .ok_or("Timestamp is outside the supported UTC range")?;
    if !(1..=9999).contains(&naive.year()) {
        return Err("Timestamp is outside the supported UTC range".into());
    }
    Ok(DateTime::from_naive_utc_and_offset(naive, Utc))
}

fn date_value(text: &str) -> Option<NaiveDate> {
    if text.len() < 4 || !text.as_bytes()[..4].iter().all(u8::is_ascii_digit) {
        return None;
    }
    let year: i32 = text[..4].parse().ok()?;
    if !(1..=9999).contains(&year) {
        return None;
    }
    if text.contains('W') {
        let bytes = text.as_bytes();
        let digits = |range: std::ops::Range<usize>| bytes[range].iter().all(u8::is_ascii_digit);
        let shape = match text.len() {
            7 => bytes[4] == b'W' && digits(5..7),
            8 => {
                bytes[4] == b'W' && digits(5..8)
                    || bytes[4] == b'-' && bytes[5] == b'W' && digits(6..8)
            }
            10 => {
                bytes[4] == b'-'
                    && bytes[5] == b'W'
                    && bytes[8] == b'-'
                    && digits(6..8)
                    && bytes[9].is_ascii_digit()
            }
            _ => false,
        };
        if !shape
            || !text
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b'W' || b == b'-')
        {
            return None;
        }
        let basic = text.replace('-', "");
        if !matches!(basic.len(), 7 | 8) || basic.as_bytes()[4] != b'W' {
            return None;
        }
        let week = basic[5..7].parse().ok()?;
        let day = if basic.len() == 8 {
            basic[7..8].parse::<u32>().ok()?
        } else {
            1
        };
        let weekday = match day {
            1 => Weekday::Mon,
            2 => Weekday::Tue,
            3 => Weekday::Wed,
            4 => Weekday::Thu,
            5 => Weekday::Fri,
            6 => Weekday::Sat,
            7 => Weekday::Sun,
            _ => return None,
        };
        NaiveDate::from_isoywd_opt(year, week, weekday)
    } else {
        if text.len() != 8
            && (text.len() != 10 || text.as_bytes()[4] != b'-' || text.as_bytes()[7] != b'-')
        {
            return None;
        }
        if !text
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_digit() || text.len() == 10 && matches!(i, 4 | 7) && b == b'-')
        {
            return None;
        }
        let format = if text.len() == 8 {
            "%Y%m%d"
        } else {
            "%Y-%m-%d"
        };
        NaiveDate::parse_from_str(text, format).ok()
    }
}

fn time_parts(text: &str) -> Option<(u32, u32, u32, u32)> {
    let fraction = text.find(['.', ',']);
    let (main, micros) = if let Some(index) = fraction {
        let fraction = &text[index + 1..];
        if fraction.is_empty() || !fraction.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let kept = &fraction[..fraction.len().min(6)];
        (
            &text[..index],
            kept.parse::<u32>().ok()? * 10u32.pow(6 - kept.len() as u32),
        )
    } else {
        (text, 0)
    };
    let values = if main.contains(':') {
        let pieces: Vec<_> = main.split(':').collect();
        if !(1..=3).contains(&pieces.len()) || pieces.iter().any(|v| v.len() != 2) {
            return None;
        }
        pieces
            .into_iter()
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()
            .ok()?
    } else {
        if !matches!(main.len(), 2 | 4 | 6) || !main.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        main.as_bytes()
            .chunks(2)
            .map(|v| std::str::from_utf8(v).ok()?.parse::<u32>().ok())
            .collect::<Option<Vec<_>>>()?
    };
    Some((
        values[0],
        *values.get(1).unwrap_or(&0),
        *values.get(2).unwrap_or(&0),
        micros,
    ))
}
