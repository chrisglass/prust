use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};

//use uuid::Uuid;

use actix_files::Files;
use chrono::Utc;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::RwLock;

use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::types::Uuid;
use sqlx::{PgPool, Row}; // We need both PgRow and Row to get access to the methods defined on Row.

fn default_string() -> String {
    "".to_owned()
}

/// Paste is our main "business object". It holds the pastes in memory.
#[derive(Serialize, Deserialize, Debug)]
struct Paste {
    #[serde(default = "default_string")]
    uuid: String,
    author: String,
    content: String,
    #[serde(default = "default_string")]
    created: String,
}

fn render_paste_template(hb: web::Data<Handlebars>, paste_instance: &Paste) -> String {
    let data = serde_json::json!(paste_instance);
    hb.render("paste", &data).unwrap()
}

fn struct_mapper(row: PgRow) -> Paste {
    let row_uuid: Uuid = row.get("uuid");
    Paste {
        uuid: row_uuid.to_hyphenated().to_string(),
        author: row.get("author"),
        content: row.get("content"),
        created: row.get("created"),
    }
}

#[get("/{uuid}")]
async fn paste(
    web::Path(uuid): web::Path<String>,
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<PgPool>, // This is a bit weird: although we pass a PgPoolOptions, we get a PgPool here.
) -> impl Responder {
    let uuid_obj = Uuid::parse_str(&uuid);

    match uuid_obj {
        Ok(ok_uuid) => {
            let db_paste = sqlx::query("SELECT * FROM paste WHERE uuid = $1")
                .bind(ok_uuid)
                .map(struct_mapper) // transform the row into a Paste object
                .fetch_one(pool.get_ref()) // Get a ref for the inner stuff
                .await;

            match db_paste {
                Ok(paste) => HttpResponse::Ok().body(render_paste_template(hb, &paste)),
                Err(_) => HttpResponse::NotFound().body("404 not found"),
            }
        }
        Err(_) => HttpResponse::NotFound().body("404 not found"),
    }
}

#[post("/")]
async fn new_paste(
    // data: web::Data<RwLock<HashMap<String, Paste>>>,
    form: web::Form<Paste>,
    pool: web::Data<PgPool>, // This is a bit weird: although we pass a PgPoolOptions, we get a PgPool here.
) -> impl Responder {
    // The uuid for our newly created paste
    let new_uuid = Uuid::new_v4();

    // the timestamp for our created paste
    let time_created = Utc::now().to_rfc3339();

    let mut tx = pool.begin().await.unwrap();
    let todo: (Uuid,) = sqlx::query_as(
        "INSERT INTO paste (uuid, author, content, created) VALUES ($1, $2, $3, $4) RETURNING uuid",
    )
    .bind(new_uuid)
    .bind(&form.author)
    .bind(&form.content)
    .bind(&time_created)
    .fetch_one(&mut tx)
    .await
    .unwrap();

    tx.commit().await.unwrap();

    HttpResponse::Found()
        .header(http::header::LOCATION, todo.0.to_hyphenated().to_string())
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u32 = 3000;
    let max_connections: u32 = 5;
    println!("Running server on port {}", port);

    // Internally the web::Data wraps an Arc, so we just need to the RWLock here.
    let state: web::Data<RwLock<HashMap<String, Paste>>> =
        web::Data::new(RwLock::new(HashMap::new()));

    // Similarly we pass handlebars as an app data.
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./templates")
        .unwrap();

    let handlebars_ref = web::Data::new(handlebars);

    let database_url = "postgres://prust:prust@localhost:5432/prust";

    // TODO: handle errors
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(&database_url)
        .await
        .unwrap();

    HttpServer::new(
        // We need to move the state into the app closure here. This will be copied/moved for each of the connections
        // since actix spawns one instance of the closure per connection
        move || {
            App::new()
                .data(pool.clone()) // Pass the database connection pool. This is already wrapped for us so data vs. app_data.
                .app_data(state.clone()) // One app per connection, so we need to .clone() here
                .app_data(handlebars_ref.clone()) // Same here, one app per connection so we need to clone()
                .service(Files::new("/static", "static/").index_file("index.html"))
                .service(paste)
                .service(new_paste)
                // We mount the static directory to root, so if none of our handlers matched yet we'll try
                // some static files.
                .service(Files::new("/", "static/").index_file("index.html"))
        },
    )
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}
