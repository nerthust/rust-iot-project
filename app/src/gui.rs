use gio::prelude::*;
use glib::{MainContext, Receiver, Sender};
use gtk::prelude::*;
use plotters::coord::types::RangedCoordf32;
use plotters::prelude::*;
use plotters_cairo::CairoBackend;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use std::{thread, time};

use crate::server::{Measurement, Variables};

// Main gui function that updates the interface view according to the contents of Arc<Mutex<Variables>>.
pub fn gui_main(vars: Arc<Mutex<Variables>>) {
    // Initialize GTK.
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        std::process::exit(1);
    }

    // Create a new GTK application.
    let app = gtk::Application::new(None, Default::default()).expect("Initialization failed...");

    // Set up application's handler.
    app.connect_activate(move |app| {
        update_gui(app, vars.clone()); // Main application's handler.
    });

    // Run GTK application.
    app.run(&std::env::args().collect::<Vec<_>>());
}

// Given a GTK app and the variables state (Arc<Mutex<Variables>>) draw a plot of BPM and
// TEMPERATURE over time.
fn update_gui(app: &gtk::Application, vars: Arc<Mutex<Variables>>) {
    // Initialize GTK window.
    let win = gtk::ApplicationWindow::new(app);

    // Set default window size.
    win.set_default_size(800, 800);
    win.set_title("Rust IOT");

    // Inistialize drawing are where plot is to be drawn.
    let drawing_area = gtk::DrawingArea::new();
    // Initialize GTK frame.
    let frame = gtk::Frame::new(None);

    // Attach drawing_area to frame.
    frame.add(&drawing_area);
    win.add(&frame);
    win.show_all();

    // Set up function that is going to update drawing are with plot. Note that Arc<Mutex<Variables>>
    // is passed as it has the points to be drawn.
    drawing_area.connect_draw(move |_, ctx| draw_plot(ctx, vars.clone()));

    // Clone drawing area.
    let drawing_area_clone = drawing_area.clone();

    // Set up a CHANNEL so that we can notify the drawing area when it has to update the plot.
    // Channels provide a sender and a receiver.
    let (sx, rx): (Sender<()>, Receiver<()>) = MainContext::channel(glib::PRIORITY_DEFAULT);

    // Fork a new thread that is going to send a signal to receiver every time plot has to be
    // redrawn. This thread loops indefinitely.
    thread::spawn(move || loop {
        // Send signal to receiver.
        sx.send(()).unwrap();

        let two_sec = time::Duration::from_millis(2000);

        // Signal is sent every two seconds so that GUI is refreshed.
        thread::sleep(two_sec);
    });

    // Attach handler to CHANNEL receiver. Every time a signal from sender is received, queue_draw
    // is called in order to refresh the plot.
    rx.attach(None, move |_| {
        drawing_area_clone.queue_draw();
        glib::Continue(true)
    });
}

// Given a Cairo context and the app's state (Arc<Mutex<Variables>>), draw a plot.
fn draw_plot(ctx: &cairo::Context, vars: Arc<Mutex<Variables>>) -> gtk::Inhibit {
    // Initialize Cairo backend wihin drawing area.
    let root = CairoBackend::new(&ctx, (600, 600))
        .unwrap()
        .into_drawing_area();

    // Fill drawing area with white color.
    root.fill(&WHITE).unwrap();

    // Set up plot margin.
    let root = root.margin(10, 10, 10, 10);

    // Set up Chart builder with all arguments to draw plot.
    let mut chart = ChartBuilder::on(&root)
        .caption(
            "BMP(red) & Oximetry(green) & Temperature(blue)",
            ("sans-serif", 20).into_font(),
        )
        .x_label_area_size(20)
        .y_label_area_size(40)
        .build_cartesian_2d(0f32..1200f32, 0f32..120f32)
        .unwrap();

    // Configure mesh when points are going to be displayed.
    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .y_label_formatter(&|x| format!("{:.1}", x))
        .draw()
        .unwrap();

    // Access application's state by requesting lock.
    let vars = vars.lock().unwrap();

    // Clone application's state so that lock is freed as soon as possible.
    let variables = (*vars).clone();

    // Drop lock to unlock access to application's state.
    std::mem::drop(vars);

    // Make BPM points to be drawn in chart.
    let bpm_points = mk_points(&variables.bpm);

    // Make TEMPERATURE points to be drawn in chart.
    let oxi_points = mk_points(&variables.oximetry);

    // Make TEMPERATURE points to be drawn in chart.
    let temperature_points = mk_points(&variables.temperature);

    // Draw BPM points.
    draw_series(&mut chart, bpm_points.clone(), &RED);
    draw_curve(&mut chart, bpm_points, &RED);

    // Draw OXIMETRY points.
    draw_series(&mut chart, oxi_points.clone(), &GREEN);
    draw_curve(&mut chart, oxi_points, &GREEN);

    // Draw TEMPERATURE points.
    draw_series(&mut chart, temperature_points.clone(), &BLUE);
    draw_curve(&mut chart, temperature_points, &BLUE);

    Inhibit(false)
}

type GuiChart<'a> = ChartContext<'a, CairoBackend<'a>, Cartesian2d<RangedCoordf32, RangedCoordf32>>;

// Given a chart, a set of points and a color, draw a curve.
fn draw_curve(chart: &mut GuiChart, points: Vec<(f32, f32)>, color: &RGBColor) {
    chart
        .draw_series(LineSeries::new(points, color))
        .unwrap()
        .label("foo")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 1000, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()
        .unwrap();
}

// Given a chart, a set of points and a color, draw points.
fn draw_series(chart: &mut GuiChart, points: Vec<(f32, f32)>, color: &RGBColor) {
    if points.len() > 0 {
        // Last point is to be drawn with coordinate labels unlike the others that will only
        // display a colored dot.
        let (last, init) = points.split_last().unwrap();

        // Draw all point series except the last element.
        chart
            .draw_series(PointSeries::of_element(
                init.to_vec(),
                5,
                color,
                &|c, s, st| return EmptyElement::at(c) + Circle::new((0, 0), s, st.filled()),
            ))
            .unwrap();

        // Draw the last point in the series but displaying the coordinate labels.
        chart
            .draw_series(PointSeries::of_element(
                vec![*last],
                5,
                color,
                &|c, s, st| {
                    return EmptyElement::at(c)
                        + Circle::new((0, 0), s, st.filled())
                        + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
                },
            ))
            .unwrap();
    }
}

// Given a vector of measurements, generate the vector of points to be drawn in chart. A point is
// simply a coordinate (t: f32, v: f32) where `t` is the time in seconds that has elapsed since the
// first measurement, and `v` is the measurement's value.
fn mk_points<'a>(v: &'a Vec<Measurement>) -> Vec<(f32, f32)> {
    // Initialize vector of points.
    let mut points = Vec::new();

    if v.len() > 0 {
        // Get unix timestamp of first measurement as seconds (t0).
        let t0 = v[0].timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Iterate over vector of measurements.
        for ms in v.iter() {
            // Get unix timestamp of measurement as seconds (t1).
            let t1 = ms.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs();

            // Compute the elapsed time between current measurement and first measurement.
            let diff = (t1 - t0) as f32;

            // Push point into vector.
            points.push((diff, ms.value))
        }
    }

    // Return points.
    points
}
