use chrono::{DateTime, Duration, Utc};
use icalendar::{Calendar, Component, Event, EventLike};

use crate::models::plant::PlantResponse;
use crate::utils::errors::AppError;

/// Generate an iCalendar feed for plant care events
pub fn generate_plant_calendar(plants: &[PlantResponse], _user_id: &str, base_url: &str) -> Result<String, AppError> {
    let mut calendar = Calendar::new()
        .name("Plant Care Schedule")
        .description("Watering and fertilizing schedule for your plants")
        .timezone("UTC")
        .done();

    let now = Utc::now();
    
    // Generate events for the next 365 days
    let end_date = now + Duration::days(365);
    
    for plant in plants {
        // Generate watering events
        generate_watering_events(&mut calendar, plant, now, end_date, base_url)?;
        
        // Generate fertilizing events
        generate_fertilizing_events(&mut calendar, plant, now, end_date, base_url)?;
    }

    Ok(calendar.to_string())
}

/// Generate watering events for a plant
fn generate_watering_events(
    calendar: &mut Calendar,
    plant: &PlantResponse,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    base_url: &str,
) -> Result<(), AppError> {
    let last_watered = plant.last_watered
        .unwrap_or_else(|| start_date - Duration::days(plant.watering_interval_days as i64));

    let interval_duration = Duration::days(plant.watering_interval_days as i64);
    let mut next_watering = last_watered + interval_duration;

    // Ensure we start from a future date
    while next_watering <= start_date {
        next_watering = next_watering + interval_duration;
    }

    let mut event_count = 0;
    while next_watering <= end_date && event_count < 100 { // Limit to prevent infinite loops
        let event = Event::new()
            .uid(&format!("water-{}-{}", plant.id, next_watering.timestamp()))
            .summary(&format!("💧 Water {}", plant.name))
            .description(&format!(
                "Time to water your {} ({}). Water every {} days.\n\nView plant details: {}/plants/{}",
                plant.name,
                plant.genus,
                plant.watering_interval_days,
                base_url,
                plant.id
            ))
            .starts(next_watering)
            .ends(next_watering + Duration::hours(1)) // 1-hour event duration
            .location(&format!("Plant: {} ({})", plant.name, plant.genus))
            .add_property("CATEGORIES", "Plant Care,Watering")
            .add_property("PRIORITY", "5") // Normal priority
            .done();

        calendar.push(event);
        next_watering = next_watering + interval_duration;
        event_count += 1;
    }

    Ok(())
}

/// Generate fertilizing events for a plant
fn generate_fertilizing_events(
    calendar: &mut Calendar,
    plant: &PlantResponse,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    base_url: &str,
) -> Result<(), AppError> {
    let last_fertilized = plant.last_fertilized
        .unwrap_or_else(|| start_date - Duration::days(plant.fertilizing_interval_days as i64));

    let interval_duration = Duration::days(plant.fertilizing_interval_days as i64);
    let mut next_fertilizing = last_fertilized + interval_duration;

    // Ensure we start from a future date
    while next_fertilizing <= start_date {
        next_fertilizing = next_fertilizing + interval_duration;
    }

    let mut event_count = 0;
    while next_fertilizing <= end_date && event_count < 100 { // Limit to prevent infinite loops
        let event = Event::new()
            .uid(&format!("fertilize-{}-{}", plant.id, next_fertilizing.timestamp()))
            .summary(&format!("🌱 Fertilize {}", plant.name))
            .description(&format!(
                "Time to fertilize your {} ({}). Fertilize every {} days.\n\nView plant details: {}/plants/{}",
                plant.name,
                plant.genus,
                plant.fertilizing_interval_days,
                base_url,
                plant.id
            ))
            .starts(next_fertilizing)
            .ends(next_fertilizing + Duration::hours(1)) // 1-hour event duration
            .location(&format!("Plant: {} ({})", plant.name, plant.genus))
            .add_property("CATEGORIES", "Plant Care,Fertilizing")
            .add_property("PRIORITY", "4") // Slightly lower priority than watering
            .done();

        calendar.push(event);
        next_fertilizing = next_fertilizing + interval_duration;
        event_count += 1;
    }

    Ok(())
}

/// Generate a calendar feed URL for a user
pub fn generate_calendar_feed_url(base_url: &str, user_id: &str, calendar_token: &str) -> String {
    format!("{}/api/v1/calendar/{}.ics?token={}", base_url, user_id, calendar_token)
}

/// Generate a secure calendar token for a user
pub fn generate_calendar_token(user_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    user_id.hash(&mut hasher);
    Utc::now().timestamp().hash(&mut hasher);
    
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plant::PlantResponse;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_plant() -> PlantResponse {
        PlantResponse {
            id: Uuid::new_v4(),
            name: "Test Plant".to_string(),
            genus: "Testicus".to_string(),
            watering_interval_days: 7,
            fertilizing_interval_days: 14,
            last_watered: Some(Utc::now()),
            last_fertilized: Some(Utc::now()),
            thumbnail_id: None,
            thumbnail_url: None,
            custom_metrics: vec![],
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
        
        // Check that the calendar contains expected components
        assert!(calendar_str.contains("BEGIN:VCALENDAR"));
        assert!(calendar_str.contains("END:VCALENDAR"));
        assert!(calendar_str.contains("SUMMARY:💧 Water Test Plant"));
        assert!(calendar_str.contains("SUMMARY:🌱 Fertilize Test Plant"));
    }

    #[test]
    fn test_generate_calendar_token() {
        let token1 = generate_calendar_token("user1");
        let token2 = generate_calendar_token("user2");
        let _token3 = generate_calendar_token("user1"); // Same user, different time
        
        // Tokens should be different for different users
        assert_ne!(token1, token2);
        
        // Tokens should be hexadecimal strings
        assert!(token1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(token2.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_calendar_feed_url() {
        let url = generate_calendar_feed_url("https://example.com", "user123", "token456");
        assert_eq!(url, "https://example.com/api/v1/calendar/user123.ics?token=token456");
    }
}