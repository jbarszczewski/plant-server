---
title: Rust + Actix + CosmosDB (MongoDB) tutorial api.
published: false
description: Simple web api done in Rust that stores and retrieve data from Azure Cosmos DB (or any MongoDB compatible database)
tags: rust,mongodb,webapi,tutorial
//cover_image: https://direct_url_to_image.jpg
---

# Intro
When working on one of my projects I decided to create simple logging API and Rust seemed like a perfect choice to learn some new tech. Same goes for going with Azure CosmosDB which now offer free tier that is perfect for learning and small personal projects.

I consider this tutorial to be a good starting point for beginner rustaceans (I'm one of those) but I assume that you know the basics. I highly recommend going through official [rustlings tutorial](https://github.com/rust-lang/rustlings).

"Final" code can be found on my [github repo](https://github.com/jbarszczewski/plant-server)

# Setting up "Hello fellow Rustacean!"
First let's create a new project by either creating new project directory and running `cargo init` from it or using `cargo new {project-name}` that also create directory for you. Once you're ready open your editor of choice (VS Code here with official 'Rust' extensions and 'cargo' that helps with staying up do date with dependencies) and let's start!

We will begin by creating simple http server that return us classic greatings. Open `Cargo.toml` and add two new dependencies:
```yaml
[dependencies]
actix-rt = "1.1.1"
actix-web = "2.0"
```
**Note:** First stick with versions I used for this tutorial and then update. I found that already few crates had some breaking changes so it's safer to application working first.

Then replace code in main.rs with the one below:
```rust
use actix_web::{web, App, HttpServer, Responder};
use std::env;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug");
    HttpServer::new(|| App::new().route("/", web::get().to(hello)))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}

async fn hello() -> impl Responder {
    format!("Hello fellow Rustacean!")
}

```

That's it! now just `cargo run` and go to `127.0.0.1:8000` in your browser.

Let's quickly see what we did here:
1. `#[actix_rt::main]` marked our main async function as to be executed in actix runtime.
2. `"RUST_LOG"`sets logger used by actix to output errors.
3. New `App` with registered request handler is passed to `HttpServer` to listen for incoming connections.

# Creating service configuration
As you can see, right now we've registered our routes in main function. In this tutorial we won't have many resources but it's good practice to have cleaner structure. Create new file: `src\logs_handlers\mod.rs` and add code below:

```
use actix_web::{web, Responder};

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/logs")
			.route(web::get().to(get_logs))
			.route(web::post().to(add_log)),
	);
}

async fn get_logs() -> impl Responder {
	format!("Not yet implemented!")
}

async fn add_log() -> impl Responder {
	format!("Not yet implemented!")
}

```

The `scoped_config()` function is responsible for registering logs resource configuration in our service. That means we can create multiple modules for each resource and then just call this configurator function for each of them from our `main` function. So, let's do that by modifying `App` builder code:

```rust
App::new().service(web::scope("/api").configure(logs_handlers::scoped_config))
```

Don't forget to import newly created module as well and add 
```rust
mod logs_handlers;
``` 
below the `use` block. 

Now we've set up our api to handle GET and POST methods on route `api/logs`. Try it!

# Connecting to CosmosDB/MongoDB
In this tutorial I use free tier Azure CosmosDB database with MongoDB API, but of course the choice is yours. Let's create db client and return some data. When I was prototyping my API, I was creating client in the handler manually. That is really inefficient way, instead we can utilize client pooling build in MongoDB client crate and setup it on app startup. Add MongoDB dependency to `Cargo.toml`
```yaml
bson = "1.0.0"
futures = "0.3.5"
mongodb = "1.0.0"
```
Open `main.rs` and add import module:
```rust
use mongodb::{options::ClientOptions, Client};
use std::sync::*;
```
And then modify your main function to look like this:
```rust
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug");
    let mut client_options = ClientOptions::parse("mongodb://free-tier-db:yfQtNbXyW2h9HUOOplCeHgjzzbJMnfMQn2BZuzAkw5gv0uBkqbdbQPdnQ98e6UtS5Z3p1ZrG4rgkmEKBURNgwg==@free-tier-db.mongo.cosmos.azure.com:10255/?ssl=true&replicaSet=globaldb&retrywrites=false&maxIdleTimeMS=120000&appName=@free-tier-db@").await.unwrap();
    client_options.app_name = Some("PlantApi".to_string());
    let client = web::Data::new(Mutex::new(Client::with_options(client_options).unwrap()));
    HttpServer::new(move || {
        App::new()
            .app_data(client.clone())
            .service(web::scope("/api").configure(logs_handlers::scoped_config))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
```
**Note:** Replace connection string with yours. Don't worry we will not leave connection string in the code in the final version.

Our new code creates a MongoDB client that is wrapped in mutex object for thread safety and which in turn is passed to application data object that is responsible for making it available in handlers.

# Fetching logs
Now we are ready to connect to our DB and make use of it. Go to `logs_handlers/mod.rs` and import few more modules:
```rust
use actix_web::{web, HttpResponse, Responder};
use bson::{doc, Bson};
use futures::stream::StreamExt;
use mongodb::{options::FindOptions, Client};
use std::sync::Mutex;

const MONGO_DB: &'static str = "iotPlantDB";
const MONGO_COLL_LOGS: &'static str = "logs";

...// no changes in  scoped_config

async fn get_logs(data: web::Data<Mutex<Client>>) -> impl Responder {
	let logs_collection = data
		.lock()
		.unwrap()
		.database(MONGO_DB)
		.collection(MONGO_COLL_LOGS);

	let filter = doc! {};
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

... // no changes in add_log
``` 
Ok, let's see what we did here. 
1. We created two constants to hold our DB and collection names.
2. Our `get_logs` function accept application data (our MongoDB client that we've set up in `main` function).
3. We use client to give us handle to the logs collection.
4. `find` function is used with no filter (returns everything) and simple sort by _id
5. We iterate results using `cursor` returned by `find `and populate result vector with incoming documents which then is returned in json format.

Now make sure you have some data in your DB and you're ready to test first call. My test data, and the one I will be using in `add_log` handler looks like this:
```json
{
	"_id": {
		"$oid": "5ee3bb1f00bc6d3b007b79ca"
	},
	"deviceId": "mock_device-01",
	"message": "test message",
	"createdOn": {
		"$date": "2020-06-12T17:27:59.404Z"
	}
}
```

Before we move to implementing POST handler let's do one more change. We should move the connection string out from the code. Let's save it as environmental variable named `CONNECTION_STRING_LOGS` and then we can replace line responsible for creating client options in `main.rs` with this:
```rust
 let mongo_url = env::var("CONNECTION_STRING_LOGS").unwrap();
 let mut client_options = ClientOptions::parse(&mongo_url).await.unwrap();
```
Much nicer solution!

# Adding logs
It's time to finish our api with the `add_log` handler. Add two more dependecies, `chrono` and `serde`, first will help us with DateTime and latter is the most popular serialize/deserialize crate. Your final dependency list should look like this:
```yaml
[dependencies]
actix-rt = "1.1.1"
actix-web = "2.0"
bson = "1.0.0"
chrono = "0.4.11"
futures = "0.3.5"
mongodb = "1.0.0"
serde = { version = "1.0", features = ["derive"] }
```

Let's go back to `logs_handlers/mod.rs`, import newly added modules and add struct for new logs:
```rust
use chrono::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewLog {
    pub id: String,
    pub message: String,
}
```
`NewLog` has `derive(Deserialize)` trait as this will be used to deserialize incoming POST body. We only need device/source id and a message to log, timestamp will be created in the handler function and MongoDB object id by the database.
Replace `add_log` function with:
```rust
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
```
As with `get_logs` this function makes use of mongo client stored in application data to get handle to the logs collection. Notice additional parameter `new_log` of type `NewLog` that we've created before. If your body match the struct (names and value types) it will be properly deserialized and ready to use. What this function does is:
1. Dynamically creates document using `new_log` data and fill `createdOn` with current UTC date and time.
2. Check the results and returns new document id if success.

And we're done! Two simple functions that can handle incoming GET and POST requests. Run the application and test it by adding new logs:
```curl
curl --location --request POST 'localhost:8000/api/logs' \
--header 'Content-Type: application/json' \
--data-raw '{
    "id":"tutorial-client",
    "message":"I'\''m a Rustacean!"
}'
```
# Optional `get_logs` code
This is a simple example so our `get_logs` function returns documents in the format it receives them. But what if we want to perform some operations on the results before we return them? We can easily deserialize document (and check if it matches our model) by modifying code slightly:
We add few more imports:
```rust
use bson::{doc, oid::ObjectId, Bson, UtcDateTime};
use serde::{Deserialize, Serialize};
```
Specify `Log` structure as it wil be different than `NewLog`
```rust
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Log {    
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "deviceId")]	
    pub device_id: String,	
    pub message: String,
    pub timestamp: UtcDateTime,
}
```

And just above the line where we push received document into the results vector, deserialize it using model above:
```rust
let log: Log = bson::from_bson(Bson::Document(document)).unwrap();
```

# Conclusion
We arrived at the end of this tutorial. As you can see it's not that complicated to create APIs in Rust. Of course, the example above is quite simple, but I think it's a good starting point even if you're not that familiar with Rust.
Please leave a comment if you liked (or not) this article or if something doesn't seem right etc.

Thanks for reading and till the next time!
