use crate::stats::CommitCounts;
use chrono::{Datelike, Duration, Local};
use colored::Colorize;

pub fn print_stats(commits: &CommitCounts) {
    let now = Local::now().date_naive();
    let days_to_show = 183; // 6 months approx

    // Align start to the Sunday before or on the 6-months-ago mark
    let start_target = now - Duration::days(days_to_show);
    let days_from_sun = start_target.weekday().num_days_from_sunday();
    let grid_start = start_target - Duration::days(days_from_sun as i64);

    let total_days = (now - grid_start).num_days() + 1;
    let total_weeks = (total_days as f64 / 7.0).ceil() as i64;

    print_months_header(grid_start, total_weeks);

    for row in 0..7 {
        // Print day labels for Mon, Wed, Fri
        match row {
            1 => print!(" Mon "),
            3 => print!(" Wed "),
            5 => print!(" Fri "),
            _ => print!("     "),
        }

        for col in 0..total_weeks {
            let current_date = grid_start + Duration::days(col * 7 + row as i64);

            if current_date > now {
                // Determine if we should print empty space or nothing
                // Usually we just print nothing if it's future, but to keep alignment?
                // Actually the graph just stops at Today.
                // But since we iterate row by row, we should probably print spaces if it's strictly a grid?
                // No, usually the graph is rectangular, but the last week might be partial.
                // But we are inside the 'row' loop.
                // If we break here, the rest of the row is empty.
                break;
            }

            let count = commits.get(&current_date).unwrap_or(&0);
            print_cell(*count, current_date == now);
        }
        println!();
    }
}

fn print_months_header(start_date: chrono::NaiveDate, weeks: i64) {
    print!("     "); // Padding for day labels
    let mut current_week = start_date;
    let mut last_month = current_week.month();

    for _ in 0..weeks {
        let month = current_week.month();
        if month != last_month {
            let month_name = current_week.format("%b").to_string();
            print!("{} ", month_name);
            last_month = month;
        } else {
            print!("  "); // Width of a cell
        }
        current_week += Duration::days(7);
    }
    println!();
}

fn print_cell(count: i32, is_today: bool) {
    // Colors matching the Go implementation approximation
    // 0: Grey -
    // 1-4: Yellow/White? Go used:
    // >0 && <5: White FG, Black BG (actually reversed in logic: \033[1;30;47m -> Black Text, White BG)
    // >=5 && <10: Black Text, Yellow BG
    // >=10: Black Text, Green BG

    // We will use Colored crate's closest.

    let text = if count == 0 {
        "  - ".dimmed().to_string()
    } else {
        let s = if count >= 10 {
            format!(" {:<2} ", count)
        } else {
            format!("  {} ", count)
        };

        if is_today {
            // Magenta BG
            s.black().on_magenta().to_string()
        } else {
            match count {
                1..=4 => s.black().on_white().to_string(),
                5..=9 => s.black().on_yellow().to_string(),
                _ => s.black().on_green().to_string(),
            }
        }
    };

    print!("{}", text);
}
