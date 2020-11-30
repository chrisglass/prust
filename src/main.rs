extern crate actix_web;
extern crate handlebars;
extern crate uuid;
#[macro_use]
extern crate serde_json;

use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};
use handlebars::Handlebars;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug)]
struct Paste {
    uuid: uuid::Uuid,
    author: String,
    content: String,
    //created: DateTime<Local>,
}

#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    // Render a moustache or whatever template showing the paste form
    let data = json!({
        "name": "Chris"
    });
    let body = hb.render("index", &data).unwrap();

    HttpResponse::Ok().body(body)
}

#[get("/{uuid}")]
async fn paste(
    web::Path(uuid): web::Path<String>,
    data: web::Data<RwLock<HashMap<String, Paste>>>,
) -> impl Responder {
    // In this case we are free to use only a read lock
    let map = data.read().unwrap();

    match map.get(&uuid) {
        Some(paste) => HttpResponse::Ok().body(format!(
            "The paste:\n  uuid: {}\n  author: {}\n  content: {}\n",
            paste.uuid.to_string(),
            paste.author,
            paste.content
        )),
        None => HttpResponse::NotFound().body("404 not found"),
    }
}

#[post("/")]
async fn new_paste(data: web::Data<RwLock<HashMap<String, Paste>>>) -> impl Responder {
    // Our mutable state. We hold the write lock to the state here.
    let mut map = data.write().unwrap();

    // All of the necessary fields here
    let new_uuid = uuid::Uuid::new_v4();
    let author = "chris".to_owned();
    let paste_content = "some super cool content".to_owned();

    // We will insert this struct into the map
    let new_paste = Paste {
        uuid: new_uuid,
        author: author,
        content: paste_content,
    };

    // Insert the paste in the in-memory map
    map.insert(new_uuid.hyphenated().to_string(), new_paste);
    // Redirect to "/{uuid}"
    HttpResponse::Found()
        .header(http::header::LOCATION, new_uuid.hyphenated().to_string())
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = 3000;
    println!("Running server on port {}", port);

    // Internally the web::Data wraps an Arc, so we just need to pass the RWLock here
    let state: web::Data<RwLock<HashMap<String, Paste>>> =
        web::Data::new(RwLock::new(HashMap::new()));

    // Similarly we pass handlebars as an app data. this one doesn't have to be mut though
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./templates")
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    HttpServer::new(
        // We need to move the state into the app closure here
        move || {
            App::new()
                .app_data(state.clone()) // One app per connection, so we need to .clone() here
                .app_data(handlebars_ref.clone()) // Same here, one app per connection so we need to clone()
                .service(index)
                .service(paste)
                .service(new_paste)
        },
    )
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
