use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, glib};

const APP_ID: &str = "io.github.ziggymax.Babawus";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(show_window);
    app.run()
}

fn show_window(app: &Application) {
    let label = Label::new(Some(&format!("Babawus {}", babawus_core::VERSION)));
    ApplicationWindow::builder()
        .application(app)
        .title("Backup")
        .default_width(640)
        .default_height(480)
        .child(&label)
        .build()
        .present();
}
