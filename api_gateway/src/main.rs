use std::{error::Error, fmt};
use serde::Serialize;
use warp::{Filter, Rejection, Reply};

#[derive(Debug, serde::Deserialize, Serialize)]
struct ImageMetadata {
    season: String,
    show_name: String,
    designer: String,
    description: String,
    final_image_key: String,
    label: String,
    type_: String,
}

#[derive(Debug)]
struct MyError {
    message: String,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MyError {}

impl warp::reject::Reject for MyError {}

async fn handle_request(metadata: ImageMetadata) -> Result<impl Reply, Rejection> {
    // Here you would insert your logic to handle the incoming metadata
    println!("Received metadata: {:?}", metadata);

    // Send the metadata to the write-to-dynamo service
    if let Err(err) = send_to_write_to_dynamo(metadata).await {
        println!("Error sending metadata to write-to-dynamo service: {}", err);
        return Err(warp::reject::custom(err));
    }

    Ok(warp::reply::html("Received metadata successfully"))
}

async fn send_to_write_to_dynamo(metadata: ImageMetadata) -> Result<(), MyError> {
    // Serialize the metadata to JSON
    let json_data = serde_json::to_string(&metadata)
        .map_err(|e| MyError {
            message: format!("Serialization error: {}", e),
        })?;

    // Make an HTTP POST request to the write-to-dynamo service
    let client = reqwest::Client::new();
    client
        .post("http://localhost:3030/upload") // Change the URI to match your write-to-dynamo service endpoint
        .body(json_data)
        .send()
        .await
        .map_err(|e| MyError {
            message: format!("HTTP request error: {}", e),
        })?;

    Ok(())
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
        .run(([127, 0, 0, 1], 3031)) // Change the port if needed
        .await;
}
