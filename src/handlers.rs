use actix_web::{HttpResponse, Responder};
use mongodb::{Client, options::ClientOptions};

pub async fn get_logs() -> impl Responder {
    HttpResponse::Ok().body("Getting logs not implemented yet!")
}

pub async fn get_log_by_id() -> impl Responder {
    HttpResponse::Ok().body("Getting log by id not implemented yet!")
}

pub async fn add_log() -> impl Responder {
    HttpResponse::Ok().body("Adding log not implemented yet!")
}
