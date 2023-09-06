The function will return the improved webserver code as follows:

```rust
use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use reqwest::Client as HttpClient;
use async_trait::async_trait;
use std::sync::Mutex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Assignment {
    id: u64,
    name: String,
    complete: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    password: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    assignments: HashMap<u64, Assignment>,
    users: HashMap<u64, User>
}

impl Database {
    fn new() -> Self {
        Self {
            assignments: HashMap::new(),
            users: HashMap::new()
        }
    }

    fn insert(&mut self, assignment: Assignment) {
        self.assignments.insert(assignment.id, assignment);
    }

    fn get(&self, id: &u64) -> Option<&Assignment> {
        self.assignments.get(id)
    }

    fn get_all(&self) -> Vec<&Assignment> {
        self.assignments.values().collect()
    }

    fn delete(&mut self, id: &u64) {
        self.assignments.remove(id);
    }

    fn update(&mut self, assignment: Assignment) {
        self.assignments.insert(assignment.id, assignment);
    }

    // USER DATA RELATED FUNCTIONS
    fn insert_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    fn get_user_by_name(&mut self, name: &str) -> Option<&User> {
        self.users.values().find(|u|  u.username == name )
    }

    //DB SAVING
    fn save_to_file(&self) -> std::io::Result<()> {
        let data: String = serde_json::to_string(&self)?;
        let mut file: fs::File = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file_content = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file_content)?;
        Ok(db)
    }
}

struct AppState {
    db: Mutex<Database>
}

async fn create_assignment(app_state: web::Data<AppState>, assignment: web::Json<Assignment>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert(assignment.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
} 

async fn read_assignment(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get(&id.into_inner()) {
        Some(assignment) => HttpResponse::Ok().json(assignment),
        None => HttpResponse::NotFound().finish()
    }
}

async fn read_all_assignments(app_state: web::Data<AppState>) -> impl Responder {
    let db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    let assignments = db.get_all();
    HttpResponse::Ok().json(assignments)
}

async fn update_assignment(app_state: web::Data<AppState>, assignment: web::Json<Assignment>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.update(assignment.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn delete_assignment(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.delete(&id.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn register(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    db.insert_user(user.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn login(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let mut db: std::sync::MutexGuard<Database> = app_state.db.lock().unwrap();
    match db.get_user_by_name(&user.username) {
        Some(stored_user) if stored_user.password == user.password => {
            HttpResponse::Ok().body("Logged in!")
        },
        _ => HttpResponse::BadRequest().body("Invalid username or password")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new()
    };

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
            .route("/assignment", web::post().to(create_assignment))
            .route("/assignment", web::get().to(read_all_assignments))
            .route("/assignment", web::put().to(update_assignment))
            .route("/assignment/{id}", web::post().to(read_assignment))
            .route("assignment/{id}", web::delete().to(delete_assignment))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```
This code provides a webserver with CRUD operations for assignments and user registration and login functionality. It uses Actix-web for the web server, Serde for serialization and deserialization, and Reqwest for HTTP client functionality. The data is stored in a simple in-memory database represented by a HashMap. The database is saved to a file after each operation to persist the data.