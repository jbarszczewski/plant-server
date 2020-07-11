use actix_web::{web, Responder};
use chrono::prelude::*;

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
	cfg.service(web::resource("/time").route(web::get().to(get_datetime)));
}

async fn get_datetime() -> impl Responder {
	println!("{:?}", Utc::now());
	format!("{:?}", Utc::now())
}
