use std::sync::{Arc, Mutex};
use std::thread;

use rust_iot_project::gui::gui_main;
use rust_iot_project::server::{IOTApp, Variables};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set server logger.
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    // Create new IOTApp instance.
    let app = IOTApp::new(8080);

    // Mutex with stored variable values to be consumed by SERVER thread.
    let server_variables = Arc::new(Mutex::new(Variables::new()));

    // Mutex with stored variable values to be consumed by GUI thread.
    let gui_variables = Arc::clone(&server_variables);

    // Fork GUI thread.
    thread::spawn(move || {
        // Function that controls GUI flow. Mutex with stored variable values is passed.
        gui_main(gui_variables);
    });

    // Initialize server. Mutex with stored variable values is passed.
    app.run(server_variables).unwrap().await
}
