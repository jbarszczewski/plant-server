use actix_web::{web, Error, HttpResponse, Responder};
use bson::{doc, oid::ObjectId, Bson, UtcDateTime};
use chrono::prelude::*;
use futures::stream::StreamExt;
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use rustc_serialize::hex::FromHex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    //   #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "_id")] // Use MongoDB's special primary key field name when serializing
    pub id: ObjectId,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    pub message: String,
    pub timestamp: UtcDateTime,
}

const MONGO_DB: &'static str = "iotPlantDB";
const MONGO_COLL_LOGS: &'static str = "logs";

pub async fn get_logs(data: web::Data<Arc<Mutex<Client>>>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let filter = doc! { "deviceId": "test_device_1" };
    //   let find_options = FindOptions::builder().sort(doc! { "timestamp": 1 }).build();
    let find_options = FindOptions::builder().sort(doc! { "_id": -1}).build();
    let mut cursor = logs_collection.find(filter, find_options).await.unwrap();

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                // let log: Log = bson::from_bson(Bson::Document(document)).unwrap();
                results.push(document);
            }
            _ => {
                return HttpResponse::InternalServerError().finish();
            }
        }
    }
    HttpResponse::Ok().json(results)
}

pub async fn get_log_by_id(
    data: web::Data<Arc<Mutex<Client>>>,
    log_id: web::Path<String>,
) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);
    let bytes = log_id.from_hex().unwrap();
    let mut byte_array: [u8; 12] = [0; 12];
    for i in 0..12 {
        byte_array[i] = bytes[i];
    }
    match logs_collection
        .find_one(doc! { "_id":ObjectId::with_bytes(byte_array)}, None)
        .await
    {
        Ok(result) => match result {
            Some(document) => return HttpResponse::Ok().json(document),
            None => {
                return HttpResponse::NotFound().body(format!("No log found with id: {}", log_id))
            }
        },
        Err(err) => {
            println!("Failed! {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    }
}

pub async fn add_log(
    data: web::Data<Arc<Mutex<Client>>>,
    device_id: String,
    message: String,
) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    match logs_collection.insert_one(doc! {"deviceId": device_id, "message": message, "timestamp": Bson::UtcDatetime(Utc::now())}, None).await {
        Ok(db_result) => {
            if let Some(new_id) = db_result.inserted_id.as_object_id() {
                println!("New document inserted with id {}", new_id);   
            }
            return HttpResponse::Created().json(db_result.inserted_id)
        }
        Err(err) =>
        {
            println!("Failed! {}", err);
            return HttpResponse::InternalServerError().finish()
        }
    }
}
