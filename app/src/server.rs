use actix_web::dev::Server;
use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

// Status endpoint to check if server is alive.
#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok()
}

// Struct representing JSON payload with microcontroller's readings. `Deserialize` trait is derived in order to
// produce JSON deserializer automatically. JSON payload is of the form: '{"bpm":90.6,"temperature":30.1}'
#[derive(Deserialize)]
struct PostVariables {
    bpm: f32,
    temperature: f32,
    oximetry: f32,
}

// POST /variables endpoint: It receives a request body of the form '{"bpm":90.6,"temperature":30.1}' and stored
// microcontroller's readings in AppState.
#[post("/variables")]
async fn post_variables(
    req: web::Json<PostVariables>, // JSON request body.
    state: web::Data<AppState>,    // SERVER's state.
) -> impl Responder {
    // Get SERVER's mutex lock to access stored variables.
    let mut vars = state.variables.lock().unwrap();

    // Get current system time.
    let now = SystemTime::now();

    // Push microcontroller's BPM Measurement into vector.
    (*vars).bpm.push(Measurement::new(now, req.bpm));

    // Push microcontroller's OXIMETRY Measurement into vector.
    (*vars).oximetry.push(Measurement::new(now, req.oximetry));

    // Push microcontroller's TEMPERATURE Measurement into vector.
    (*vars)
        .temperature
        .push(Measurement::new(now, req.temperature));

    // Respond with HTTP 200.
    HttpResponse::Ok()
}

// A Measurement is a microcontroller reading and the timestamp of that reading on the server's
// reception of the request.
#[derive(Clone)]
pub struct Measurement {
    pub timestamp: SystemTime,
    pub value: f32,
}

// Implement a constructor for a Measurement.
impl Measurement {
    pub fn new(timestamp: SystemTime, value: f32) -> Self {
        Measurement { timestamp, value }
    }
}

// Variables are two vectors that store the microcontroller's BPM and TEMPERATURE readings over
// time.
#[derive(Clone)]
pub struct Variables {
    pub bpm: Vec<Measurement>,
    pub temperature: Vec<Measurement>,
    pub oximetry: Vec<Measurement>,
}

// Implement constructor for Variables.
impl Variables {
    pub fn new() -> Self {
        Variables {
            bpm: Vec::new(),
            temperature: Vec::new(),
            oximetry: Vec::new(),
        }
    }
}

// AppState is a container that holds the variables that have been received over time via HTTP
// requests. A Mutex is needed because the GUI thread is also reading this state. Also, an Arc container
// (Atomic reference counter) is needed because multiple threads can own this resources concurrently.
pub struct AppState {
    variables: Arc<Mutex<Variables>>,
}

// IOTApp represents the SERVER.
pub struct IOTApp {
    port: u16,
}

// Implementation of methods for IOTApp struct.
impl IOTApp {
    // Constructor that just receives port.
    pub fn new(port: u16) -> Self {
        IOTApp { port }
    }

    // Given an Arc<Mutex<Variables>> that contains the state of the application, run the server.
    pub fn run(&self, variables: Arc<Mutex<Variables>>) -> std::io::Result<Server> {
        println!("Server running on port: {}", self.port);

        // Create new HttpServer instance.
        let server = HttpServer::new(move || {
            // Create AppState
            let state = AppState {
                variables: Arc::clone(&variables), // Clone Arc container so that ownership is not moved.
            };

            App::new()
                .data(state) // Provide application state.
                .wrap(middleware::Logger::default()) // Set up logger.
                .service(status) // Set up status endpoint.
                .service(post_variables) // Set up post_variables endpoint.
        });

        // Bind server to localhost and port.
        let server = server.bind(format!("127.0.0.1:{}", self.port));

        Ok(server?.run())
    }
}
