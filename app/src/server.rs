use actix_web::dev::Server;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Deserialize)]
struct PostVariables {
    bpm: f32,
    temperature: f32,
}

#[post("/variables")]
async fn post_variables(
    req: web::Json<PostVariables>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut vars = state.variables.lock().unwrap();
    let now = SystemTime::now();
    (*vars).bpm.push(Measurement::new(now, req.bpm));
    (*vars)
        .temperature
        .push(Measurement::new(now, req.temperature));

    HttpResponse::Ok()
}

#[derive(Clone)]
pub struct Measurement {
    pub timestamp: SystemTime,
    pub value: f32,
}

impl Measurement {
    pub fn new(timestamp: SystemTime, value: f32) -> Self {
        Measurement { timestamp, value }
    }
}

#[derive(Clone)]
pub struct Variables {
    pub bpm: Vec<Measurement>,
    pub temperature: Vec<Measurement>,
}

impl Variables {
    pub fn new() -> Self {
        Variables {
            bpm: Vec::new(),
            temperature: Vec::new(),
        }
    }
}

pub struct AppState {
    variables: Arc<Mutex<Variables>>,
}

pub struct IOTApp {
    port: u16,
}

impl IOTApp {
    pub fn new(port: u16) -> Self {
        IOTApp { port }
    }

    pub fn run(&self, variables: Arc<Mutex<Variables>>) -> std::io::Result<Server> {
        println!("Server running on port: {}", self.port);

        let server = HttpServer::new(move || {
            let state = AppState {
                variables: Arc::clone(&variables),
            };

            App::new()
                .data(state)
                .wrap(middleware::Logger::default())
                .service(status)
                .service(post_variables)
        });

        let server = server.bind(format!("127.0.0.1:{}", self.port));

        Ok(server?.run())
    }
}
