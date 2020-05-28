use actix_web::{web, App, HttpServer};
mod handlers;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");

    HttpServer::new(|| {
        App::new()
            .route("/logs", web::get().to(handlers::get_logs))
            .route("/logs/{id}", web::get().to(handlers::get_log_by_id))
            .route("/logs", web::post().to(handlers::add_log))
    })
    .bind("127.0.0.1:8188")?
    .run()
    .await
}
