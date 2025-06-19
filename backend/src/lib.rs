use utoipa::OpenApi;

pub mod admin;
pub mod app_state;
pub mod auth;
pub mod database;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod utils;

use models::{
    google_oauth::{
        CreateGoogleTaskRequest, GoogleOAuthCallbackRequest, GoogleOAuthSuccessResponse,
        GoogleOAuthUrlResponse, GoogleTasksStatus, SyncPlantTasksRequest,
    },
    invite::{
        CreateInviteRequest, InviteResponse, ValidateInviteRequest, WaitlistResponse,
        WaitlistSignupRequest,
    },
    photo::{Photo, PhotosResponse},
    plant::{CareSchedule, CreateCareScheduleRequest, CreateCustomMetricRequest, CreatePlantRequest, CustomMetric, MetricDataType, PlantResponse, PlantsResponse, UpdateCareScheduleRequest, UpdateCustomMetricRequest, UpdatePlantRequest},
    tracking_entry::{
        CreateTrackingEntryRequest, EntryType, TrackingEntriesResponse, TrackingEntry,
    },
    user::{AuthResponse, CreateUserRequest, LoginRequest, UserResponse, UserRole},
};

use admin::SystemStats;
use handlers::admin::{
    AdminDashboardResponse, AdminSettingsResponse, BulkUserAction, BulkUserActionRequest,
    InviteInfo, UpdateAdminSettingsRequest, UpdateUserRequest, UserListResponse,
};

use handlers::google_tasks::StoreTokensRequest;

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
        crate::handlers::invites::join_waitlist,
        crate::handlers::invites::list_waitlist,
        crate::handlers::plants::list_plants,
        crate::handlers::plants::create_plant,
        crate::handlers::plants::get_plant,
        crate::handlers::plants::update_plant,
        crate::handlers::plants::delete_plant,
        crate::handlers::tracking::list_entries,
        crate::handlers::tracking::create_entry,
        crate::handlers::google_tasks::get_google_auth_url,
        crate::handlers::google_tasks::handle_google_oauth_callback,
        crate::handlers::google_tasks::store_google_tokens,
        crate::handlers::google_tasks::get_google_tasks_status,
        crate::handlers::google_tasks::disconnect_google_tasks,
        crate::handlers::google_tasks::sync_plant_tasks,
        crate::handlers::google_tasks::create_task,
    ),
    components(
        schemas(
            AuthResponse,
            CreateUserRequest,
            LoginRequest,
            UserResponse,
            UserRole,
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
            WaitlistResponse,
            WaitlistSignupRequest,
            CreateTrackingEntryRequest,
            EntryType,
            TrackingEntriesResponse,
            TrackingEntry,
            Photo,
            PhotosResponse,
            PlantResponse,
            PlantsResponse,
            CreatePlantRequest,
            UpdatePlantRequest,
            CreateCustomMetricRequest,
            UpdateCustomMetricRequest,
            CareSchedule,
            CreateCareScheduleRequest,
            UpdateCareScheduleRequest,
            CustomMetric,
            MetricDataType,
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
        (name = "invites", description = "Invite system and waitlist endpoints"),
        (name = "plants", description = "Plant management endpoints"),
        (name = "tracking", description = "Plant care tracking endpoints"),
        (name = "photos", description = "Photo management endpoints"),
        (name = "google-tasks", description = "Google Tasks integration endpoints"),
    ),
    info(
        title = "Planty API",
        version = "0.1.0",
        description = "A REST API for Planty - tracking plant care and growth metrics",
        license(name = "MIT"),
    )
)]
pub struct ApiDoc;
