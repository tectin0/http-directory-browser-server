#![feature(lazy_cell)]

mod zipfile;

use std::{
    cell::{LazyCell, OnceCell},
    sync::OnceLock,
};

use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

pub const ROOT: LazyCell<String> = LazyCell::new(|| {
    let root = std::env::current_dir().unwrap();

    root.to_str().unwrap().to_string()
});

pub const PORT: OnceLock<u16> = OnceLock::new();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let port = PORT.get_or_init(|| {
        let port = std::env::var("PORT").unwrap_or_else(|_| "15522".to_string());

        port.parse().unwrap()
    }).to_owned();

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(hello)
            .service(index)
            .service(zip)
    })
    .bind(("0.0.0.0", port))?
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

#[get("/{subpath}")]
async fn index(path: web::Path<String>) -> impl Responder {
    let subpath = path.into_inner();

    let subpath = subpath.replace(";", "/");

    let full_path = format!("{}/{}", ROOT.as_str(), subpath);

    log::debug!("full_path: {}", full_path);

    let content = std::fs::read_dir(full_path).unwrap();

    let content_as_string = content
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string())
        .collect::<Vec<String>>()
        .join("\n");

    HttpResponse::Ok().body(content_as_string)
}

#[get("/{subpath}/{filename}.zip")]
async fn zip(path: web::Path<(String, String)>) -> impl Responder {
    let (subpath, filename) = path.into_inner();

    let subpath = subpath.replace(";", "/");

    let full_path = format!("{}/{}", ROOT.as_str(), subpath);

    log::debug!("full_path: {}", full_path);

    let destination_filename = format!("tmp/{}/{}.zip", subpath, filename);

    crate::zipfile::zip(&full_path, &destination_filename).unwrap();

    let content = std::fs::read(destination_filename).unwrap();

    HttpResponse::Ok()
        .content_type("application/zip")
        .body(content)
}
