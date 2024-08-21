use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use mini_redis::client;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::runtime::Runtime;

async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Connect to mini-redis server
    let mut client: client::Client = client::connect("127.0.0.1:6379").await.expect("Failed to connect to mini-redis");

    // Fetch data from mini-redis
    let result = client.get("hello").await.expect("Failed to fetch data");

    // Prepare the response
    let response_text = match result {
        Some(value) => format!("Fetched data: {:?}", String::from_utf8(value.to_vec()).unwrap()),
        None => "No data found for the key".to_string(),
    };

    Ok(Response::new(Body::from(response_text)))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
