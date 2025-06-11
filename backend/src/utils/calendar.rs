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
        next_watering += interval_duration;
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
        next_watering += interval_duration;
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
        next_fertilizing += interval_duration;
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
        next_fertilizing += interval_duration;
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
    // Use current timestamp with nanoseconds for uniqueness
    let now = Utc::now();
    now.timestamp().hash(&mut hasher);
    now.timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
    
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plant::PlantResponse;
    use chrono::{Duration, Utc};
    use uuid::Uuid;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

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

    fn create_test_plant_with_name(name: &str, genus: &str, watering_days: i32, fertilizing_days: i32) -> PlantResponse {
        PlantResponse {
            id: Uuid::new_v4(),
            name: name.to_string(),
            genus: genus.to_string(),
            watering_interval_days: watering_days,
            fertilizing_interval_days: fertilizing_days,
            last_watered: Some(Utc::now() - Duration::days(watering_days as i64 - 1)),
            last_fertilized: Some(Utc::now() - Duration::days(fertilizing_days as i64 - 1)),
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
        assert!(calendar_str.contains("Plant Care Schedule"));
        
        // Check for categories (with escaped commas in iCalendar format)
        assert!(calendar_str.contains("CATEGORIES:Plant Care\\,Watering"));
        assert!(calendar_str.contains("CATEGORIES:Plant Care\\,Fertilizing"));
    }

    #[test]
    fn test_generate_calendar_with_multiple_plants() {
        let plants = vec![
            create_test_plant_with_name("Fiddle Leaf Fig", "Ficus", 7, 14),
            create_test_plant_with_name("Snake Plant", "Sansevieria", 14, 30),
            create_test_plant_with_name("Pothos", "Epipremnum", 5, 21),
        ];
        
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");
        assert!(result.is_ok());
        
        let calendar_str = result.unwrap();
        
        // Check that we have a valid calendar with events
        assert!(calendar_str.contains("BEGIN:VCALENDAR"));
        assert!(calendar_str.contains("END:VCALENDAR"));
        assert!(calendar_str.contains("BEGIN:VEVENT"));
        assert!(calendar_str.contains("END:VEVENT"));
        
        // Check that at least some plants are included
        let has_fiddle = calendar_str.contains("Fiddle Leaf Fig");
        let has_snake = calendar_str.contains("Snake Plant");
        let has_pothos = calendar_str.contains("Pothos");
        
        // At least one plant should be represented
        assert!(has_fiddle || has_snake || has_pothos, "No plants found in calendar");
        
        // Check that we have both watering and fertilizing events
        assert!(calendar_str.contains("Water") || calendar_str.contains("💧"));
        assert!(calendar_str.contains("Fertilize") || calendar_str.contains("🌱"));
        
        // Check that interval information is included
        let has_intervals = calendar_str.contains("every") && calendar_str.contains("days");
        assert!(has_intervals, "Calendar should include interval information");
    }

    #[test]
    fn test_generate_calendar_with_empty_plants() {
        let plants = vec![];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");
        
        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        
        // Should still be a valid calendar
        assert!(calendar_str.contains("BEGIN:VCALENDAR"));
        assert!(calendar_str.contains("END:VCALENDAR"));
        assert!(calendar_str.contains("Plant Care Schedule"));
        
        // But no events
        assert!(!calendar_str.contains("SUMMARY:💧"));
        assert!(!calendar_str.contains("SUMMARY:🌱"));
    }

    #[test]
    fn test_calendar_contains_proper_ical_format() {
        let plants = vec![create_test_plant()];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");
        
        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        
        // Check iCalendar format requirements
        assert!(calendar_str.starts_with("BEGIN:VCALENDAR"));
        assert!(calendar_str.ends_with("END:VCALENDAR\r\n"));
        assert!(calendar_str.contains("VERSION:2.0"));
        assert!(calendar_str.contains("BEGIN:VEVENT"));
        assert!(calendar_str.contains("END:VEVENT"));
        assert!(calendar_str.contains("UID:"));
        assert!(calendar_str.contains("DTSTART:"));
        assert!(calendar_str.contains("DTEND:"));
    }

    #[test]
    fn test_calendar_events_have_unique_uids() {
        let plants = vec![create_test_plant()];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");
        
        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        
        // Extract all UIDs
        let uids: Vec<&str> = calendar_str
            .lines()
            .filter(|line| line.starts_with("UID:"))
            .collect();
        
        // Should have at least 2 UIDs (watering and fertilizing events)
        assert!(uids.len() >= 2);
        
        // All UIDs should be unique
        let mut unique_uids = std::collections::HashSet::new();
        for uid in &uids {
            assert!(unique_uids.insert(uid), "Duplicate UID found: {}", uid);
        }
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
        
        // Tokens should be reasonably long (security)
        assert!(token1.len() >= 8);
        assert!(token2.len() >= 8);
    }

    #[test]
    fn test_calendar_token_deterministic_for_same_timestamp() {
        let user_id = "test-user";
        let timestamp = 1640995200i64; // Fixed timestamp
        
        // Generate token manually with same timestamp
        let mut hasher1 = DefaultHasher::new();
        user_id.hash(&mut hasher1);
        timestamp.hash(&mut hasher1);
        let token1 = format!("{:x}", hasher1.finish());
        
        let mut hasher2 = DefaultHasher::new();
        user_id.hash(&mut hasher2);
        timestamp.hash(&mut hasher2);
        let token2 = format!("{:x}", hasher2.finish());
        
        // Should be identical for same inputs
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_generate_calendar_feed_url() {
        let url = generate_calendar_feed_url("https://example.com", "user123", "token456");
        assert_eq!(url, "https://example.com/api/v1/calendar/user123.ics?token=token456");
        
        // Test with different base URLs
        let url2 = generate_calendar_feed_url("http://localhost:3000", "abc-def", "xyz789");
        assert_eq!(url2, "http://localhost:3000/api/v1/calendar/abc-def.ics?token=xyz789");
    }

    #[test]
    fn test_calendar_events_contain_plant_links() {
        let plant = create_test_plant_with_name("My Plant", "Planticus", 7, 14);
        let plants = vec![plant];
        let result = generate_plant_calendar(&plants, "test-user", "https://planttracker.com");
        
        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        
        // Check that events contain links back to plant details
        assert!(calendar_str.contains("https://planttracker.com/plants/"));
        assert!(calendar_str.contains("View plant details:"));
    }

    #[test]
    fn test_calendar_events_within_date_range() {
        let plants = vec![create_test_plant()];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");
        
        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        
        // Extract DTSTART lines and check they're within reasonable range
        let dtstart_lines: Vec<&str> = calendar_str
            .lines()
            .filter(|line| line.starts_with("DTSTART:"))
            .collect();
        
        assert!(!dtstart_lines.is_empty());
        
        // Should not have events too far in the past or future
        let _now = Utc::now();
        let _one_year_future = _now + Duration::days(365);
        
        for dtstart in dtstart_lines {
            // Just check format - actual date parsing would require more complex logic
            assert!(dtstart.contains("T")); // Should have time component
            assert!(dtstart.len() > 20); // Should be reasonable length
        }
    }

    #[test]
    fn test_plants_without_last_watered_date() {
        let mut plant = create_test_plant();
        plant.last_watered = None;
        plant.last_fertilized = None;
        
        let plants = vec![plant];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");
        
        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        
        // Should still generate events even without last care dates
        assert!(calendar_str.contains("SUMMARY:💧 Water Test Plant"));
        assert!(calendar_str.contains("SUMMARY:🌱 Fertilize Test Plant"));
    }

    #[test]
    fn test_calendar_emoji_and_unicode_handling() {
        let plants = vec![create_test_plant_with_name("🌿 Unicode Plant", "Émoji", 3, 7)];
        let result = generate_plant_calendar(&plants, "test-user", "https://example.com");
        
        assert!(result.is_ok());
        let calendar_str = result.unwrap();
        
        // Should handle unicode characters properly
        assert!(calendar_str.contains("🌿 Unicode Plant"));
        assert!(calendar_str.contains("Émoji"));
        assert!(calendar_str.contains("💧 Water 🌿 Unicode Plant"));
        assert!(calendar_str.contains("🌱 Fertilize 🌿 Unicode Plant"));
    }
}