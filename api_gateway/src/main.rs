use tokio;
use warp::{Filter, Rejection, Reply};

#[derive(Debug, serde::Deserialize)]
struct ImageMetadata {
    season: String,
    show_name: String,
    designer: String,
    description: String,
    final_image_key: String,
    label: String,
    type_: String,
}

async fn handle_request(metadata: ImageMetadata) -> Result<impl Reply, Rejection> {
    // Here you would insert your logic to handle the incoming metadata
    println!("Received metadata: {:?}", metadata);
    Ok(warp::reply::html("Received metadata successfully"))
}

#[tokio::main]
async fn main() {
    // Define the API endpoint route
    let api_route = warp::post()
        .and(warp::path("upload"))
        .and(warp::body::json())
        .and_then(handle_request);

    // Start the warp server
    warp::serve(api_route)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
