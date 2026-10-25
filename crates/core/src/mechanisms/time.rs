use crate::ports::clock::Clock;

pub fn now_ms(clock: &dyn Clock) -> i64 {
    clock.now_ms()
}

use crate::domain::session::ListGroup;
use chrono::{Datelike, Duration, TimeZone, Utc};

pub fn group_by_activity<Tz: TimeZone>(
    rows: &[(i64, i64)],
    now_ms: i64,
    tz: &Tz,
) -> Vec<ListGroup<i64>>
where
    Tz::Offset: std::fmt::Display,
{
    let Some(today) = Utc
        .timestamp_millis_opt(now_ms)
        .single()
        .map(|d| d.with_timezone(tz).date_naive())
    else {
        return Vec::new();
    };
    let yesterday = today - Duration::days(1);
    let this_week = today.iso_week();
    let last_week = (today - Duration::weeks(1)).iso_week();
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.cmp(&a.0)));
    let mut groups: Vec<ListGroup<i64>> = Vec::new();
    for (id, at) in sorted {
        let Some(date) = Utc
            .timestamp_millis_opt(at)
            .single()
            .map(|d| d.with_timezone(tz).date_naive())
        else {
            continue;
        };
        let label = if date == today {
            "Today".to_owned()
        } else if date == yesterday {
            "Yesterday".to_owned()
        } else if date.iso_week() == this_week {
            "This week".to_owned()
        } else if date.iso_week() == last_week {
            "Last week".to_owned()
        } else {
            date.format("%B %Y").to_string()
        };
        if let Some(group) = groups.iter_mut().find(|group| group.label == label) {
            group.rows.push(id);
        } else {
            groups.push(ListGroup {
                label,
                rows: vec![id],
            });
        }
    }
    groups
}
