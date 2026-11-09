mod backup_page;
mod format_page;
mod icons;
mod info_page;
mod setup_page;
mod window;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use babawus_core::computer_lock::{COMPUTER_LOCK_FILE, ComputerLock, Taken};
use babawus_core::settings::SETTINGS_FILE;
use babawus_core::text::Text;
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
            window::show(app);
        }
        Ok(Taken::AlreadyOpen(user)) => show_already_open(app, &user),
        // The lock only guards against a second window. Each medium has a lock of its own, so
        // Babawus is more use open than closed.
        Err(error) => {
            eprintln!("Babawus could not take the lock in {COMPUTER_LOCK_FILE}: {error}");
            window::show(app);
        }
    }
}

fn show_already_open(app: &Application, user: &str) {
    let language = window::current_language(&glib::user_config_dir().join(SETTINGS_FILE));
    let text = Text::AlreadyOpen.fill(language, &[("user", user)]);
    let label = Label::builder()
        .label(text)
        .wrap(true)
        .margin_start(MARGIN)
        .margin_end(MARGIN)
        .build();
    ApplicationWindow::builder()
        .application(app)
        .title(Text::WindowTitle.get(language))
        .default_width(480)
        .default_height(120)
        .child(&label)
        .build()
        .present();
}
