// Copyright 2024 KONGSBERG
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
//    this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors
//    may be used to endorse or promote products derived from this software
//    without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
// WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

#![allow(unused, dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

const UNIX_YEAR: u64 = 1970;
const SECS_MINUTE: u64 = 60;
const SECS_HOUR: u64 = 60 * SECS_MINUTE;
const SECS_DAY: u64 = 24 * SECS_HOUR;
const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

const MONTH: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_year(year: u64) -> u64 {
    if is_leap(year) { 366 } else { 365 }
}

fn format_time(secs: u64) -> String {
    let clock = secs % SECS_DAY;
    format!(
        "{}:{}:{}",
        clock / SECS_HOUR,
        (clock % SECS_HOUR) / SECS_MINUTE,
        clock % SECS_MINUTE,
    )
}

fn format_date(secs: u64) -> String {
    let mut day = secs / SECS_DAY;
    let mut year = UNIX_YEAR;
    let mut month = 0;

    while day >= days_in_year(year) {
        day -= days_in_year(year);
        year += 1;
    }

    loop {
        let days_in_mo = DAYS_IN_MONTH[month];
        let leap_day = u64::from(is_leap(year));
        let delta = days_in_mo + leap_day;

        if day >= delta {
            day -= delta;
            month += 1;
        } else {
            break;
        }
    }

    format!("{} {:2} {year}", day + 1, MONTH[month])
}

fn now_secs() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_secs())
        .ok()
}

/// Returns a string of the current date in the format of `%b %d %Y`.
pub fn date() -> String {
    if let Some(secs) = now_secs() {
        format_date(secs)
    } else {
        "<unknown>".to_string()
    }
}

/// Returns a string of the current UTC time in %H:%M:%S format.
pub fn utc_time() -> String {
    if let Some(secs) = now_secs() {
        format_time(secs)
    } else {
        "00:00:00".to_string()
    }
}
