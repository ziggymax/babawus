use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use babawus_core::computer_lock::{COMPUTER_LOCK_FILE, ComputerLock, Taken};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Label, glib};

const APP_ID: &str = "io.github.ziggymax.Babawus";
const MARGIN: i32 = 24;

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    // Kept until the application ends. The system releases it at the latest when the process ends.
    let lock = Rc::new(RefCell::new(None));
    app.connect_activate(move |app| activate(app, &lock));
    app.run()
}

/// Runs when Babawus starts, and when the same user starts it again: the session bus then hands
/// that start to this process, so all that is left to do is to bring the window to the front.
/// Another user has another session bus, which is why the lock is needed.
fn activate(app: &Application, lock: &RefCell<Option<ComputerLock>>) {
    if let Some(window) = app.windows().first() {
        window.present();
        return;
    }
    let username = glib::user_name();
    let username = username.to_string_lossy();
    match ComputerLock::take(Path::new(COMPUTER_LOCK_FILE), &username) {
        Ok(Taken::Ours(taken)) => {
            *lock.borrow_mut() = Some(taken);
            show_main_window(app);
        }
        Ok(Taken::AlreadyOpen(user)) => show_already_open(app, &user),
        // The lock only guards against a second window. Each medium has a lock of its own, so
        // Babawus is more use open than closed.
        Err(error) => {
            eprintln!("Babawus could not take the lock in {COMPUTER_LOCK_FILE}: {error}");
            show_main_window(app);
        }
    }
}

fn show_window(app: &Application, width: i32, height: i32, text: &str) {
    ApplicationWindow::builder()
        .application(app)
        .title("Backup")
        .default_width(width)
        .default_height(height)
        .child(
            &Label::builder()
                .label(text)
                .wrap(true)
                .margin_start(MARGIN)
                .margin_end(MARGIN)
                .build(),
        )
        .build()
        .present();
}

fn show_main_window(app: &Application) {
    show_window(app, 640, 480, &format!("Babawus {}", babawus_core::VERSION));
}

fn show_already_open(app: &Application, user: &str) {
    let text = format!("Babawus is already open for {user} on this computer");
    show_window(app, 480, 120, &text);
}
