use actix_web::{Error, HttpResponse, Responder};
use bson::{bson, doc};
use chrono::prelude::*;
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    #[serde(rename = "_id")] // Use MongoDB's special primary key field name when serializing
    pub id: Option<bson::oid::ObjectId>,
    pub deviceId: String,
    pub message: String,
    pub timestamp: i64,
}

const MONGO_URL: &'static str = "mongodb://free-tier-db:yfQtNbXyW2h9HUOOplCeHgjzzbJMnfMQn2BZuzAkw5gv0uBkqbdbQPdnQ98e6UtS5Z3p1ZrG4rgkmEKBURNgwg==@free-tier-db.mongo.cosmos.azure.com:10255/?ssl=true&replicaSet=globaldb&retrywrites=false&maxIdleTimeMS=120000&appName=@free-tier-db@";
const MONGO_DB: &'static str = "iotPlantDB";
const MONGO_COLL_LOGS: &'static str = "logs";

pub async fn get_logs() -> impl Responder {
    let mut client_options = ClientOptions::parse(MONGO_URL).await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("PlantApi".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // Get a handle to a database.
    let iot_db = client.database(MONGO_DB);

    let logs_collection = iot_db.collection(MONGO_COLL_LOGS);
    let filter = doc! { "deviceId": "testDevice_1" };
    let find_options = FindOptions::builder().sort(doc! { "timestamp": 1 }).build();

    let cursor = logs_collection.find(filter, find_options).await;
    HttpResponse::Ok().body("Getting logs not implemented yet!")
}

pub async fn get_log_by_id() -> impl Responder {
    HttpResponse::Ok().body("Getting log by id not implemented yet!")
}

pub async fn add_log() -> Result<HttpResponse, Error> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse(MONGO_URL).await.unwrap();

    // Manually set an option.
    client_options.app_name = Some("PlantApi".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // Get a handle to a database.
    let iot_db = client.database(MONGO_DB);

    let logs_collection = iot_db.collection(MONGO_COLL_LOGS);

    let new_doc = doc! { "deviceId": "testDevice_1", "message": "George Orwell", "timestamp": bson::Bson::UtcDatetime(Utc::now())};
    logs_collection.insert_one(new_doc, None).await; // Insert into a MongoDB collection
    Ok(HttpResponse::Ok().body("Adding log not implemented yet!"))
}
