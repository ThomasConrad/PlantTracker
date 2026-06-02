use chrono::{Duration, Utc};
use icalendar::{Calendar, Component, Event, EventLike};
use rand::RngCore;

use crate::models::plant::PlantResponse;
use crate::utils::errors::AppError;

/// Generate an iCalendar feed for plant care events
pub fn generate_plant_calendar(
    plants: &[PlantResponse],
    _user_id: &str,
    base_url: &str,
) -> Result<String, AppError> {
    let mut calendar = Calendar::new()
        .name("Plant Care Schedule")
        .description("Care schedule for your plants")
        .timezone("UTC")
        .done();

    let now = Utc::now();
    let end_date = now + Duration::days(365);

    for plant in plants {
        for task in &plant.care_tasks {
            let ct = &task.task;
            // Only generate events for tasks with intervals
            let Some(interval_days) = ct.interval_days else {
                continue;
            };
            if interval_days <= 0 {
                continue;
            }

            let last_performed = ct
                .last_performed
                .unwrap_or_else(|| now - Duration::days(i64::from(interval_days)));

            let interval_duration = Duration::days(i64::from(interval_days));
            let mut next = last_performed + interval_duration;

            // Skip past events
            let start_threshold = now - Duration::hours(1);
            while next <= start_threshold {
                next += interval_duration;
            }

            let icon = ct.icon.as_deref().unwrap_or("📋");
            let mut event_count = 0;
            while next <= end_date && event_count < 100 {
                let amount_info = ct.amount.map_or(String::new(), |amt| {
                    format!(
                        " Amount: {}{}",
                        amt,
                        ct.unit.as_deref().unwrap_or("")
                    )
                });

                let event = Event::new()
                    .uid(&format!("care-{}-{}-{}", ct.id, event_count, next.timestamp() as i64))
                    .summary(&format!("{} {} {}", icon, ct.name, plant.name))
                    .description(&format!(
                        "Time to {} your {} ({}).{} Every {} days.\n\nView plant details: {}/plants/{}",
                        ct.name.to_lowercase(),
                        plant.name,
                        plant.genus,
                        amount_info,
                        interval_days,
                        base_url,
                        plant.id
                    ))
                    .starts(next)
                    .ends(next + Duration::hours(1))
                    .location(&format!("Plant: {} ({})", plant.name, plant.genus))
                    .add_property("CATEGORIES", &format!("Plant Care,{}", ct.name))
                    .add_property("PRIORITY", "5")
                    .done();

                calendar.push(event);
                next += interval_duration;
                event_count += 1;
            }
        }
    }

    Ok(calendar.to_string())
}

/// Generate a calendar feed URL for a user
#[allow(dead_code)]
pub fn generate_calendar_feed_url(base_url: &str, user_id: &str, calendar_token: &str) -> String {
    format!(
        "{}/api/v1/calendar/{}.ics?token={}",
        base_url, user_id, calendar_token
    )
}

/// Generate a secure calendar token for a user
pub fn generate_calendar_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::care_task::{CareTask, CareTaskWithStatus};
    use crate::models::plant::PlantResponse;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_care_task(name: &str, icon: &str, interval: i32) -> CareTaskWithStatus {
        let now = Utc::now();
        let task = CareTask {
            id: Uuid::new_v4(),
            plant_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            name: name.to_string(),
            icon: Some(icon.to_string()),
            color: None,
            interval_days: Some(interval),
            amount: None,
            unit: None,
            notes: None,
            last_performed: Some(now),
            sort_order: 0,
            archived_at: None,
            created_at: now,
            updated_at: now,
        };
        CareTaskWithStatus {
            task,
            next_due: Some(now + Duration::days(i64::from(interval))),
            days_overdue: Some(-interval as i64),
            is_due: false,
        }
    }

    fn create_test_plant() -> PlantResponse {
        PlantResponse {
            id: Uuid::new_v4(),
            name: "Test Plant".to_string(),
            genus: "Testicus".to_string(),
            preview_id: None,
            preview_url: None,
            archived_at: None,
            custom_metrics: vec![],
            care_tasks: vec![
                make_care_task("Water", "💧", 7),
                make_care_task("Fertilize", "🌱", 14),
            ],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_id: "test-user".to_string(),
        }
    }

    #[test]
    fn test_generate_plant_calendar() {
        let plants = vec![create_test_plant()];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");

        assert!(result.is_ok());
        let calendar_str = result.unwrap();

        assert!(calendar_str.contains("BEGIN:VCALENDAR"));
        assert!(calendar_str.contains("END:VCALENDAR"));
        assert!(calendar_str.contains("Water Test Plant"));
        assert!(calendar_str.contains("Fertilize Test Plant"));
    }

    #[test]
    fn test_generate_calendar_with_empty_plants() {
        let plants = vec![];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");

        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        assert!(calendar_str.contains("BEGIN:VCALENDAR"));
        assert!(!calendar_str.contains("BEGIN:VEVENT"));
    }

    #[test]
    fn test_generate_calendar_token() {
        let token1 = generate_calendar_token();
        let token2 = generate_calendar_token();
        assert_ne!(token1, token2);
        assert!(token1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(token1.len() >= 8);
    }

    #[test]
    fn test_generate_calendar_feed_url() {
        let url = generate_calendar_feed_url("https://example.com", "user123", "token456");
        assert_eq!(
            url,
            "https://example.com/api/v1/calendar/user123.ics?token=token456"
        );
    }
}
