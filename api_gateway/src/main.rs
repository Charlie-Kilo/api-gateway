use std::{error::Error, fmt};
use serde::{Deserialize, Serialize};
use warp::{http::header, Filter, Rejection, Reply};

#[derive(Debug, Deserialize, Serialize)]
struct ImageMetadata {
    season: String,
    show_name: String,
    designer: String,
    description: String,
    final_image_key: String,
    label: String,
    type_: String,
    requestId: String, // Add requestId field
}

#[derive(Debug, Deserialize, Serialize)]
struct Imagepath {
    final_image_path: String,
    requestId: String, // Add requestId field
}

#[derive(Debug, Deserialize, Serialize)]
struct ImageUrl {
    url: String,
    requestId: String, // Add requestId field
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
    // Extract requestId from the incoming metadata
    let requestId = metadata.requestId.clone();
    println!("Received metadata: {:?}, requestId: {}", metadata, requestId);

    // Send the metadata to the write-to-dynamo service
    if let Err(err) = send_to_write_to_dynamo(metadata, requestId.clone()).await {
        println!("Error sending metadata to write-to-dynamo service: {}", err);
        return Err(warp::reject::custom(err));
    }

    Ok(warp::reply::html("Received metadata successfully"))
}


async fn send_to_write_to_dynamo(metadata: ImageMetadata, requestId: String) -> Result<(), MyError> {
    // Serialize the metadata to JSON
    let json_data = serde_json::to_string(&metadata)
        .map_err(|e| MyError {
            message: format!("Serialization error: {}", e),
        })?;
    // Make an HTTP POST request to the write-to-dynamo service
    let client = reqwest::Client::new();
    client
        .post("http://localhost:3033/upload") // Change the URI to match your write-to-dynamo service endpoint
        .body(json_data)
        .header("requestId", requestId) // Include requestId in the header
        .send()
        .await
        .map_err(|e| MyError {
            message: format!("HTTP request error: {}", e),
        })?;

    Ok(())
}

async fn return_file_path_handler(image_path: Imagepath) -> Result<impl Reply, Rejection> {
    // Extract requestId from the incoming image_path
    let requestId = image_path.requestId.clone();
    println!("Handling request in existing API gateway: {:?}", image_path);
    let response_body = serde_json::json!({
        "final_image_path": image_path.final_image_path
    });
    Ok(warp::reply::html("Handled by existing API gateway"))
}

async fn post_url_handler(image_url: ImageUrl) -> Result<impl Reply, Rejection> {
    // Extract the URL from the JSON payload
    let url = image_url.url;
    let requestId = image_url.requestId.clone(); // Extract requestId from the incoming image_url
    println!("Received URL: {} Received Id: {}", url, requestId);
    
    // Send the URL to the save image service
    match send_to_save_image_service(url, requestId.clone()).await {
        Ok(message) => Ok(warp::reply::html(message)),
        Err(err) => Err(warp::reject::custom(MyError {
            message: format!("Error sending URL to save image service: {}", err),
        })),
    }
}

// Modify the function signature to accept `requestId` parameter
async fn send_to_save_image_service(url: String, requestId: String) -> Result<String, Box<dyn Error>> {
    // Serialize the URL payload to JSON
    let json_data = serde_json::to_string(&ImageUrl { url: url.clone(), requestId: requestId.clone() })
        .map_err(|e| format!("Serialization error: {}", e))?;

    // Make an HTTP POST request to the other service
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3032/url")
        .body(json_data)
        .header("requestId", requestId) // Include requestId in the header
        .send()
        .await
        .map_err(|e| format!("HTTP request error: {}", e))?;

    // Check if the request was successful
    if response.status().is_success() {
        Ok("URL sent successfully".to_string())
    } else {
        Err("Failed to send URL".to_string().into())
    }
}


#[tokio::main]
async fn main() {
    // Define the API endpoint routes
    let write_to_dynamo = warp::post()
        .and(warp::path("dynamo"))
        .and(warp::body::json())
        .and_then(handle_request);

    let return_file_path = warp::post()
        .and(warp::path("path"))
        .and(warp::body::json())
        .and_then(return_file_path_handler);

    let post_url = warp::post()
        .and(warp::path("url"))
        .and(warp::body::json())
        .and_then(post_url_handler);

    // Combine all routes
    let routes = write_to_dynamo.or(return_file_path).or(post_url);

    // Apply CORS globally to all routes
    use warp::http::header;

    // Inside the main function before starting the server
    let cors = warp::cors()
        .allow_any_origin() // Allow requests from any origin
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]) // Allow specific HTTP methods
        .allow_headers(vec!["Content-Type"]); // Allow specific headers
    let routes_with_cors = routes.with(cors);
    
    // Start the warp server
    warp::serve(routes_with_cors)
        .run(([127, 0, 0, 1], 3031))
        .await;    
}
