mod file;

use chrono::Local;

fn current_jp_date() -> String {
    let current_date = Local::now();
    let formatted_date = current_date.format("%Y-%m-%d").to_string();
    formatted_date
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_japanese_date() {
        // data = "2024-03-30";
        let current_date = Local::now();
        let date = current_date.format("%Y-%m-%d").to_string();
        assert_eq!(current_jp_date(), date);
    }
}
