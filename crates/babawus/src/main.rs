mod backup_page;
mod choose_page;
mod format_page;
mod icons;
mod info_page;
mod log_window;
mod scroll;
mod setup_page;
mod window;
mod wrap;

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

use babawus_core::computer_lock::{COMPUTER_LOCK_FILE, ComputerLock, Taken};
use babawus_core::fs::RealFs;
use babawus_core::reminder::{self, CHECK_OPTION, state_file};
use babawus_core::settings::SETTINGS_FILE;
use babawus_core::text::Text;
use babawus_core::time::local_clock;
use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, Label, Orientation, glib,
};

const APP_ID: &str = "io.github.ziggymax.Babawus";
const MARGIN: i32 = 24;
const LOGO_SIZE: i32 = 64;

fn main() -> glib::ExitCode {
    let mut args: Vec<String> = std::env::args().collect();
    // With `--checklastbackup`, Babawus only opens if the last backup is too long ago, and then
    // first asks (issue 137). That is decided before anything is shown, so that
    // otherwise it closes without a trace.
    let reminder_days = if args.iter().any(|arg| arg == CHECK_OPTION) {
        args.retain(|arg| arg != CHECK_OPTION);
        let state_file = state_file(
            std::env::var_os("XDG_STATE_HOME").as_deref(),
            &glib::home_dir(),
        );
        let noted = reminder::read(&RealFs, &state_file);
        match reminder::due(noted, SystemTime::now(), local_clock) {
            Some(days) => Some(days),
            None => return glib::ExitCode::SUCCESS,
        }
    } else {
        None
    };
    let app = Application::builder().application_id(APP_ID).build();
    // Kept until the application ends. The system releases it at the latest when the process ends.
    let lock = Rc::new(RefCell::new(None));
    let reminder = Cell::new(reminder_days);
    app.connect_activate(move |app| match reminder.take() {
        Some(days) => show_reminder(app, days, &lock),
        None => activate(app, &lock),
    });
    // GTK doesn't know the option, so it is left out of what GTK sees.
    app.run_with_args(&args)
}

/// Runs when Babawus starts, and when the same user starts it again: the session bus then hands
/// that start to this process, so all that is left to do is to bring the window to the front.
/// Another user has another session bus, which is why the lock is needed.
fn activate(app: &Application, lock: &Rc<RefCell<Option<ComputerLock>>>) {
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

/// Asks whether to back up now, because the last backup was `days` days ago. "Back up now"
/// opens Babawus as a normal start would; "Not now", or closing, closes it (issue
/// 137).
fn show_reminder(app: &Application, days: i64, lock: &Rc<RefCell<Option<ComputerLock>>>) {
    let language = window::current_language(&glib::user_config_dir().join(SETTINGS_FILE));
    let heading = Text::ReminderDays.fill(language, &[("days", &days.to_string())]);
    let not_now = Button::with_label(Text::ReminderNotNow.get(language));
    let back_up = Button::with_label(Text::ReminderBackUp.get(language));
    back_up.add_css_class("suggested-action");
    let window = message_window(
        app,
        &heading,
        Text::ReminderQuestion.get(language),
        &[&not_now, &back_up],
    );
    let closing = window.clone();
    not_now.connect_clicked(move |_| closing.close());
    let (opening, app, lock) = (window.clone(), app.clone(), lock.clone());
    back_up.connect_clicked(move |_| {
        // The reminder goes first, or the start would only bring it to the front. Babawus is
        // held meanwhile, so that it doesn't end without windows in between.
        let _held = app.hold();
        opening.close();
        activate(&app, &lock);
    });
    window.set_default_widget(Some(&back_up));
    window.present();
    back_up.grab_focus();
}

/// Says that Babawus is open for another account already, as a message with an OK button that
/// closes it (`ui-review/UiReview.md` W06).
fn show_already_open(app: &Application, user: &str) {
    let language = window::current_language(&glib::user_config_dir().join(SETTINGS_FILE));
    let message = Text::AlreadyOpen.fill(language, &[("user", user)]);
    let ok = Button::with_label(Text::Ok.get(language));
    ok.add_css_class("suggested-action");
    let window = message_window(app, &message, Text::AlreadyOpenDetail.get(language), &[&ok]);
    let closing = window.clone();
    ok.connect_clicked(move |_| closing.close());
    window.set_default_widget(Some(&ok));
    window.present();
    ok.grab_focus();
}

/// A small window with the logo, `heading` in bold, `detail`, and `buttons` at the bottom
/// right.
fn message_window(
    app: &Application,
    heading: &str,
    detail: &str,
    buttons: &[&Button],
) -> ApplicationWindow {
    let language = window::current_language(&glib::user_config_dir().join(SETTINGS_FILE));
    let heading = Label::builder()
        .use_markup(true)
        .label(format!("<b>{}</b>", glib::markup_escape_text(heading)))
        .wrap(true)
        .xalign(0.0)
        .build();
    let detail = Label::builder()
        .label(detail)
        .wrap(true)
        .xalign(0.0)
        .build();
    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.set_halign(Align::End);
    for button in buttons {
        row.append(*button);
    }
    let texts = GtkBox::new(Orientation::Vertical, 8);
    texts.set_hexpand(true);
    texts.append(&heading);
    texts.append(&detail);
    texts.append(&row);
    let logo = icons::image(&icons::logo(), LOGO_SIZE);
    logo.set_valign(Align::Start);
    let content = GtkBox::new(Orientation::Horizontal, MARGIN);
    content.set_margin_top(MARGIN);
    content.set_margin_bottom(MARGIN);
    content.set_margin_start(MARGIN);
    content.set_margin_end(MARGIN);
    content.append(&logo);
    content.append(&texts);
    ApplicationWindow::builder()
        .application(app)
        .title(Text::WindowTitle.get(language))
        .default_width(480)
        .resizable(false)
        .child(&content)
        .build()
}
