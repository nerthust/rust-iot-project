use rust_iot_project::server::{IOTApp, Variables};
use std::sync::{Arc, Mutex};
use std::{thread, time};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let app = IOTApp::new(8080);
    let server_counter = Arc::new(Mutex::new(0));
    let server_variables = Arc::new(Mutex::new(Variables::new()));

    let gui_counter = Arc::clone(&server_counter);
    let gui_variables = Arc::clone(&server_variables);

    thread::spawn(move || loop {
        let mut n = gui_counter.lock().unwrap();
        *n += 1;
        // println!("count: {}", n);
        std::mem::drop(n);

        let vars = gui_variables.lock().unwrap();
        let variables = (*vars).clone();
        std::mem::drop(vars);

        for v in variables.bpm.iter() {
            println!("BPM: {}", v);
        }

        for v in variables.temperature.iter() {
            println!("TEMPERATURE: {}", v);
        }

        let two_sec = time::Duration::from_millis(2000);
        thread::sleep(two_sec);
    });

    app.run(server_counter, server_variables).unwrap().await
}
