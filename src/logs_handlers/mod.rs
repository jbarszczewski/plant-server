use actix_web::{web, HttpResponse, Responder};
use bson::{doc, oid::ObjectId, Bson};
use chrono::prelude::*;
use futures::stream::StreamExt;
use mongodb::{options::FindOptions, Client};
use rustc_serialize::hex::FromHex;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize)]
pub struct NewLog {
    pub id: String,
    pub message: String,
}

const MONGO_DB: &'static str = "iotPlantDB";
const MONGO_COLL_LOGS: &'static str = "logs";

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/logs")
            .route(web::get().to(get_logs))
            .route(web::post().to(add_log)),
    );
    cfg.service(web::resource("/logs/{id}").route(web::get().to(get_log_by_id)));
}

async fn get_logs(data: web::Data<Mutex<Client>>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    let filter = doc! {};
    //   let find_options = FindOptions::builder().sort(doc! { "createdOn": 1 }).build();
    let find_options = FindOptions::builder().sort(doc! { "_id": -1}).build();
    let mut cursor = logs_collection.find(filter, find_options).await.unwrap();

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                results.push(document);
            }
            _ => {
                return HttpResponse::InternalServerError().finish();
            }
        }
    }
    HttpResponse::Ok().json(results)
}

async fn get_log_by_id(
    data: web::Data<Mutex<Client>>,
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

async fn add_log(data: web::Data<Mutex<Client>>, new_log: web::Json<NewLog>) -> impl Responder {
    let logs_collection = data
        .lock()
        .unwrap()
        .database(MONGO_DB)
        .collection(MONGO_COLL_LOGS);

    match logs_collection.insert_one(doc! {"deviceId": &new_log.id, "message": &new_log.message, "createdOn": Bson::DateTime(Utc::now())}, None).await {
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
