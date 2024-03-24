#![feature(lazy_cell)]

use std::cell::{LazyCell, OnceCell};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

pub const ROOT: LazyCell<String> = LazyCell::new(|| {
    let root = std::env::current_dir().unwrap();

    root.to_str().unwrap().to_string()
});

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/")]
async fn hello() -> impl Responder {
    let root_content = std::fs::read_dir(ROOT.as_str()).unwrap();

    let content_as_string = root_content
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
        .collect::<Vec<String>>()
        .join("\n");

    HttpResponse::Ok().body(content_as_string)
}
