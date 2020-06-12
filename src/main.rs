use actix_web::{web, App, HttpServer};
use mongodb::{options::ClientOptions, Client};
use std::sync::*;

mod logs_handlers;

const MONGO_URL: &'static str = "mongodb://free-tier-db:yfQtNbXyW2h9HUOOplCeHgjzzbJMnfMQn2BZuzAkw5gv0uBkqbdbQPdnQ98e6UtS5Z3p1ZrG4rgkmEKBURNgwg==@free-tier-db.mongo.cosmos.azure.com:10255/?ssl=true&replicaSet=globaldb&retrywrites=false&maxIdleTimeMS=120000&appName=@free-tier-db@";

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    let mut client_options = ClientOptions::parse(MONGO_URL).await.unwrap();
    client_options.app_name = Some("PlantApi".to_string());
    let client = web::Data::new(Mutex::new(Client::with_options(client_options).unwrap()));

    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .service(web::scope("/api").configure(logs_handlers::scoped_config))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
