//! `struct tm` for a Unix time, in UTC.

/// Year, month (1-12) and day of the month of a day count since
/// 1970-01-01 (Howard Hinnant's `civil_from_days`).
pub fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let mday = doy - (153 * mp + 2) / 5 + 1;
    let mon = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(mon <= 2), mon, mday)
}

/// `struct tm` fields (sec, min, hour, mday, mon 0-11, year-1900, wday,
/// yday, isdst) for a Unix time, in UTC.
pub fn tm_from_unix(t: i64) -> [i32; 9] {
    const CUM: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let (days, secs) = (t.div_euclid(86_400), t.rem_euclid(86_400));
    let (year, mon, mday) = civil_from_days(days);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let yday = CUM[(mon - 1) as usize] + mday - 1 + i64::from(leap && mon > 2);
    [
        (secs % 60) as i32,
        (secs / 60 % 60) as i32,
        (secs / 3600) as i32,
        mday as i32,
        (mon - 1) as i32,
        (year - 1900) as i32,
        (days + 4).rem_euclid(7) as i32,
        yday as i32,
        0,
    ]
}
