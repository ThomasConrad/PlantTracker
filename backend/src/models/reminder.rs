use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReminderPreferences {
    pub enabled: bool,
    pub reminder_time: String,
    pub timezone: String,
    pub browser_notifications_enabled: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReminderPreferencesRequest {
    pub enabled: bool,
    #[validate(regex(path = "*REMINDER_TIME_RE"))]
    pub reminder_time: String,
    #[validate(length(min = 1, max = 64))]
    pub timezone: String,
    pub browser_notifications_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DueReminder {
    pub plant_id: String,
    pub plant_name: String,
    pub reminder_type: String,
    pub due_at: String,
    pub due_date: String,
    pub days_overdue: i64,
    pub already_sent: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DueRemindersResponse {
    pub reminders: Vec<DueReminder>,
    pub total_due: usize,
    pub unsent_count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DispatchRemindersResponse {
    pub reminders: Vec<DueReminder>,
    pub sent_count: usize,
}

lazy_static::lazy_static! {
    static ref REMINDER_TIME_RE: regex::Regex =
        regex::Regex::new(r"^(?:[01]\d|2[0-3]):[0-5]\d$").unwrap();
}
