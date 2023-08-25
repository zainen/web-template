use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;
use async_trait::async_trait;
use std::sync::Mutex;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ForexPair {
    id: u64,
    pair: String,
    price: f64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    forex_pairs: HashMap<u64, ForexPair>,
}

impl Database {
    fn new() -> Self {
        Self {
            forex_pairs: HashMap::new(),
        }
    }

    fn insert(&mut self, forex_pair: ForexPair) {
        self.forex_pairs.insert(forex_pair.id, forex_pair);
    }

    fn get(&self, id: &u64) -> Option<&ForexPair> {
        self.forex_pairs.get(id)
    }

    fn get_all(&self) -> Vec<&ForexPair> {
        self.forex_pairs.values().collect()
    }

    fn update(&mut self, forex_pair: ForexPair) {
        self.forex_pairs.insert(forex_pair.id, forex_pair);
    }
}

struct AppState {
    db: Mutex<Database>
}

async fn create_forex_pair(app_state: web::Data<AppState>, forex_pair: web::Json<ForexPair>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert(forex_pair.into_inner());
    HttpResponse::Ok().finish()
} 

async fn read_forex_pair(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get(&id.into_inner()) {
        Some(forex_pair) => HttpResponse::Ok().json(forex_pair),
        None => HttpResponse::NotFound().finish()
    }
}

async fn read_all_forex_pairs(app_state: web::Data<AppState>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    let forex_pairs = db.get_all();
    HttpResponse::Ok().json(forex_pairs)
}

async fn update_forex_pair(app_state: web::Data<AppState>, forex_pair: web::Json<ForexPair>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.update(forex_pair.into_inner());
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Database::new();

    let data = web::Data::new(AppState {
        db: Mutex::new(db)
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                .allowed_origin_fn(|origin, _request_head| {
                    origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                })
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                .allowed_header(header::CONTENT_TYPE)
                .supports_credentials()
                .max_age(3600)
            )
            .app_data(data.clone())
            .route("/forex_pair", web::post().to(create_forex_pair))
            .route("/forex_pair", web::get().to(read_all_forex_pairs))
            .route("/forex_pair", web::put().to(update_forex_pair))
            .route("/forex_pair/{id}", web::get().to(read_forex_pair))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}