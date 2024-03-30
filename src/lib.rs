mod file;

use chrono::{Local, Utc};

fn unix_time_stamp() -> i64 {
    let now = Utc::now();
    let unix_time_stamp = now.timestamp();
    unix_time_stamp
}

fn jp_date() -> String {
    let current_date = Local::now();
    let formatted_date = current_date.format("%Y-%m-%d").to_string();
    formatted_date
}
