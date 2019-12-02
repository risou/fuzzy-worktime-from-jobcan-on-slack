extern crate reqwest;
extern crate slack_api;
extern crate chrono;
use chrono::{DateTime, Local, TimeZone};
use chrono::prelude::{Datelike};

struct DailyWorktime {
    day: i64,
    time: i64,
}

fn main() {
    let client = reqwest::Client::new();

    let today = Local::today();
    let oldest_str = format!("{}-{}-01T00:00:00+09:00", today.year(), today.month());
    let oldest = match DateTime::parse_from_rfc3339(&oldest_str) {
        Ok(n) => { n.timestamp().to_string() },
        Err(err) => { err.to_string() },
    };
    let today_str = &today.to_string();

    let req = slack_api::im::HistoryRequest{
        channel: "",
        oldest: Some(&oldest),
        latest: Some(today_str),
        count: Some(300),
        ..slack_api::im::HistoryRequest::default()
    };

    let results = slack_api::im::history(&client, "", &req);

    let mut times = Vec::new();

    if let Ok(results) = results {
        if let Some(messages) = results.messages {
            for message in messages {
                if let slack_api::Message::BotMessage(bot_message) = message {
                    if let Some(text) = bot_message.text {
                        if let Some(_) = text.find("打刻しました :smiley:") {
                            let time = Local.timestamp(bot_message.ts.unwrap().parse::<f64>().unwrap() as i64, 0);
                            times.push(time);
                        }
                    }
                }
            }
        }
    } else {
        println!("{:?}", results);
    }

    let mut current_day: i64 = 0;
    let mut start_time: i64 = 0;
    let mut worktime: DailyWorktime = DailyWorktime { day: 0, time: 0 };
    let mut worktimes: Vec<DailyWorktime> = Vec::new();

    for time in times.iter().rev() {
        if current_day == time.day() as i64 {
            if start_time == 0 {
                start_time = time.timestamp();
            } else {
                worktime.time += time.timestamp() - start_time;
                start_time = 0;
            }
        } else {
            if current_day != 0 {
                worktimes.push(worktime);
            }
            current_day = time.day() as i64;
            start_time = time.timestamp();
            worktime = DailyWorktime {
                day: current_day,
                time: 0,
            };
        }
    }
    worktimes.push(worktime);

    let total_days = worktimes.len();
    let mut total_time = 0;

    for working_day in worktimes {
        let mut time = working_day.time;
        if time >= 60 * 60 * 4 {
            time -= 60 * 60;
        }
        total_time += time;
    }

    let avg_time = total_time / total_days as i64;
    let avg_hours = avg_time / 60 / 60;
    let avg_minutes = avg_time / 60 - avg_hours * 60;
    let total_hours = total_time / 60 / 60;
    let total_minutes = total_time / 60 - total_hours * 60;
    println!("稼働日数: {}, 平均稼働時間: {}, 合計稼働時間: {}", total_days, format!("{: >02}:{: >02}", avg_hours, avg_minutes), format!("{: >02}:{: >02}", total_hours, total_minutes));
}
