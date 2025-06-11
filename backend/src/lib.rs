use utoipa::OpenApi;

pub mod auth;
pub mod database;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod utils;

use models::{
    photo::{Photo, PhotosResponse},
    plant::{CreateCustomMetricRequest, CreatePlantRequest, CustomMetric, MetricDataType, PlantResponse, PlantsResponse},
    tracking_entry::{
        CreateTrackingEntryRequest, EntryType, TrackingEntriesResponse, TrackingEntry,
    },
    user::{AuthResponse, CreateUserRequest, LoginRequest, UserResponse},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::auth::login,
        crate::handlers::auth::register,
        crate::handlers::tracking::list_entries,
        crate::handlers::tracking::create_entry,
    ),
    components(
        schemas(
            AuthResponse,
            CreateUserRequest,
            LoginRequest,
            UserResponse,
            CreateTrackingEntryRequest,
            EntryType,
            TrackingEntriesResponse,
            TrackingEntry,
            Photo,
            PhotosResponse,
            PlantResponse,
            PlantsResponse,
            CreatePlantRequest,
            CreateCustomMetricRequest,
            CustomMetric,
            MetricDataType,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "plants", description = "Plant management endpoints"),
        (name = "tracking", description = "Plant care tracking endpoints"),
        (name = "photos", description = "Photo management endpoints"),
    ),
    info(
        title = "Planty API",
        version = "0.1.0",
        description = "A REST API for Planty - tracking plant care and growth metrics",
        license(name = "MIT"),
    )
)]
pub struct ApiDoc;
