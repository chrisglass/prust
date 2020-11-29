extern crate chrono;
extern crate handlebars;
extern crate hyper;
extern crate serde_json;
extern crate uuid;

use chrono::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug)]
struct Paste {
    author: String,
    paste: String,
    created: DateTime<Local>,
}

fn get_index(_req: Request<Body>) -> Body {
    Body::from("Try posting data to /echo")
}

fn get_paste(id: String) -> Body {
    Body::from(format!("This is paste for id {}", id))
}

async fn router(
    req: Request<Body>,
    // data: Mutex<HashMap<&str, Paste>>,
) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    // match (req.method(), req.uri().path()) {
    match req.method() {
        &Method::GET => {
            match req.uri().path() {
                "/" => {
                    // If the get is on the "/" assume that's the index.
                    *response.body_mut() = get_index(req);
                }
                _ => {
                    // If the get is for anything else, assume we're being asked about an ID
                    let splits: Vec<&str> = req.uri().path().split("/").collect();
                    let id = splits[1];
                    *response.body_mut() = get_paste(id.to_owned());
                }
            }
        }

        &Method::POST => {
            // Parse the form contents into a new Paste struct
            // Generate a new uuid for the paste
            // Save the Paste in the "DB"
            // Return a redirect to the appropriate GET
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Ok(response)
}

#[tokio::main]
async fn main() {
    let port = 3000;
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let data: Arc<Mutex<HashMap<&str, Paste>>> = Arc::new(Mutex::new(HashMap::new()));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(move |_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(router))
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("Server started on {}", &addr);
    println!("UUID4: {}", Uuid::new_v4());
    println!("It is now: {}", Local::now());
    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
