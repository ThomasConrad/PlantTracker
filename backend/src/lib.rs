use utoipa::OpenApi;

pub mod admin;
pub mod app_state;
pub mod auth;
pub mod database;
pub mod handlers;
pub mod llm;
pub mod middleware;
pub mod models;
pub mod utils;

use models::{
    google_oauth::{
        CreateGoogleTaskRequest, GoogleOAuthCallbackRequest, GoogleOAuthSuccessResponse,
        GoogleOAuthUrlResponse, GoogleTasksStatus, SyncPlantTasksRequest,
    },
    invite::{CreateInviteRequest, InviteResponse, ValidateInviteRequest},
    photo::{Photo, PhotosResponse},
    plant::{
        CreateCustomMetricRequest, CreatePlantRequest, CreatePlantCareTaskInput,
        CustomMetric, MetricDataType, PlantResponse, PlantsResponse,
        UpdateCustomMetricRequest, UpdatePlantRequest,
    },
    care_task::{
        CareTask, CareTaskWithStatus, CareTasksResponse, CreateCareTaskRequest,
        LogCareTaskRequest, ReorderCareTasksRequest, UpdateCareTaskRequest,
    },
    reminder::{
        DispatchRemindersResponse, DueReminder, DueRemindersResponse, ReminderPreferences,
        UpdateReminderPreferencesRequest,
    },
    tracking_entry::{
        CreateTrackingEntryRequest, Measurement, TrackingEntriesResponse, TrackingEntry,
    },
    user::{AuthResponse, CreateUserRequest, FirstDayOfWeek, LoginRequest, PreferredUnits, UserResponse, UserRole},
};

use admin::SystemStats;
use handlers::admin::{
    AdminDashboardResponse, AdminSettingsResponse, BulkUserAction, BulkUserActionRequest,
    InviteInfo, UpdateAdminSettingsRequest, UpdateUserRequest, UserListResponse,
};

use handlers::google_tasks::StoreTokensRequest;
use handlers::care_tasks::LogCareTaskResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::login,
        crate::handlers::auth::register,
        crate::handlers::admin::get_admin_dashboard,
        crate::handlers::admin::list_users,
        crate::handlers::admin::update_user,
        crate::handlers::admin::delete_user,
        crate::handlers::admin::bulk_user_action,
        crate::handlers::admin::get_admin_settings,
        crate::handlers::admin::update_admin_settings,
        crate::handlers::admin::get_system_health,
        crate::handlers::invites::create_invite,
        crate::handlers::invites::validate_invite,
        crate::handlers::invites::list_invites,
        crate::handlers::plants::list_plants,
        crate::handlers::plants::create_plant,
        crate::handlers::plants::get_plant,
        crate::handlers::plants::update_plant,
        crate::handlers::plants::delete_plant,
        crate::handlers::plants::archive_plant,
        crate::handlers::plants::unarchive_plant,
        crate::handlers::tracking::list_entries,
        crate::handlers::tracking::create_entry,
        crate::handlers::google_tasks::get_google_auth_url,
        crate::handlers::google_tasks::handle_google_oauth_callback,
        crate::handlers::google_tasks::store_google_tokens,
        crate::handlers::google_tasks::get_google_tasks_status,
        crate::handlers::google_tasks::disconnect_google_tasks,
        crate::handlers::google_tasks::sync_plant_tasks,
        crate::handlers::google_tasks::create_task,
        crate::handlers::reminders::get_preferences,
        crate::handlers::reminders::update_preferences,
        crate::handlers::reminders::get_due_reminders,
        crate::handlers::reminders::dispatch_due_reminders,
        crate::handlers::care_tasks::list_care_tasks,
        crate::handlers::care_tasks::create_care_task,
        crate::handlers::care_tasks::get_care_task,
        crate::handlers::care_tasks::update_care_task,
        crate::handlers::care_tasks::delete_care_task,
        crate::handlers::care_tasks::log_care_task,
        crate::handlers::care_tasks::archive_care_task,
        crate::handlers::care_tasks::unarchive_care_task,
        crate::handlers::care_tasks::reorder_care_tasks,
    ),
    components(
        schemas(
            AuthResponse,
            CreateUserRequest,
            LoginRequest,
            UserResponse,
            UserRole,
            PreferredUnits,
            FirstDayOfWeek,
            SystemStats,
            AdminDashboardResponse,
            AdminSettingsResponse,
            UserListResponse,
            UpdateUserRequest,
            UpdateAdminSettingsRequest,
            BulkUserActionRequest,
            BulkUserAction,
            InviteInfo,
            CreateInviteRequest,
            InviteResponse,
            ValidateInviteRequest,
            CreateTrackingEntryRequest,
            Measurement,
            TrackingEntriesResponse,
            TrackingEntry,
            Photo,
            PhotosResponse,
            PlantResponse,
            PlantsResponse,
            CreatePlantRequest,
            CreatePlantCareTaskInput,
            UpdatePlantRequest,
            CreateCustomMetricRequest,
            UpdateCustomMetricRequest,
            CareTask,
            CareTaskWithStatus,
            CareTasksResponse,
            CreateCareTaskRequest,
            UpdateCareTaskRequest,
            LogCareTaskRequest,
            LogCareTaskResponse,
            ReorderCareTasksRequest,
            CustomMetric,
            MetricDataType,
            ReminderPreferences,
            UpdateReminderPreferencesRequest,
            DueReminder,
            DueRemindersResponse,
            DispatchRemindersResponse,
            CreateGoogleTaskRequest,
            GoogleOAuthCallbackRequest,
            GoogleOAuthSuccessResponse,
            GoogleOAuthUrlResponse,
            GoogleTasksStatus,
            SyncPlantTasksRequest,
            StoreTokensRequest,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "admin", description = "Admin user and system management endpoints"),
        (name = "invites", description = "Invite system endpoints"),
        (name = "plants", description = "Plant management endpoints"),
        (name = "care_tasks", description = "Care task management endpoints"),
        (name = "tracking", description = "Plant care tracking endpoints"),
        (name = "photos", description = "Photo management endpoints"),
        (name = "google-tasks", description = "Google Tasks integration endpoints"),
        (name = "reminders", description = "In-app and browser reminder endpoints"),
    ),
    info(
        title = "Planty API",
        version = "0.1.0",
        description = "A REST API for Planty - tracking plant care and growth metrics",
        license(name = "MIT"),
    )
)]
pub struct ApiDoc;
