use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use futures::stream::TryStreamExt;

#[derive(Serialize, Deserialize, Debug)]
struct Articles {
    title: String,
    author: String,
}

async fn handle_request(
    _req: Request<Body>,
    mongo_client: Arc<Client>,
) -> Result<Response<Body>, Infallible> {
    // Fetch data from MongoDB
    let db: mongodb::Database = mongo_client.database("JestDB");
    let collection: mongodb::Collection<Articles> = db.collection::<Articles>("articles");
    let filter: mongodb::bson::Document = doc! {};

    match collection.find_one(filter, None).await {
        Ok(Some(result)) => {
            // Return the MongoDB data
            Ok(Response::new(Body::from(format!(
                "Fetched from MongoDB: {:?}",
                result
            ))))
        }
        Ok(None) => Ok(Response::new(Body::from("Data not found"))),
        Err(e) => Ok(Response::new(Body::from(format!(
            "MongoDB query error: {}",
            e
        )))),
    }
}

#[tokio::main]
async fn main() {
    // Set up MongoDB client
    let mongo_client_options: ClientOptions = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .expect("Failed to parse MongoDB URI");
    let mongo_client: Client = Client::with_options(mongo_client_options).expect("Failed to initialize MongoDB client");

    let mongo_client: Arc<Client> = Arc::new(mongo_client);

    let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(move |_conn| {
        let mongo_client: Arc<Client> = Arc::clone(&mongo_client);
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, Arc::clone(&mongo_client))
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
