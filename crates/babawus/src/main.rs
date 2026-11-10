mod backup_page;
mod choose_page;
mod format_page;
mod icons;
mod info_page;
mod scroll;
mod setup_page;
mod window;
mod wrap;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use babawus_core::computer_lock::{COMPUTER_LOCK_FILE, ComputerLock, Taken};
use babawus_core::settings::SETTINGS_FILE;
use babawus_core::text::Text;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, glib,
};

const APP_ID: &str = "io.github.ziggymax.Babawus";
const MARGIN: i32 = 24;
const LOGO_SIZE: i32 = 64;

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

/// Says that Babawus is open for another account already, as a message with an OK button that
/// closes it (`ui-review/UiReview.md` W06).
fn show_already_open(app: &Application, user: &str) {
    let language = window::current_language(&glib::user_config_dir().join(SETTINGS_FILE));
    let message = Text::AlreadyOpen.fill(language, &[("user", user)]);
    let heading = Label::builder()
        .use_markup(true)
        .label(format!("<b>{}</b>", glib::markup_escape_text(&message)))
        .wrap(true)
        .xalign(0.0)
        .build();
    let detail = Label::builder()
        .label(Text::AlreadyOpenDetail.get(language))
        .wrap(true)
        .xalign(0.0)
        .build();
    let ok = Button::builder()
        .label(Text::AlreadyOpenOk.get(language))
        .halign(Align::End)
        .build();
    ok.add_css_class("suggested-action");
    let texts = GtkBox::new(Orientation::Vertical, 8);
    texts.set_hexpand(true);
    texts.append(&heading);
    texts.append(&detail);
    texts.append(&ok);
    let logo = icons::image(&icons::logo(), LOGO_SIZE);
    logo.set_valign(Align::Start);
    let content = GtkBox::new(Orientation::Horizontal, MARGIN);
    content.set_margin_top(MARGIN);
    content.set_margin_bottom(MARGIN);
    content.set_margin_start(MARGIN);
    content.set_margin_end(MARGIN);
    content.append(&logo);
    content.append(&texts);
    let window = ApplicationWindow::builder()
        .application(app)
        .title(Text::WindowTitle.get(language))
        .default_width(480)
        .resizable(false)
        .child(&content)
        .build();
    let closing = window.clone();
    ok.connect_clicked(move |_| closing.close());
    window.set_default_widget(Some(&ok));
    window.present();
    ok.grab_focus();
}
