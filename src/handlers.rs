use actix_web::{web, Error, HttpResponse, Responder};
use bson::{doc, oid::ObjectId, Bson, UtcDateTime};
use chrono::prelude::*;
use futures::stream::StreamExt;
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")] // Use MongoDB's special primary key field name when serializing
    pub id: Option<ObjectId>,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub message: String,
    pub timestamp: UtcDateTime,
}

const MONGO_DB: &'static str = "iotPlantDB";
const MONGO_COLL_LOGS: &'static str = "logs";

pub async fn get_logs(data: web::Data<Arc<Mutex<Client>>>) -> impl Responder {
    let client = data.lock().unwrap();
    let iot_db = client.database(MONGO_DB);
    let logs_collection = iot_db.collection(MONGO_COLL_LOGS);

    let filter = doc! { "deviceId": "test_device_1" };
    //   let find_options = FindOptions::builder().sort(doc! { "timestamp": 1 }).build();
    let find_options = FindOptions::builder().sort(doc! { "_id": -1}).build();
    let mut cursor = logs_collection.find(filter, find_options).await.unwrap();

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                println!("document {}", document);
                results.push(document);
            }
            _ => {}
        }
    }
    HttpResponse::Ok().json(results)
}

pub async fn get_log_by_id() -> impl Responder {
    HttpResponse::Ok().body("Getting log by id not implemented yet!")
}

pub async fn add_log(
    data: web::Data<Arc<Mutex<Client>>>,
    device_id: String,
    message: String,
) -> impl Responder {
    let client = data.lock().unwrap();
    let iot_db = client.database(MONGO_DB);
    let logs_collection = iot_db.collection(MONGO_COLL_LOGS);

    match logs_collection.insert_one(doc! {"deviceId": device_id, "message": message, "timestamp": Bson::UtcDatetime(Utc::now())}, None).await {
        Ok(db_result) => {
            if let Some(new_id) = db_result.inserted_id.as_object_id() {
                println!("New document inserted with id {}", new_id);            
                //HttpResponse::Ok().json(new_id)
            }
            return HttpResponse::Created().json(db_result.inserted_id)
        }
        Err(err) =>
        {
            println!("Failed!");
            return HttpResponse::InternalServerError().finish()
        }
    }
}
