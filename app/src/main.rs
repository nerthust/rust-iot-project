use std::sync::{Arc, Mutex};
use std::{thread, time};

use rust_iot_project::gui::gui_main;
use rust_iot_project::server::{IOTApp, Variables};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let app = IOTApp::new(8080);
    let server_counter = Arc::new(Mutex::new(0));
    let server_variables = Arc::new(Mutex::new(Variables::new()));

    let gui_counter = Arc::clone(&server_counter);
    let gui_variables = Arc::clone(&server_variables);

    thread::spawn(move || {
        gui_main(gui_variables);
    });

    app.run(server_counter, server_variables).unwrap().await
}
