use gio::prelude::*;
use glib::{MainContext, Sender, Receiver};
use gtk::prelude::*;
use plotters::prelude::*;
use plotters_cairo::CairoBackend;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::{thread, time};

use crate::server::Variables;

pub fn gui_main(vars: Arc<Mutex<Variables>>) {
    let mut vars = vars.lock().unwrap();
    let variables = (*vars).clone();
    std::mem::drop(vars);

    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        std::process::exit(1);
    }

    let app = gtk::Application::new(None, Default::default()).expect("Initialization failed...");

    app.connect_activate(|app| {
        update_gui(app);
    });

    app.run(&std::env::args().collect::<Vec<_>>());
}

fn update_gui(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::new(app);
    win.set_default_size(800, 800);
    win.set_title("Rust IOT");

    let drawing_area = gtk::DrawingArea::new();
    let frame = gtk::Frame::new(None);

    frame.add(&drawing_area);
    win.add(&frame);
    win.show_all();
    drawing_area.connect_draw(move |_, ctx| draw_plot(ctx) );

    let drawing_area_clone = drawing_area.clone();
    let (sx, rx): (Sender<()>, Receiver<()>) = MainContext::channel(glib::PRIORITY_DEFAULT);

    thread::spawn(move || {
        loop {
            sx.send(()).unwrap();

            let two_sec = time::Duration::from_millis(2000);
            thread::sleep(two_sec);
        }
    });

    rx.attach(None, move |val| {
        drawing_area_clone.queue_draw();
        glib::Continue(true)
    });
}

fn draw_plot(ctx: &cairo::Context) -> gtk::Inhibit {
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
        .caption("This is our first plot", ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(0f32..10f32, 0f32..10f32)
        .unwrap();

    // Then we can draw a mesh
    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw()
        .unwrap();

    // Similarly, we can draw point series
    chart
        .draw_series(PointSeries::of_element(
            vec![(0.0, 0.0), (5.0, 5.0), (8.0, 7.0)],
            5,
            &RED,
            &|c, s, st| {
                return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
            + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
            + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
            },
        ))
        .unwrap();

    Inhibit(false)
}
