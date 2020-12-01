extern crate actix_web;
extern crate handlebars;
extern crate uuid;

extern crate serde;

#[macro_use]
extern crate serde_json;

extern crate actix_files;

use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};

use actix_files::Files;
use handlebars::Handlebars;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Serialize, Debug)]
struct Paste {
    uuid: String,
    author: String,
    content: String,
    //created: DateTime<Local>,
}

#[derive(Deserialize, Debug)]
pub struct PostedPaste {
    author: String,
    content: String,
}

// #[get("/")]
// async fn index() -> impl Responder {
//     HttpResponse::Ok().body(include_str!("../static/index.html"))
// }

fn render_paste_template(hb: web::Data<Handlebars>, paste_instance: &Paste) -> String {
    let data = json!(paste_instance);
    hb.render("paste", &data).unwrap()
}

#[get("/{uuid}")]
async fn paste(
    web::Path(uuid): web::Path<String>,
    data: web::Data<RwLock<HashMap<String, Paste>>>,
    hb: web::Data<Handlebars<'_>>,
) -> impl Responder {
    // In this case we are free to use only a read lock
    let map = data.read().unwrap();

    match map.get(&uuid) {
        Some(paste) => HttpResponse::Ok().body(render_paste_template(hb, paste)),
        None => HttpResponse::NotFound().body("404 not found"),
    }
}

#[post("/")]
async fn new_paste(
    data: web::Data<RwLock<HashMap<String, Paste>>>,
    // form: web::Form<PostedPaste>,
) -> impl Responder {
    // Our mutable state. We hold the write lock to the state here.
    let mut map = data.write().unwrap();

    // All of the necessary fields here
    let new_uuid = uuid::Uuid::new_v4().to_hyphenated().to_string().to_owned();
    // let author = form.author.clone();
    // let paste_content = form.content.clone();
    let author = "Chris".to_owned();
    let paste_content = "content".to_owned();

    // We will insert this struct into the map
    let new_paste = Paste {
        uuid: new_uuid.clone(),
        author: author,
        content: paste_content,
    };

    // Insert the paste in the in-memory map
    map.insert(new_uuid.clone(), new_paste);
    // Redirect to "/{uuid}"
    HttpResponse::Found()
        .header(http::header::LOCATION, new_uuid)
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
                // .service(index)
                .service(Files::new("/static", "static/").index_file("index.html"))
                .service(paste)
                .service(new_paste)
                // We mount the static directory to root now, so if none of our handlers matched yet we'll try
                // some static files.
                .service(Files::new("/", "static/").index_file("index.html"))
        },
    )
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
