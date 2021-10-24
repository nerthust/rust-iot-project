use gio::prelude::*;
use glib::{MainContext, Receiver, Sender};
use gtk::prelude::*;
use plotters::prelude::*;
use plotters_cairo::CairoBackend;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use std::{thread, time};

use crate::server::{Measurement, Variables};

pub fn gui_main(vars: Arc<Mutex<Variables>>) {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        std::process::exit(1);
    }

    let app = gtk::Application::new(None, Default::default()).expect("Initialization failed...");

    app.connect_activate(move |app| {
        update_gui(app, vars.clone());
    });

    app.run(&std::env::args().collect::<Vec<_>>());
}

fn update_gui(app: &gtk::Application, vars: Arc<Mutex<Variables>>) {
    let win = gtk::ApplicationWindow::new(app);
    win.set_default_size(800, 800);
    win.set_title("Rust IOT");

    let drawing_area = gtk::DrawingArea::new();
    let frame = gtk::Frame::new(None);

    frame.add(&drawing_area);
    win.add(&frame);
    win.show_all();
    drawing_area.connect_draw(move |_, ctx| draw_plot(ctx, vars.clone()));

    let drawing_area_clone = drawing_area.clone();
    let (sx, rx): (Sender<()>, Receiver<()>) = MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || loop {
        sx.send(()).unwrap();

        let two_sec = time::Duration::from_millis(2000);
        thread::sleep(two_sec);
    });

    rx.attach(None, move |_| {
        drawing_area_clone.queue_draw();
        glib::Continue(true)
    });
}

fn draw_plot(ctx: &cairo::Context, vars: Arc<Mutex<Variables>>) -> gtk::Inhibit {
    ctx.rectangle(1.0, 1.0, 100.0, 200.0);
    ctx.fill();

    let root = CairoBackend::new(&ctx, (600, 600))
        .unwrap()
        .into_drawing_area();

    root.fill(&WHITE).unwrap();
    let root = root.margin(10, 10, 10, 10);
    // After this point, we should be able to draw construct a chart context
    let mut chart = ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption(
            "BMP(red) & Temperature(blue)",
            ("sans-serif", 20).into_font(),
        )
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(0f32..1200f32, 0f32..120f32)
        .unwrap();

    // Draw a mesh
    chart
        .configure_mesh()
        .x_labels(5)
        .y_labels(5)
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()
        .unwrap();

    let vars = vars.lock().unwrap();
    let variables = (*vars).clone();
    std::mem::drop(vars);

    let bpm_points = mk_points(&variables.bpm);
    let temperature_points = mk_points(&variables.temperature);

    // Draw BPM points
    chart
        .draw_series(PointSeries::of_element(bpm_points, 5, &RED, &|c, s, st| {
            return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
        }))
        .unwrap();

    // Draw TEMPERATURE points
    chart
        .draw_series(PointSeries::of_element(
            temperature_points,
            5,
            &BLUE,
            &|c, s, st| {
                return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
            },
        ))
        .unwrap();

    Inhibit(false)
}

fn mk_points<'a>(v: &'a Vec<Measurement>) -> Vec<(f32, f32)> {
    let mut points = Vec::new();

    if v.len() > 0 {
        let t0 = v[0].timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();

        for ms in v.iter() {
            let t1 = ms.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();
            let diff = (t1 - t0) as f32;
            points.push((diff, ms.value))
        }
    }

    points
}
