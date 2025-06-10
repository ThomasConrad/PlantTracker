use plant_tracker_api::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let openapi = ApiDoc::openapi();
    println!("{}", serde_json::to_string_pretty(&openapi).unwrap());
}
