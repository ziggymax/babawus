//! The start screen: the media list on the left, the details of the selected medium on the
//! right, with the actions for it below them. See "Start Babawus and see status" in
//! Requirements.md. What to show comes from [`babawus_core::start_screen`]; this module only
//! puts it on the screen.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, SystemTime};

use babawus_core::backup_screen::Tracker;

use babawus_core::coverage_screen::{Gathered, coverage_screen};
use babawus_core::format_screen::{
    DriveChoice, Failure, can_format, drive_choices, label_problem, result_screen, suggested_label,
    warning,
};
use babawus_core::fs::{FileSystem, RealFs};
use babawus_core::history_screen::history;
use babawus_core::identity::this_computer_id;
use babawus_core::language::{Language, Languages};
use babawus_core::lock::ThisProcess;
use babawus_core::log::{LastBackup, Run, read_runs};
use babawus_core::media::{Medium, Status, Viewer};
use babawus_core::platform::{self, EjectError, Ejected, Sticks};
use babawus_core::run::{Backup, Ended, Message, RunningBackup};
use babawus_core::settings::Settings;
use babawus_core::setup_screen::{
    self, Editor, Here, HereAction, Opened, SaveError, overview, save_error_text,
};
use babawus_core::start_screen::{self, Action, Checked, Shown, Unlisted};
use babawus_core::text::{self, Text};
use babawus_core::time::local_clock;
use babawus_core::verdict_screen::{Context, verdict_screen};
use babawus_core::versions::Format;
use gtk4::prelude::*;
use gtk4::{
    AboutDialog, AlertDialog, Align, Application, ApplicationWindow, Box, Button, HeaderBar, Image,
    Label, ListBox, ListBoxRow, MenuButton, Orientation, Overlay, PolicyType, ScrolledWindow,
    SelectionMode, Separator, Spinner, Stack, gio, glib,
};

use crate::backup_page::BackupPage;
use crate::choose_page::ChoosePage;
use crate::format_page::FormatPage;
use crate::help_window::HelpWindow;
use crate::icons;
use crate::info_page::InfoPage;
use crate::log_window::LogWindow;
use crate::restore_window::RestoreWindow;
use crate::setup_page::{Instead, SetupEvent, SetupPage};

const MARGIN: i32 = 24;
const SPACING: i32 = 12;
/// Narrow enough that the user's four buttons fit on one line at 1024 × 768, in German too
/// (issue 125).
const LIST_WIDTH: i32 = 240;
/// How often the backup screen is updated: at most about 10 times a second. See
/// "Responsiveness" in Requirements.md.
const BACKUP_UPDATES: Duration = Duration::from_millis(100);
const LOGO_SIZE: i32 = 80;
/// Leaves room for the logo in the upper right corner of the details.
const LOGO_ROOM: i32 = LOGO_SIZE + MARGIN;
/// The size of a stick in the media list (issue 120).
const STICK_SIZE: i32 = 48;
/// The size of the marks in the explanation, close to the height of its text.
const MARK_SIZE: i32 = 18;
/// The size of each row's Eject button, whose picture has a circle of its own.
const EJECT_SIZE: i32 = 36;

const CSS: &str = "
.babawus-row-label { font-weight: bold; }
.babawus-stick-row {
    border: 1px solid alpha(currentColor, 0.5);
    border-radius: 8px;
    margin: 3px 6px;
}
/* The picture draws the button's circle itself. */
.babawus-eject {
    background: none;
    border: none;
    box-shadow: none;
    min-width: 0;
    min-height: 0;
    padding: 0;
}
.babawus-eject:hover { opacity: 0.85; }
.babawus-eject:active { opacity: 0.7; }
.babawus-eject:disabled { opacity: 0.4; }
/* Green on the selected row too, whose colours would otherwise win (issue 134). */
.babawus-start-backup {
    font-size: 18pt;
    padding: 4px 12px;
    border: 2px solid white;
    background: #6db442;
    color: white;
}
.babawus-start-backup:hover { background: #5fa138; }
.babawus-start-backup:active { background: #538f30; }
.babawus-start-backup:disabled { background: #a8c99a; color: #f2f2f2; }
.babawus-title { font-size: 20pt; font-weight: bold; }
.babawus-status { font-size: 13pt; }
.babawus-last-backup { font-size: 13pt; }
.babawus-plug-in { font-size: 16pt; }
.babawus-verdict { font-size: 18pt; font-weight: bold; }
.babawus-next-step { font-weight: bold; }
.babawus-advanced {
    background-color: #f0a020;
    color: #3a2400;
    border-radius: 6px;
    padding: 1px 8px;
    font-weight: bold;
}
";

/// A message about something Babawus did, such as ejecting, kept as texts so that it can be
/// shown again in another language.
#[derive(Clone)]
struct Notice {
    text: Text,
    /// The label of the medium it is about.
    label: String,
}

/// One row of the media list: the stick, its label and its Eject button, and below them
/// "Start backup" while it is the selected Babawus stick (issue 134).
struct RowWidgets {
    row: ListBoxRow,
    stick: Image,
    label: Label,
    spinner: Spinner,
    eject: Button,
    start_backup: Button,
}

/// The parts of the details that change.
struct DetailsWidgets {
    pages: Stack,
    plug_in: Label,
    /// What to do, below the empty page's heading, such as "Pull it out and plug it back in.".
    empty_advice: Label,
    /// A notice on the empty page, such as "You can pull out BACKUP-A now.".
    empty_notice: Label,
    /// "Prepare…" on the empty page, so that a blank stick can be formatted while Advanced is
    /// on. See issue 78.
    empty_prepare: Button,
    title: Label,
    mark: Image,
    spinner: Spinner,
    status: Label,
    about: Label,
    last_backup: Label,
    skipped: Image,
    explanation: Label,
    buttons: HashMap<Action, Button>,
    /// The line between the texts and the buttons, hidden when there are no buttons.
    buttons_line: Separator,
    /// A notice in the explanation, such as "Ejecting BACKUP-A…".
    notice: Label,
    logo: Image,
}

/// A backup, from starting it until the user goes back to the start screen.
struct BackupState {
    /// The label of the medium.
    label: String,
    /// The udisks2 drive ID of the medium, to eject it.
    drive_id: String,
    /// The account's last backup to the medium before this run.
    previous: Option<LastBackup>,
    /// When the run ended, once it has.
    ended_at: Option<SystemTime>,
    /// Nothing once the run has ended.
    running: Option<RunningBackup>,
    tracker: Tracker,
    /// How the run ended, once it has. `Some(None)` means the worker stopped without saying,
    /// which can only be a crash.
    ended: Option<Option<Ended>>,
    /// The user closed the window and agreed to stop, so it closes once the run has ended.
    close_when_done: bool,
    /// Closes the question whether to stop, if the run ends while it is open.
    question: gio::Cancellable,
}

/// How far the format view has got.
enum FormatStage {
    /// The user is choosing. What they chose is kept in the widgets.
    Choosing,
    /// Formatting the medium that will be labelled `label`.
    Working {
        label: String,
    },
    Done {
        label: String,
        root: PathBuf,
        format: Format,
    },
    Failed {
        label: String,
        failure: Failure,
    },
}

/// The format view, from opening it until the user goes back.
struct FormatState {
    stage: FormatStage,
    /// The drives that may be formatted, as last listed.
    choices: Vec<DriveChoice>,
    /// The udisks2 drive ID of the chosen drive.
    drive: Option<String>,
    /// The user has typed a label, so it is no longer replaced by the suggestion.
    label_edited: bool,
}

/// How far the configuration editor has got.
enum SetupStage {
    Loading,
    Editing(Editor),
    /// The configuration can't be trusted, so only a new one can be started.
    Unreadable,
    NewerFormat,
    NotBabawus,
    /// The configuration couldn't be read, with what the system said.
    Failed(String),
}

/// What the configuration editor says below its contents.
enum SetupNotice {
    Saved,
    /// Already in words, because it may hold what the system said.
    Error(String),
}

/// The configuration editor, from opening it until the user goes back.
struct SetupState {
    /// The label of the medium.
    label: String,
    root: PathBuf,
    format: Format,
    hostname: String,
    stage: SetupStage,
    notice: Option<SetupNotice>,
    /// The stick is being read or written off the main thread.
    busy: bool,
    /// Counts the editors opened, so that what an older one read is dropped.
    generation: u64,
}

/// What the history or "What's backed up" screen is about. The texts are made again from it when
/// the language changes.
enum InfoView {
    /// The user whose backup to the medium is running, if any, and the runs once they have been
    /// read. `Some(None)` means the log could not be read.
    History {
        in_progress: Option<String>,
        runs: Option<Option<Vec<Run>>>,
    },
    /// What reading the medium found, once it is done.
    Coverage(Option<Gathered>),
}

/// The history or "What's backed up" screen, from opening it until the user goes back.
struct InfoState {
    /// The label of the medium.
    label: String,
    /// The udisks2 object path of the medium's file system, and where it is mounted.
    object_path: String,
    root: PathBuf,
    view: InfoView,
    /// Counts the screens opened, so that what an older one read is dropped.
    generation: u64,
}

struct Window {
    window: ApplicationWindow,
    /// The start screen, the backup screen, or the history or "What's backed up".
    screens: Stack,
    backup_page: BackupPage,
    backup: RefCell<Option<BackupState>>,
    info_page: InfoPage,
    info: RefCell<Option<InfoState>>,
    info_generation: Cell<u64>,
    format_page: FormatPage,
    format: RefCell<Option<FormatState>>,
    /// Set while Babawus itself changes the format view's widgets, so that their signals are
    /// ignored.
    format_updating: Cell<bool>,
    setup_page: SetupPage,
    choose_page: Rc<ChoosePage>,
    /// "Choose what to back up…" is showing, over the editor.
    choosing: Cell<bool>,
    setup: RefCell<Option<SetupState>>,
    setup_generation: Cell<u64>,
    /// Set while Babawus itself changes the editor's widgets, so that their signals are ignored.
    setup_updating: Cell<bool>,
    title: Label,
    advanced_marker: Label,
    menu: MenuButton,
    list: ListBox,
    rows: RefCell<Vec<RowWidgets>>,
    details: DetailsWidgets,

    sticks: Option<Sticks>,
    this: ThisProcess,
    username: String,
    settings_path: PathBuf,

    language: Cell<Language>,
    advanced: Cell<bool>,
    /// The media in the list, each with what checking it found out, once that is known.
    media: RefCell<Vec<(Medium, Option<Checked>)>>,
    /// The udisks2 drive IDs of the drives Babawus may format.
    formattable: RefCell<HashSet<String>>,
    /// The udisks2 object path of the selected medium's file system.
    selected: RefCell<Option<String>>,
    /// Counts the checks, so that the results of an older one are dropped.
    generation: Cell<u64>,
    /// Set while the list is being changed, so that its selection signals are ignored.
    updating: Cell<bool>,
    notice: RefCell<Option<Notice>>,
    /// The drive being ejected, if any.
    ejecting: RefCell<Option<String>>,
    /// A stick that is plugged in but not listed, for the empty page.
    unlisted: Cell<Option<Unlisted>>,
    /// The open log windows, by the udisks2 object path of their medium's file system: at most
    /// one per stick (issue 135).
    log_windows: RefCell<HashMap<String, Rc<LogWindow>>>,
    /// The Help window, while it is open.
    help_window: RefCell<Option<Rc<HelpWindow>>>,
    /// The open Restore windows, by the udisks2 object path of their medium's file system: at
    /// most one per stick (issue 161).
    restore_windows: RefCell<HashMap<String, Rc<RestoreWindow>>>,
    /// The sticks that files are being restored from, by the same path. "Start backup" is greyed
    /// out for them.
    restoring: RefCell<HashSet<String>>,
}

/// Opens the start screen.
pub fn show(app: &Application) {
    add_css();
    let settings_path = platform::settings_file();
    let language = current_language(&settings_path);
    let sticks = Sticks::connect()
        .inspect_err(|error| eprintln!("Babawus could not list the sticks: {error}"))
        .ok();
    let computer_id = this_computer_id(&RealFs).ok().flatten().unwrap_or_default();
    // Without these, no lock on a medium can be checked, and every medium counts as unlocked.
    let this = ThisProcess::read(&RealFs, &computer_id).unwrap_or_else(|error| {
        eprintln!("Babawus could not read details of this process: {error}");
        ThisProcess {
            computer_id: computer_id.clone(),
            boot_id: String::new(),
            process_id: std::process::id(),
            program: String::new(),
        }
    });

    let title = Label::new(None);
    let advanced_marker = Label::builder()
        .visible(false)
        .valign(Align::Center)
        .build();
    advanced_marker.add_css_class("babawus-advanced");
    let title_box = Box::new(Orientation::Horizontal, SPACING);
    title_box.append(&title);
    title_box.append(&advanced_marker);
    let menu = MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .build();
    let header = HeaderBar::new();
    header.set_title_widget(Some(&title_box));
    header.pack_end(&menu);

    let list = ListBox::builder()
        .selection_mode(SelectionMode::Single)
        .build();
    list.add_css_class("navigation-sidebar");
    let list_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .width_request(LIST_WIDTH)
        .child(&list)
        .build();

    let (details, details_view) = details_widgets();
    let content = Box::new(Orientation::Horizontal, 0);
    content.append(&list_scroll);
    content.append(&Separator::new(Orientation::Vertical));
    content.append(&details_view);
    let backup_page = BackupPage::new();
    // Each screen asks only for the width and height it needs itself. Otherwise the widest and
    // tallest screens, even hidden ones, set the window's least size: long texts, as in German,
    // push its width past what a small screen has, and a hidden "Choose what to back up…",
    // which doesn't scroll as a whole, made the window grow taller after Cancel (T-049).
    let screens = Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .build();
    screens.add_named(&content, Some("start"));
    screens.add_named(&backup_page.root, Some("backup"));
    let info_page = InfoPage::new();
    screens.add_named(&info_page.root, Some("info"));
    let format_page = FormatPage::new(platform::is_administrator());
    screens.add_named(&format_page.root, Some("format"));
    // The page passes what is pressed in its lists to the window, which only exists further
    // down.
    let setup_events: Rc<RefCell<Weak<Window>>> = Rc::new(RefCell::new(Weak::new()));
    let events = setup_events.clone();
    let setup_page = SetupPage::new(move |event| {
        if let Some(window) = events.borrow().upgrade() {
            window.setup_event(event);
        }
    });
    screens.add_named(&setup_page.root, Some("setup"));
    let choose_page = ChoosePage::new();
    screens.add_named(&choose_page.root, Some("choose"));

    let window = ApplicationWindow::builder()
        .application(app)
        // Opens at 700 × 768 with its title bar, assuming Full HD screens (issue 118). A stick's
        // buttons wrap onto centred lines when they don't fit on one (issue 165).
        .default_width(700)
        .default_height(768)
        .titlebar(&header)
        .child(&screens)
        .build();

    let this_window = Rc::new(Window {
        window,
        screens,
        backup_page,
        backup: RefCell::new(None),
        info_page,
        info: RefCell::new(None),
        info_generation: Cell::new(0),
        format_page,
        format: RefCell::new(None),
        format_updating: Cell::new(false),
        setup_page,
        choose_page,
        choosing: Cell::new(false),
        setup: RefCell::new(None),
        setup_generation: Cell::new(0),
        setup_updating: Cell::new(false),
        title,
        advanced_marker,
        menu,
        list,
        rows: RefCell::new(Vec::new()),
        details,
        sticks,
        this,
        username: glib::user_name().to_string_lossy().into_owned(),
        settings_path,
        language: Cell::new(language),
        advanced: Cell::new(false),
        media: RefCell::new(Vec::new()),
        formattable: RefCell::new(HashSet::new()),
        selected: RefCell::new(None),
        generation: Cell::new(0),
        updating: Cell::new(false),
        notice: RefCell::new(None),
        ejecting: RefCell::new(None),
        unlisted: Cell::new(None),
        log_windows: RefCell::new(HashMap::new()),
        help_window: RefCell::new(None),
        restore_windows: RefCell::new(HashMap::new()),
        restoring: RefCell::new(HashSet::new()),
    });
    *setup_events.borrow_mut() = Rc::downgrade(&this_window);
    this_window.connect_signals();
    this_window.add_actions();
    this_window.set_texts();
    this_window.media_changed();
    if let Some(sticks) = &this_window.sticks {
        let weak = Rc::downgrade(&this_window);
        sticks.connect_changed(move || {
            if let Some(window) = weak.upgrade() {
                window.media_changed();
            }
        });
    }
    // The window keeps its state alive until it is destroyed. The handlers only hold weak
    // references, so this is the one strong one.
    let state = RefCell::new(Some(this_window.clone()));
    this_window.window.connect_destroy(move |_| {
        // The log windows close with Babawus, which would otherwise go on running for them.
        if let Some(window) = state.take() {
            window.close_log_windows();
        }
    });
    this_window.window.present();
}

/// The languages read at start, before any window opens.
static LANGUAGES: std::sync::OnceLock<Languages> = std::sync::OnceLock::new();

/// Keeps the languages read at start, for every window. Called once, before any window opens.
pub fn set_languages(languages: Languages) {
    LANGUAGES
        .set(languages)
        .expect("the languages are only read once");
}

/// The languages read at start.
pub fn languages() -> &'static Languages {
    LANGUAGES
        .get()
        .expect("the languages are read before any window opens")
}

/// The language to show at the start: the user's choice, otherwise the system's.
pub fn current_language(settings_path: &std::path::Path) -> Language {
    let settings = Settings::load(&RealFs, settings_path);
    let system_names = glib::language_names();
    settings.language(languages(), system_names.iter().map(|name| name.as_str()))
}

fn add_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(CSS);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn wrapping_label() -> Label {
    Label::builder().wrap(true).xalign(0.0).build()
}

/// The details pane, and the widgets in it that change. From the top: the label, the stick's
/// details, a thin line, and the explanation, which starts with the status and holds every
/// message about the stick. See `ui-review/UiReview.md` item G01.
fn details_widgets() -> (DetailsWidgets, Overlay) {
    let title = Label::builder().xalign(0.0).build();
    title.add_css_class("babawus-title");
    let about = wrapping_label();
    about.add_css_class("dim-label");
    // As tall as the logo, so that the line below it passes under the logo.
    let heading = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_end(LOGO_ROOM)
        .height_request(LOGO_SIZE - MARGIN / 2)
        .build();
    heading.append(&title);
    heading.append(&about);

    let mark = icons::image(&icons::mark(start_screen::Mark::Ready), MARK_SIZE);
    let spinner = Spinner::builder().spinning(true).build();
    let status = Label::builder().xalign(0.0).build();
    let status_row = Box::new(Orientation::Horizontal, 8);
    status_row.append(&mark);
    status_row.append(&spinner);
    status_row.append(&status);
    let last_backup = Label::builder().xalign(0.0).build();
    let skipped = icons::image(&icons::mark(start_screen::Mark::Attention), MARK_SIZE);
    let last_backup_row = Box::new(Orientation::Horizontal, 8);
    last_backup_row.append(&last_backup);
    last_backup_row.append(&skipped);
    let explanation = wrapping_label();
    let notice = wrapping_label();
    notice.set_visible(false);
    let explained = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .build();
    explained.append(&status_row);
    explained.append(&last_backup_row);
    explained.append(&explanation);
    explained.append(&notice);

    // The everyday buttons, then the advanced ones, on one row that wraps like words when the
    // window is narrow, or the texts are long, as in German, so that the window needn't grow
    // (issue 134).
    let actions = action_row();
    let mut buttons = HashMap::new();
    for action in Action::ALL {
        let button = Button::builder().halign(Align::Start).build();
        actions.append(&button);
        buttons.insert(action, button);
    }
    let buttons_line = Separator::new(Orientation::Horizontal);

    let medium_page = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(SPACING)
        .build();
    medium_page.append(&heading);
    medium_page.append(&Separator::new(Orientation::Horizontal));
    medium_page.append(&explained);
    medium_page.append(&buttons_line);
    medium_page.append(&actions);

    let plug_in = Label::builder()
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .valign(Align::Center)
        .halign(Align::Center)
        .build();
    plug_in.add_css_class("babawus-plug-in");
    let centred_label = || {
        Label::builder()
            .wrap(true)
            .justify(gtk4::Justification::Center)
            .halign(Align::Center)
            .visible(false)
            .build()
    };
    let empty_advice = centred_label();
    let empty_notice = centred_label();

    let empty_prepare = Button::builder().halign(Align::Center).build();
    let empty_page = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(SPACING)
        .valign(Align::Center)
        .build();
    empty_page.append(&plug_in);
    empty_page.append(&empty_advice);
    empty_page.append(&empty_notice);
    empty_page.append(&empty_prepare);

    let pages = Stack::builder().vexpand(true).build();
    pages.add_named(&medium_page, Some("medium"));
    pages.add_named(&empty_page, Some("empty"));

    let pane = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_top(MARGIN)
        .margin_bottom(MARGIN)
        .margin_start(MARGIN)
        .margin_end(MARGIN)
        .hexpand(true)
        .build();
    pane.append(&pages);

    let logo = icons::image(&icons::logo_with_name(), LOGO_SIZE);
    logo.set_halign(Align::End);
    logo.set_valign(Align::Start);
    logo.set_margin_top(MARGIN / 2);
    logo.set_margin_end(MARGIN / 2);
    let overlay = Overlay::builder().child(&pane).build();
    overlay.add_overlay(&logo);

    let widgets = DetailsWidgets {
        pages,
        plug_in,
        empty_advice,
        empty_notice,
        empty_prepare,
        title,
        mark,
        spinner,
        status,
        about,
        last_backup,
        skipped,
        explanation,
        buttons,
        buttons_line,
        notice,
        logo,
    };
    (widgets, overlay)
}

impl Window {
    fn connect_signals(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.backup_page.cancel.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.ask_to_stop(false);
            }
        });
        let weak = Rc::downgrade(self);
        self.backup_page.eject.connect_clicked(move |_| {
            let Some(window) = weak.upgrade() else { return };
            let drive = window
                .backup
                .borrow()
                .as_ref()
                .map(|backup| (backup.drive_id.clone(), backup.label.clone()));
            if let Some((drive_id, label)) = drive {
                window.eject_drive(drive_id, label);
            }
        });
        let weak = Rc::downgrade(self);
        self.backup_page.back.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.back_to_start();
            }
        });
        let weak = Rc::downgrade(self);
        self.info_page.back.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.close_info();
            }
        });
        self.connect_format_signals();
        self.connect_setup_signals();
        // Closing the window during a run asks first, like Cancel. Closing it straight away
        // would end the run without its end entry in the log.
        let weak = Rc::downgrade(self);
        self.window.connect_close_request(move |_| {
            let Some(window) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            // Formatting can't be stopped, and closing would leave the medium half done.
            if window.formatting() {
                return glib::Propagation::Stop;
            }
            // Leaving with unsaved changes asks first. See issue 84.
            if window.setup_unsaved() {
                window.ask_to_leave_setup(AfterLeaving::CloseWindow);
                return glib::Propagation::Stop;
            }
            let running = window
                .backup
                .borrow()
                .as_ref()
                .is_some_and(|backup| backup.running.is_some());
            if !running {
                // The log windows close with Babawus, which would otherwise go on running for
                // them.
                window.close_log_windows();
                return glib::Propagation::Proceed;
            }
            window.ask_to_stop(true);
            glib::Propagation::Stop
        });
        let weak = Rc::downgrade(self);
        self.list.connect_row_selected(move |_, row| {
            let Some(window) = weak.upgrade() else { return };
            if window.updating.get() {
                return;
            }
            let selected = row.map(|row| row.widget_name().to_string());
            if *window.selected.borrow() != selected {
                *window.notice.borrow_mut() = None;
            }
            *window.selected.borrow_mut() = selected;
            window.show_details();
        });
        for (&action, button) in &self.details.buttons {
            let weak: Weak<Self> = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(window) = weak.upgrade() {
                    window.act(action);
                }
            });
        }
    }

    /// The menu's actions: Language, Advanced and About.
    fn add_actions(self: &Rc<Self>) {
        let language = gio::SimpleAction::new_stateful(
            "language",
            Some(glib::VariantTy::STRING),
            &self.language.get().code().to_variant(),
        );
        let weak = Rc::downgrade(self);
        language.connect_activate(move |action, parameter| {
            let Some(window) = weak.upgrade() else { return };
            let Some(code) = parameter.and_then(|parameter| parameter.str()) else {
                return;
            };
            let Some(chosen) = languages().find(code) else {
                return;
            };
            action.set_state(&code.to_variant());
            window.choose_language(chosen);
        });
        self.window.add_action(&language);

        let advanced = gio::SimpleAction::new_stateful("advanced", None, &false.to_variant());
        let weak = Rc::downgrade(self);
        advanced.connect_activate(move |action, _| {
            let Some(window) = weak.upgrade() else { return };
            if window.advanced.get() {
                window.turn_advanced_off(action);
            } else {
                window.ask_for_advanced(action);
            }
        });
        self.window.add_action(&advanced);

        // Each row's Eject, with the object path of its medium's file system (issue 134).
        let eject = gio::SimpleAction::new("eject-stick", Some(glib::VariantTy::STRING));
        let weak = Rc::downgrade(self);
        eject.connect_activate(move |_, parameter| {
            let Some(window) = weak.upgrade() else { return };
            let Some(object_path) = parameter.and_then(|parameter| parameter.str()) else {
                return;
            };
            let medium = window
                .media
                .borrow()
                .iter()
                .find(|(medium, _)| medium.volume.object_path == object_path)
                .map(|(medium, _)| medium.clone());
            if let Some(medium) = medium {
                window.eject_drive(medium.drive.id, medium.volume.label);
            }
        });
        self.window.add_action(&eject);

        // "Start backup", below the selected medium's row.
        let start = gio::SimpleAction::new("start-backup", None);
        let weak = Rc::downgrade(self);
        start.connect_activate(move |_, _| {
            if let Some(window) = weak.upgrade() {
                window.start_backup();
            }
        });
        self.window.add_action(&start);

        // "Show in the log", for a run in the history, by its run ID.
        let show_in_log = gio::SimpleAction::new("show-in-log", Some(glib::VariantTy::STRING));
        let weak = Rc::downgrade(self);
        show_in_log.connect_activate(move |_, parameter| {
            if let Some(window) = weak.upgrade()
                && let Some(run_id) = parameter.and_then(|parameter| parameter.str())
            {
                window.show_in_log(run_id);
            }
        });
        self.window.add_action(&show_in_log);

        let help = gio::SimpleAction::new("help", None);
        let weak = Rc::downgrade(self);
        help.connect_activate(move |_, _| {
            if let Some(window) = weak.upgrade() {
                window.show_help();
            }
        });
        self.window.add_action(&help);
        if let Some(app) = self.window.application() {
            app.set_accels_for_action("win.help", &["F1"]);
        }

        let about = gio::SimpleAction::new("about", None);
        let weak = Rc::downgrade(self);
        about.connect_activate(move |_, _| {
            if let Some(window) = weak.upgrade() {
                window.show_about();
            }
        });
        self.window.add_action(&about);
    }

    /// Turns Advanced on or off, and shows or hides what goes with it.
    fn set_advanced(&self, action: &gio::SimpleAction, on: bool) {
        action.set_state(&on.to_variant());
        self.advanced.set(on);
        self.set_texts();
    }

    /// Turns Advanced off, leaving an advanced page for the start page first: "Prepare a stick",
    /// or the setup with "Choose what to back up". Unsaved changes to the setup ask first, and
    /// "Keep editing" keeps Advanced on (issue 165). While a stick is formatted, Advanced is
    /// greyed out in the menu, so this isn't reached then.
    fn turn_advanced_off(self: &Rc<Self>, action: &gio::SimpleAction) {
        if self.formatting() {
            return;
        }
        if self.setup.borrow().is_some() {
            if self.setup_unsaved() {
                self.ask_to_leave_setup(AfterLeaving::AdvancedOff);
                return;
            }
            self.close_setup();
        }
        if self.format.borrow().is_some() {
            self.close_format();
        }
        self.set_advanced(action, false);
    }

    /// The menu's Advanced, a toggle.
    fn advanced_action(&self) -> Option<gio::SimpleAction> {
        self.window
            .lookup_action("advanced")
            .and_then(|action| action.downcast::<gio::SimpleAction>().ok())
    }

    /// Greys out Advanced in the menu while a stick is formatted, which can't be stopped.
    fn update_advanced_action(&self) {
        if let Some(action) = self.advanced_action() {
            action.set_enabled(!self.formatting());
        }
    }

    /// Warns before Advanced is turned on, and turns it on only after "Continue". "Cancel" has
    /// the focus, so Enter or Esc keeps it off (issue 152).
    fn ask_for_advanced(self: &Rc<Self>, action: &gio::SimpleAction) {
        let language = self.language.get();
        let dialog = AlertDialog::builder()
            .message(Text::AdvancedQuestion.get(language))
            .detail(Text::AdvancedDetail.get(language))
            .buttons([
                Text::Cancel.get(language),
                Text::AdvancedContinue.get(language),
            ])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        let weak = Rc::downgrade(self);
        let action = action.clone();
        dialog.choose(Some(&self.window), gio::Cancellable::NONE, move |answer| {
            if answer == Ok(1)
                && let Some(window) = weak.upgrade()
            {
                window.set_advanced(&action, true);
            }
        });
    }

    fn choose_language(&self, language: Language) {
        self.language.set(language);
        let help_window = self.help_window.borrow().clone();
        if let Some(help_window) = help_window {
            help_window.set_language(language);
        }
        let restore_windows: Vec<Rc<RestoreWindow>> =
            self.restore_windows.borrow().values().cloned().collect();
        for restore_window in restore_windows {
            restore_window.set_language(language);
        }
        let settings = Settings {
            language: Some(language.code().to_owned()),
        };
        // The choice is only remembered for next time, so this doesn't get in the user's way.
        if let Err(error) = settings.save(&RealFs, &self.settings_path) {
            eprintln!(
                "Babawus could not save {}: {error}",
                self.settings_path.display()
            );
        }
        self.set_texts();
    }

    fn menu_model(&self) -> gio::Menu {
        let language = self.language.get();
        let submenu = gio::Menu::new();
        for each in languages().all() {
            let item = gio::MenuItem::new(Some(each.name()), None);
            item.set_action_and_target_value(Some("win.language"), Some(&each.code().to_variant()));
            submenu.append_item(&item);
        }
        let menu = gio::Menu::new();
        menu.append_submenu(Some(Text::MenuLanguage.get(language)), &submenu);
        menu.append(Some(Text::MenuAdvanced.get(language)), Some("win.advanced"));
        menu.append(Some(Text::MenuHelp.get(language)), Some("win.help"));
        menu.append(Some(Text::MenuAbout.get(language)), Some("win.about"));
        menu
    }

    /// Opens the Help, or brings it to the front if it is open (issue 154).
    fn show_help(self: &Rc<Self>) {
        let open = self.help_window.borrow().clone();
        if let Some(help_window) = open {
            help_window.window.present();
            return;
        }
        let Some(app) = self.window.application() else {
            return;
        };
        let help_window = HelpWindow::open(&app, self.language.get());
        let weak = Rc::downgrade(self);
        help_window.window.connect_close_request(move |_| {
            if let Some(window) = weak.upgrade() {
                window.help_window.borrow_mut().take();
            }
            glib::Propagation::Proceed
        });
        *self.help_window.borrow_mut() = Some(help_window);
    }

    fn show_about(&self) {
        let language = self.language.get();
        let dialog = AboutDialog::builder()
            .transient_for(&self.window)
            .modal(true)
            .program_name("Babawus")
            .version(babawus_core::VERSION)
            .comments(text::about_comments(language))
            .logo(&icons::logo())
            .website("https://github.com/ziggymax/babawus")
            .build();
        // GTK gives the first selectable text the focus, which selects all of it: the name.
        // Nothing in the dialog needs copying (`ui-review/UiReview.md` W05).
        unselectable(dialog.upcast_ref());
        // The comment is markup, which `AboutDialog` would show as it is (issue 132).
        use_markup_for(dialog.upcast_ref(), &text::about_comments(language));
        dialog.present();
    }

    /// Sets every text again, for example after the language changed.
    fn set_texts(&self) {
        let language = self.language.get();
        let title = Text::WindowTitle.get(language);
        self.window.set_title(Some(title));
        self.title.set_label(title);
        self.title.add_css_class("title");
        self.advanced_marker
            .set_label(Text::AdvancedMarker.get(language));
        self.advanced_marker
            .set_tooltip_text(Some(Text::AdvancedMarkerTooltip.get(language)));
        self.advanced_marker.set_visible(self.advanced.get());
        self.menu.set_menu_model(Some(&self.menu_model()));
        self.menu.set_tooltip_text(Some(Text::Menu.get(language)));
        self.details
            .logo
            .set_tooltip_text(Some(Text::Logo.get(language)));
        for (&action, button) in &self.details.buttons {
            button.set_label(action.label(language));
        }
        self.details
            .empty_prepare
            .set_label(Action::Prepare.label(language));
        self.format_page.set_texts(language);
        self.update_rows();
        self.show_details();
        self.show_backup();
        self.show_info();
        self.show_format();
        self.show_setup();
        if self.choosing.get() {
            self.choose_page.set_language(language);
        }
    }

    /// The sticks changed, or Babawus has just started: the list is read again, and each
    /// medium is checked off the main thread.
    fn media_changed(self: &Rc<Self>) {
        let Some(sticks) = &self.sticks else {
            return;
        };
        let snapshot = sticks.snapshot();
        self.unlisted.set(start_screen::unlisted(&snapshot));
        *self.formattable.borrow_mut() = snapshot
            .formattable()
            .into_iter()
            .map(|target| target.drive.id)
            .collect();
        // What an earlier check found stays until the new one is done, so that the rows don't
        // flicker.
        let old = std::mem::take(&mut *self.media.borrow_mut());
        let media: Vec<_> = snapshot
            .media()
            .into_iter()
            .map(|medium| {
                let checked = old
                    .iter()
                    .find(|(each, _)| each == &medium)
                    .and_then(|(_, checked)| checked.clone());
                (medium, checked)
            })
            .collect();
        // A notice such as "You can pull out BACKUP-A now." has done its job once a stick is
        // plugged in.
        let path = |(medium, _): &(Medium, Option<Checked>)| medium.volume.object_path.clone();
        let before: HashSet<String> = old.iter().map(path).collect();
        if media.iter().any(|entry| !before.contains(&path(entry)))
            && self.ejecting.borrow().is_none()
        {
            *self.notice.borrow_mut() = None;
        }
        let to_check: Vec<Medium> = media.iter().map(|(medium, _)| medium.clone()).collect();
        // A log window closes when its stick has gone (issue 135).
        let gone: Vec<Rc<LogWindow>> = {
            let mut windows = self.log_windows.borrow_mut();
            let paths: HashSet<String> = media.iter().map(path).collect();
            let gone: Vec<String> = windows
                .keys()
                .filter(|key| !paths.contains(*key))
                .cloned()
                .collect();
            gone.iter().filter_map(|key| windows.remove(key)).collect()
        };
        for log_window in gone {
            log_window.window.close();
        }
        // So does a Restore window, stopping a restore that is going on (issue 161).
        let gone: Vec<Rc<RestoreWindow>> = {
            let mut windows = self.restore_windows.borrow_mut();
            let paths: HashSet<String> = media.iter().map(path).collect();
            let gone: Vec<String> = windows
                .keys()
                .filter(|key| !paths.contains(*key))
                .cloned()
                .collect();
            gone.iter().filter_map(|key| windows.remove(key)).collect()
        };
        for restore_window in gone {
            restore_window.stop();
            restore_window.window.close();
        }
        *self.media.borrow_mut() = media;
        self.update_rows();
        self.show_details();
        self.update_format_drives();

        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let this = self.this.clone();
        let username = self.username.clone();
        let checking = gio::spawn_blocking(move || {
            let viewer = Viewer {
                system: &RealFs,
                this: &this,
                username: &username,
            };
            to_check
                .into_iter()
                .map(|medium| {
                    let checked = viewer.check(&RealFs, &medium);
                    (medium, checked)
                })
                .collect::<Vec<_>>()
        });
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            // The check can only fail by panicking, which has been reported already.
            let Ok(results) = checking.await else { return };
            if let Some(window) = weak.upgrade() {
                window.checked(generation, results);
            }
        });
    }

    /// A check has finished. Its results are dropped if a newer check has started since.
    fn checked(&self, generation: u64, results: Vec<(Medium, Checked)>) {
        if self.generation.get() != generation {
            return;
        }
        for (medium, checked) in results {
            if let Some(entry) = self
                .media
                .borrow_mut()
                .iter_mut()
                .find(|(each, _)| *each == medium)
            {
                entry.1 = Some(checked);
            }
        }
        self.update_rows();
        self.show_details();
    }

    /// Brings the list in line with the media, and selects the medium that should be.
    fn update_rows(&self) {
        let language = self.language.get();
        let media = self.media.borrow();
        self.updating.set(true);
        let mut rows = self.rows.borrow_mut();
        let same_media = rows.len() == media.len()
            && rows
                .iter()
                .zip(media.iter())
                .all(|(row, (medium, _))| row.row.widget_name() == medium.volume.object_path);
        if !same_media {
            self.list.remove_all();
            *rows = media
                .iter()
                .map(|(medium, _)| {
                    let row = new_row(&medium.volume.object_path);
                    self.list.append(&row.row);
                    row
                })
                .collect();
        }
        for (row, (medium, checked)) in rows.iter().zip(media.iter()) {
            let shown = start_screen::row(medium, checked.as_ref(), language);
            row.label.set_label(&shown.label);
            row.stick.set_paintable(Some(&icons::stick(shown.babawus)));
            row.row.set_tooltip_text(Some(&shown.tooltip));
            row.spinner.set_visible(shown.checking);
            row.eject.set_tooltip_text(Some(&shown.eject_tooltip));
            row.start_backup.set_label(Text::StartBackup.get(language));
        }
        let selected = self.selected.borrow().clone();
        let selected = start_screen::selection(&media, selected.as_deref())
            .map(|medium| medium.volume.object_path.clone());
        let row = selected.as_ref().and_then(|selected| {
            rows.iter()
                .find(|row| row.row.widget_name() == selected.as_str())
        });
        match row {
            Some(row) => self.list.select_row(Some(&row.row)),
            None => self.list.unselect_all(),
        }
        *self.selected.borrow_mut() = selected;
        self.updating.set(false);
    }

    /// Shows the details of the selected medium.
    fn show_details(&self) {
        let language = self.language.get();
        let widgets = &self.details;
        // The notice goes in the explanation, or below the empty page's text once the stick
        // has left the list.
        let notice = self
            .notice
            .borrow()
            .as_ref()
            .map(|notice| notice.text.fill(language, &[("label", &notice.label)]));
        for label in [&widgets.notice, &widgets.empty_notice] {
            label.set_label(notice.as_deref().unwrap_or_default());
            label.set_visible(notice.is_some());
        }
        // Only one stick is ejected at a time. "Start backup" shows again below the selected
        // stick further down, if it is a Babawus stick.
        let ejecting = self.ejecting.borrow().is_some();
        for row in self.rows.borrow().iter() {
            row.eject.set_sensitive(!ejecting);
            row.start_backup.set_visible(false);
        }
        let media = self.media.borrow();
        let selected = self.selected.borrow();
        let Some((medium, checked)) = selected.as_ref().and_then(|selected| {
            media
                .iter()
                .find(|(medium, _)| &medium.volume.object_path == selected)
        }) else {
            let (text, advice) = start_screen::empty_text(
                self.sticks.is_some(),
                !media.is_empty(),
                self.unlisted.get(),
            );
            widgets.empty_advice.set_label(
                advice
                    .map(|advice| advice.get(language))
                    .unwrap_or_default(),
            );
            widgets.empty_advice.set_visible(advice.is_some());
            widgets.plug_in.set_label(text.get(language));
            widgets
                .empty_prepare
                .set_visible(self.advanced.get() && !self.formattable.borrow().is_empty());
            widgets.pages.set_visible_child_name("empty");
            return;
        };
        widgets.pages.set_visible_child_name("medium");
        let shown = Shown {
            medium,
            checked: checked.as_ref(),
            formattable: !self.formattable.borrow().is_empty(),
        };
        let details = start_screen::details(
            &shown,
            SystemTime::now(),
            local_clock,
            language,
            self.advanced.get(),
        );
        widgets.title.set_label(&details.label);
        widgets.status.set_label(&details.status);
        widgets.spinner.set_visible(details.mark.is_none());
        widgets.mark.set_visible(details.mark.is_some());
        if let Some(mark) = details.mark {
            widgets.mark.set_paintable(Some(&icons::mark(mark)));
        }
        widgets.about.set_label(&details.about);
        match &details.last_backup {
            Some((text, skipped)) => {
                widgets.last_backup.set_label(text);
                widgets.last_backup.set_visible(true);
                widgets.skipped.set_visible(*skipped);
                widgets
                    .skipped
                    .set_tooltip_text(Some(Text::SkippedMark.get(language)));
            }
            None => {
                widgets.last_backup.set_visible(false);
                widgets.skipped.set_visible(false);
            }
        }
        widgets
            .explanation
            .set_label(details.explanation.as_deref().unwrap_or_default());
        widgets
            .explanation
            .set_visible(details.explanation.is_some());
        for (action, button) in &widgets.buttons {
            match details.buttons.iter().find(|shown| shown.action == *action) {
                Some(shown) => {
                    button.set_visible(true);
                    button.set_tooltip_text(Some(&shown.tooltip));
                }
                None => button.set_visible(false),
            }
        }
        widgets
            .buttons_line
            .set_visible(!details.buttons.is_empty());
        let rows = self.rows.borrow();
        let row = rows
            .iter()
            .find(|row| row.row.widget_name() == medium.volume.object_path.as_str());
        if let (Some(row), Some(start)) = (row, &details.start_backup) {
            // Not while files are being restored from the stick (issue 161).
            let restoring = self.restoring.borrow().contains(&medium.volume.object_path);
            row.start_backup.set_visible(true);
            row.start_backup.set_sensitive(start.enabled && !restoring);
            let tooltip = if restoring {
                Text::RestoreRunning.get(language).to_owned()
            } else {
                start.tooltip.clone()
            };
            row.start_backup.set_tooltip_text(Some(&tooltip));
        }
    }

    fn act(self: &Rc<Self>, action: Action) {
        // The buttons for the other actions are greyed out until their screens are built.
        match action {
            Action::ShowHistory => self.open_history(),
            Action::WhatsBackedUp => self.open_coverage(),
            Action::Restore => self.open_restore(),
            Action::Prepare => self.open_format(),
            Action::ChangeSetup => {
                if let Some(medium) = self.selected_medium()
                    && let Some(format) = medium.volume.format()
                {
                    self.open_setup(medium.volume.label, medium.root, format);
                }
            }
        }
    }

    /// Opens the history of the selected medium, and reads its log off the main thread.
    fn open_history(self: &Rc<Self>) {
        let Some(medium) = self.selected_medium() else {
            return;
        };
        let in_progress = self.media.borrow().iter().find_map(|(each, checked)| {
            match checked.as_ref().map(|checked| &checked.status) {
                Some(Status::InProgress { user }) if each == &medium => Some(user.clone()),
                _ => None,
            }
        });
        let view = InfoView::History {
            in_progress,
            runs: None,
        };
        let generation = self.open_info(&medium, view);
        let root = medium.root.clone();
        let reading = gio::spawn_blocking(move || read_runs(&RealFs, &root).ok());
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let runs = reading.await.ok().flatten();
            if let Some(window) = weak.upgrade() {
                window.info_read(generation, |view| {
                    if let InfoView::History { runs: read, .. } = view {
                        *read = Some(runs);
                    }
                });
            }
        });
    }

    /// Closes every log window, and the Help, which close with Babawus.
    fn close_log_windows(&self) {
        let help_window = self.help_window.borrow_mut().take();
        if let Some(help_window) = help_window {
            help_window.window.close();
        }
        let open: Vec<Rc<LogWindow>> = self
            .log_windows
            .borrow_mut()
            .drain()
            .map(|(_, open)| open)
            .collect();
        for log_window in open {
            log_window.window.close();
        }
        let open: Vec<Rc<RestoreWindow>> = self
            .restore_windows
            .borrow_mut()
            .drain()
            .map(|(_, open)| open)
            .collect();
        for restore_window in open {
            restore_window.stop();
            restore_window.window.close();
        }
    }

    /// Shows the run `run_id` of the history's medium in its log window, which opens if it
    /// isn't open yet.
    fn show_in_log(self: &Rc<Self>, run_id: &str) {
        let Some((label, object_path, root)) = self.info.borrow().as_ref().map(|info| {
            (
                info.label.clone(),
                info.object_path.clone(),
                info.root.clone(),
            )
        }) else {
            return;
        };
        let open = self.log_windows.borrow().get(&object_path).cloned();
        if let Some(log_window) = open {
            log_window.show_run(run_id);
            return;
        }
        let Some(app) = self.window.application() else {
            return;
        };
        let log_window = LogWindow::open(&app, &label, root, run_id, self.language.get());
        let weak = Rc::downgrade(self);
        let key = object_path.clone();
        log_window.window.connect_close_request(move |_| {
            if let Some(window) = weak.upgrade() {
                window.log_windows.borrow_mut().remove(&key);
            }
            glib::Propagation::Proceed
        });
        self.log_windows
            .borrow_mut()
            .insert(object_path, log_window);
    }

    /// Opens the Restore window for the selected medium, or brings it to the front if it is open
    /// already (issue 161). Finding the backup only reads the setup, which is small.
    fn open_restore(self: &Rc<Self>) {
        let Some(medium) = self.selected_medium() else {
            return;
        };
        let object_path = medium.volume.object_path.clone();
        let open = self.restore_windows.borrow().get(&object_path).cloned();
        if let Some(open) = open {
            open.window.present();
            return;
        }
        let Some(format) = medium.volume.format() else {
            return;
        };
        let viewer = Viewer {
            system: &RealFs,
            this: &self.this,
            username: &self.username,
        };
        let language = self.language.get();
        let Some(folder) = viewer.backup_folder(&RealFs, &medium.root) else {
            *self.notice.borrow_mut() = Some(Notice {
                text: Text::RestoreCantRead,
                label: medium.volume.label.clone(),
            });
            self.show_details();
            return;
        };
        let Some(app) = self.window.application() else {
            return;
        };
        let weak = Rc::downgrade(self);
        let drive_id = medium.drive.id.clone();
        let backup_running =
            std::boxed::Box::new(move || {
                weak.upgrade().is_some_and(|window| {
                    window.backup.borrow().as_ref().is_some_and(|backup| {
                        backup.running.is_some() && backup.drive_id == drive_id
                    })
                })
            });
        let weak = Rc::downgrade(self);
        let busy_path = object_path.clone();
        let on_busy = std::boxed::Box::new(move |busy: bool| {
            let Some(window) = weak.upgrade() else { return };
            if busy {
                window.restoring.borrow_mut().insert(busy_path.clone());
            } else {
                window.restoring.borrow_mut().remove(&busy_path);
            }
            window.show_details();
        });
        let restore_window = RestoreWindow::open(
            &app,
            &medium.volume.label,
            &self.username,
            folder,
            format,
            language,
            backup_running,
            on_busy,
        );
        let weak = Rc::downgrade(self);
        let closed_path = object_path.clone();
        // Not on "destroy": this list holds the window, so it would never be destroyed, and opening
        // it again would show the closed window (T-097). The Restore window's own handler runs
        // first, and while it keeps the window open to ask "Stop restoring?", this one doesn't run.
        restore_window.window.connect_close_request(move |_| {
            if let Some(window) = weak.upgrade() {
                window.restore_windows.borrow_mut().remove(&closed_path);
            }
            glib::Propagation::Proceed
        });
        self.restore_windows
            .borrow_mut()
            .insert(object_path, restore_window);
    }

    /// Opens "What's backed up" for the selected medium, and reads it off the main thread.
    fn open_coverage(self: &Rc<Self>) {
        let Some(medium) = self.selected_medium() else {
            return;
        };
        let generation = self.open_info(&medium, InfoView::Coverage(None));
        let this = self.this.clone();
        let username = self.username.clone();
        let root = medium.root.clone();
        let home = platform::home_folder();
        let reading = gio::spawn_blocking(move || {
            let viewer = Viewer {
                system: &RealFs,
                this: &this,
                username: &username,
            };
            viewer.coverage(&RealFs, &root, &home)
        });
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            // Reading can only fail by panicking, which has been reported already.
            let gathered = reading.await.unwrap_or(Gathered::Unreadable);
            if let Some(window) = weak.upgrade() {
                window.info_read(generation, |view| {
                    *view = InfoView::Coverage(Some(gathered));
                });
            }
        });
    }

    /// Switches to the history or "What's backed up" screen for `medium`. Returns the number
    /// that tells what is read for this screen from what was read for an earlier one.
    fn open_info(&self, medium: &Medium, view: InfoView) -> u64 {
        let generation = self.info_generation.get() + 1;
        self.info_generation.set(generation);
        *self.notice.borrow_mut() = None;
        *self.info.borrow_mut() = Some(InfoState {
            label: medium.volume.label.clone(),
            object_path: medium.volume.object_path.clone(),
            root: medium.root.clone(),
            view,
            generation,
        });
        self.show_screen("info");
        self.show_info();
        generation
    }

    /// Something has been read for the screen numbered `generation`. It is dropped if the user
    /// has gone back since.
    fn info_read(&self, generation: u64, change: impl FnOnce(&mut InfoView)) {
        if let Some(info) = self.info.borrow_mut().as_mut()
            && info.generation == generation
        {
            change(&mut info.view);
        }
        self.show_info();
    }

    fn show_info(&self) {
        let language = self.language.get();
        let info = self.info.borrow();
        let Some(info) = info.as_ref() else {
            return;
        };
        let page = &self.info_page;
        match &info.view {
            InfoView::History { runs: None, .. } => {
                page.show_loading(&info.label, Text::HistoryHeading, language);
            }
            InfoView::History {
                in_progress,
                runs: Some(runs),
            } => {
                let rows = runs.as_ref().map(|runs| {
                    let now = SystemTime::now();
                    history(runs, in_progress.as_deref(), now, local_clock, language)
                });
                page.show_history(&info.label, rows.as_deref(), self.advanced.get(), language);
            }
            InfoView::Coverage(None) => {
                page.show_loading(&info.label, Text::CoverageHeading, language);
            }
            InfoView::Coverage(Some(gathered)) => {
                let shown = coverage_screen(gathered, &self.username, language);
                page.show_coverage(&info.label, &shown, language);
            }
        }
    }

    fn connect_setup_signals(self: &Rc<Self>) {
        let page = &self.setup_page;
        let weak = Rc::downgrade(self);
        page.choose.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.open_choose();
            }
        });
        let weak = Rc::downgrade(self);
        self.choose_page.done.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.close_choose(true);
            }
        });
        let weak = Rc::downgrade(self);
        self.choose_page.cancel.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.close_choose(false);
            }
        });
        let weak = Rc::downgrade(self);
        page.save.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.save_setup(None);
            }
        });
        let weak = Rc::downgrade(self);
        page.back.connect_clicked(move |_| {
            let Some(window) = weak.upgrade() else { return };
            if window.setup_unsaved() {
                window.ask_to_leave_setup(AfterLeaving::Nothing);
            } else {
                window.close_setup();
            }
        });
        let weak = Rc::downgrade(self);
        page.start_new.connect_clicked(move |_| {
            let Some(window) = weak.upgrade() else { return };
            let language = window.language.get();
            let weak = Rc::downgrade(&window);
            window.confirm(
                Text::SetupStartNewQuestion.get(language),
                Text::SetupStartNewDetail.get(language),
                Text::SetupStartNew.get(language).trim_end_matches('…'),
                move || {
                    if let Some(window) = weak.upgrade() {
                        window.start_new_setup();
                    }
                },
            );
        });
        let weak = Rc::downgrade(self);
        self.format_page.add.connect_clicked(move |_| {
            let Some(window) = weak.upgrade() else { return };
            let done = match window.format.borrow().as_ref().map(|format| &format.stage) {
                Some(FormatStage::Done {
                    label,
                    root,
                    format,
                }) => Some((label.clone(), root.clone(), *format)),
                _ => None,
            };
            if let Some((label, root, format)) = done {
                window.close_format();
                window.open_setup(label, root, format);
            }
        });
    }

    /// Asks `question`, with `detail`, and calls `confirmed` if the answer is `yes`.
    fn confirm(&self, question: &str, detail: &str, yes: &str, confirmed: impl FnOnce() + 'static) {
        let language = self.language.get();
        let dialog = AlertDialog::builder()
            .message(question)
            .detail(detail)
            .buttons([Text::Cancel.get(language), yes])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        dialog.choose(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |answer| {
                if answer == Ok(1) {
                    confirmed();
                }
            },
        );
    }

    /// Shows `message` in a pop-up with OK, and calls `closed` once it is closed.
    fn tell(&self, message: &str, closed: impl FnOnce() + 'static) {
        let language = self.language.get();
        let dialog = AlertDialog::builder()
            .message(message)
            .buttons([Text::Ok.get(language)])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        dialog.choose(Some(&self.window), None::<&gio::Cancellable>, move |_| {
            closed()
        });
    }

    fn setup_unsaved(&self) -> bool {
        if self.choosing.get() && self.choose_page.changed() {
            return true;
        }
        self.setup
            .borrow()
            .as_ref()
            .is_some_and(|setup| match &setup.stage {
                SetupStage::Editing(editor) => editor.unsaved(),
                _ => false,
            })
    }

    /// Asks whether to leave the editor without saving, and then does `after` too.
    fn ask_to_leave_setup(self: &Rc<Self>, after: AfterLeaving) {
        let language = self.language.get();
        let dialog = AlertDialog::builder()
            .message(Text::SetupDiscardQuestion.get(language))
            .detail(Text::SetupDiscardDetail.get(language))
            .buttons([
                Text::SetupKeepEditing.get(language),
                Text::SetupDiscard.get(language),
            ])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        let weak = Rc::downgrade(self);
        dialog.choose(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |answer| {
                let Some(window) = weak.upgrade() else { return };
                if answer != Ok(1) {
                    return;
                }
                window.close_setup();
                match after {
                    AfterLeaving::Nothing => {}
                    AfterLeaving::CloseWindow => window.window.close(),
                    AfterLeaving::AdvancedOff => {
                        if let Some(action) = window.advanced_action() {
                            window.set_advanced(&action, false);
                        }
                    }
                }
            },
        );
    }

    /// Opens the configuration editor for the medium labelled `label` at `root`, and reads its
    /// configuration off the main thread.
    fn open_setup(self: &Rc<Self>, label: String, root: PathBuf, format: Format) {
        if self.setup.borrow().is_some() {
            return;
        }
        let generation = self.setup_generation.get() + 1;
        self.setup_generation.set(generation);
        *self.notice.borrow_mut() = None;
        // Only shown in the configuration, so a missing hostname stops nothing.
        let hostname = platform::hostname(&RealFs).unwrap_or_default();
        *self.setup.borrow_mut() = Some(SetupState {
            label,
            root: root.clone(),
            format,
            hostname,
            stage: SetupStage::Loading,
            notice: None,
            busy: true,
            generation,
        });
        self.show_screen("setup");
        self.show_setup();
        let reading = gio::spawn_blocking(move || {
            let medium = setup_screen::Medium {
                files: &RealFs,
                root: &root,
                format,
            };
            match Editor::open(&medium) {
                Ok(Opened::Editing(editor)) => SetupStage::Editing(editor),
                Ok(Opened::Unreadable) => SetupStage::Unreadable,
                Ok(Opened::NewerFormat) => SetupStage::NewerFormat,
                Ok(Opened::NotBabawus) => SetupStage::NotBabawus,
                Err(error) => SetupStage::Failed(error.to_string()),
            }
        });
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            // Reading can only fail by panicking, which has been reported already.
            let stage = reading
                .await
                .unwrap_or_else(|_| SetupStage::Failed(String::new()));
            if let Some(window) = weak.upgrade() {
                window.setup_read(generation, stage, None);
            }
        });
    }

    /// Reading or writing for the editor numbered `generation` has finished. It is dropped if
    /// the user has gone back since.
    fn setup_read(&self, generation: u64, stage: SetupStage, notice: Option<SetupNotice>) {
        if let Some(setup) = self.setup.borrow_mut().as_mut()
            && setup.generation == generation
        {
            setup.stage = stage;
            setup.notice = notice;
            setup.busy = false;
        }
        self.show_setup();
    }

    /// Replaces a configuration that can't be trusted with a new one, once saved.
    fn start_new_setup(self: &Rc<Self>) {
        let (root, format, generation) = {
            let mut setup = self.setup.borrow_mut();
            let Some(setup) = setup.as_mut() else { return };
            setup.busy = true;
            (setup.root.clone(), setup.format, setup.generation)
        };
        self.show_setup();
        let starting = gio::spawn_blocking(move || {
            let medium = setup_screen::Medium {
                files: &RealFs,
                root: &root,
                format,
            };
            match Editor::start_new(&medium) {
                Ok(editor) => SetupStage::Editing(editor),
                Err(error) => SetupStage::Failed(error.to_string()),
            }
        });
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let stage = starting
                .await
                .unwrap_or_else(|_| SetupStage::Failed(String::new()));
            if let Some(window) = weak.upgrade() {
                window.setup_read(generation, stage, None);
            }
        });
    }

    /// Something was pressed in the editor's lists.
    fn setup_event(self: &Rc<Self>, event: SetupEvent) {
        let language = self.language.get();
        match event {
            SetupEvent::Numbers {
                id,
                username,
                older_versions,
                run_check_days,
            } => {
                if let Some(setup) = self.setup.borrow_mut().as_mut()
                    && let SetupStage::Editing(editor) = &mut setup.stage
                {
                    editor.set_older_versions(&id, &username, older_versions);
                    editor.set_run_check_days(&id, &username, run_check_days);
                    setup.notice = None;
                }
                self.show_setup();
            }
            SetupEvent::Here(HereAction::Add) => {
                self.change_setup(|editor, here| editor.add_here(here).is_ok());
            }
            SetupEvent::Here(HereAction::LinkComputer(folder)) => {
                let question =
                    Text::SetupLinkComputerQuestion.fill(language, &[("folder", &folder)]);
                let detail = Text::SetupLinkComputerDetail.fill(language, &[("folder", &folder)]);
                let weak = Rc::downgrade(self);
                self.confirm(
                    &question,
                    &detail,
                    Text::SetupLinkConfirm.get(language),
                    move || {
                        if let Some(window) = weak.upgrade() {
                            window.change_setup(|editor, here| {
                                editor.link_computer(&folder, here).is_ok()
                            });
                        }
                    },
                );
            }
            SetupEvent::Here(HereAction::LinkAccount(old)) => {
                let question = Text::SetupLinkAccountQuestion.fill(language, &[("old", &old)]);
                let detail = Text::SetupLinkAccountDetail
                    .fill(language, &[("old", &old), ("user", &self.username)]);
                let weak = Rc::downgrade(self);
                self.confirm(
                    &question,
                    &detail,
                    Text::SetupLinkConfirm.get(language),
                    move || {
                        if let Some(window) = weak.upgrade() {
                            window.save_setup(Some(old));
                        }
                    },
                );
            }
            SetupEvent::RemoveComputer { id, folder } => {
                let question =
                    Text::SetupRemoveComputerQuestion.fill(language, &[("folder", &folder)]);
                let weak = Rc::downgrade(self);
                self.confirm(
                    &question,
                    Text::SetupRemoveDetail.get(language),
                    Text::SetupRemoveConfirm.get(language),
                    move || {
                        if let Some(window) = weak.upgrade() {
                            window.change_setup(|editor, _| editor.remove_computer(&id).is_ok());
                        }
                    },
                );
            }
            SetupEvent::RemoveAccount {
                id,
                folder,
                username,
            } => {
                let question = Text::SetupRemoveAccountQuestion
                    .fill(language, &[("user", &username), ("folder", &folder)]);
                let weak = Rc::downgrade(self);
                self.confirm(
                    &question,
                    Text::SetupRemoveDetail.get(language),
                    Text::SetupRemoveConfirm.get(language),
                    move || {
                        if let Some(window) = weak.upgrade() {
                            window.change_setup(|editor, _| {
                                editor.remove_account(&id, &username).is_ok()
                            });
                        }
                    },
                );
            }
        }
    }

    /// Makes a change that waits for Save. `change` says whether it could be made.
    fn change_setup(&self, change: impl FnOnce(&mut Editor, &Here) -> bool) {
        {
            let mut setup = self.setup.borrow_mut();
            let Some(setup) = setup.as_mut() else { return };
            let SetupStage::Editing(editor) = &mut setup.stage else {
                return;
            };
            let here = Here {
                system: &RealFs,
                this: &self.this,
                username: &self.username,
                hostname: &setup.hostname,
            };
            let changed = change(editor, &here);
            let text = Text::SetupInvalid.get(self.language.get()).to_owned();
            setup.notice = (!changed).then_some(SetupNotice::Error(text));
        }
        self.show_setup();
    }

    /// Saves the configuration off the main thread. With `link_to`, this account is first linked
    /// to its earlier name, which saves too.
    fn save_setup(self: &Rc<Self>, link_to: Option<String>) {
        let (mut editor, root, format, hostname, generation) = {
            let mut setup = self.setup.borrow_mut();
            let Some(setup) = setup.as_mut() else { return };
            let SetupStage::Editing(editor) = &setup.stage else {
                return;
            };
            if setup.busy {
                return;
            }
            setup.busy = true;
            setup.notice = None;
            (
                editor.clone(),
                setup.root.clone(),
                setup.format,
                setup.hostname.clone(),
                setup.generation,
            )
        };
        self.show_setup();
        let this = self.this.clone();
        let username = self.username.clone();
        let saving = gio::spawn_blocking(move || {
            let medium = setup_screen::Medium {
                files: &RealFs,
                root: &root,
                format,
            };
            let here = Here {
                system: &RealFs,
                this: &this,
                username: &username,
                hostname: &hostname,
            };
            let result = match &link_to {
                Some(old) => editor.link_account(&medium, &here, old),
                None => editor.save(&medium, &here),
            };
            (editor, result)
        });
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(window) = weak.upgrade() else { return };
            let language = window.language.get();
            let Ok((editor, result)) = saving.await else {
                window.setup_failed(generation, String::new());
                return;
            };
            let notice = match result {
                Ok(()) => SetupNotice::Saved,
                Err(error) => {
                    if let SaveError::Io(error) = &error {
                        eprintln!("Babawus could not save the setup: {error}");
                    }
                    SetupNotice::Error(save_error_text(&error, language))
                }
            };
            window.setup_read(generation, SetupStage::Editing(editor), Some(notice));
        });
    }

    fn setup_failed(&self, generation: u64, error: String) {
        let notice = SetupNotice::Error(error.clone());
        self.setup_read(generation, SetupStage::Failed(error), Some(notice));
    }

    fn show_setup(&self) {
        let language = self.language.get();
        let setup = self.setup.borrow();
        let Some(setup) = setup.as_ref() else {
            return;
        };
        let notice = setup.notice.as_ref().map(|notice| match notice {
            SetupNotice::Saved => Text::SetupSaved.get(language).to_owned(),
            SetupNotice::Error(text) => text.clone(),
        });
        let page = &self.setup_page;
        let notice = notice.as_deref();
        let message = |text: Text, start_new| {
            page.show_instead(
                &setup.label,
                Instead::Message(text.get(language), start_new),
                notice,
                language,
            );
        };
        match &setup.stage {
            SetupStage::Loading => {
                page.show_instead(&setup.label, Instead::Loading, notice, language);
            }
            SetupStage::Unreadable => message(Text::SetupUnreadable, !setup.busy),
            SetupStage::NewerFormat => message(Text::SetupNewerFormat, false),
            SetupStage::NotBabawus => message(Text::SetupNotBabawus, false),
            SetupStage::Failed(error) => {
                let text = Text::SetupSaveFailed.fill(language, &[("error", error)]);
                page.show_instead(&setup.label, Instead::Message(&text, false), None, language);
            }
            SetupStage::Editing(editor) => {
                let here = Here {
                    system: &RealFs,
                    this: &self.this,
                    username: &self.username,
                    hostname: &setup.hostname,
                };
                let shown = overview(editor, &here, language);
                self.setup_updating.set(true);
                page.show_overview(&setup.label, &shown, notice, setup.busy, language);
                self.setup_updating.set(false);
            }
        }
    }

    /// Opens "Choose what to back up…" for this account, over the editor.
    fn open_choose(&self) {
        let opened = {
            let setup = self.setup.borrow();
            let Some(setup) = setup.as_ref() else { return };
            let SetupStage::Editing(editor) = &setup.stage else {
                return;
            };
            let here = Here {
                system: &RealFs,
                this: &self.this,
                username: &self.username,
                hostname: &setup.hostname,
            };
            let free = RealFs.free_space(&setup.root).ok();
            editor.choice(&here).map(|choice| (choice, free))
        };
        let Some((choice, free)) = opened else { return };
        self.choosing.set(true);
        self.choose_page
            .open(&self.username, choice, free, self.language.get());
        self.show_screen("choose");
    }

    /// Leaves "Choose what to back up…". With `keep`, the choice goes into the editor, and waits
    /// for Save there.
    fn close_choose(&self, keep: bool) {
        if !self.choosing.replace(false) {
            return;
        }
        if keep {
            let choice = self.choose_page.choice();
            let mut setup = self.setup.borrow_mut();
            if let Some(setup) = setup.as_mut()
                && let SetupStage::Editing(editor) = &mut setup.stage
            {
                let here = Here {
                    system: &RealFs,
                    this: &self.this,
                    username: &self.username,
                    hostname: &setup.hostname,
                };
                editor.set_choice(&here, &choice);
                setup.notice = None;
            }
        }
        self.choose_page.close();
        self.show_screen("setup");
        self.show_setup();
    }

    /// Leaves the editor, dropping any unsaved changes. The list is checked again, because the
    /// setup may have changed.
    fn close_setup(self: &Rc<Self>) {
        if self.choosing.replace(false) {
            self.choose_page.close();
        }
        if self.setup.take().is_none() {
            return;
        }
        self.show_screen("start");
        self.media_changed();
    }

    fn connect_format_signals(self: &Rc<Self>) {
        let page = &self.format_page;
        let weak = Rc::downgrade(self);
        page.drives.connect_row_selected(move |_, _| {
            if let Some(window) = weak.upgrade()
                && !window.format_updating.get()
            {
                window.format_drive_chosen();
            }
        });
        let weak = Rc::downgrade(self);
        page.label.connect_changed(move |_| {
            let Some(window) = weak.upgrade() else { return };
            if window.format_updating.get() {
                return;
            }
            if let Some(format) = window.format.borrow_mut().as_mut() {
                format.label_edited = true;
            }
            window.show_format();
        });
        // Choosing Ext4 turns exFAT off before it turns Ext4 on, so the page is shown again for
        // Ext4 too, or its note would wait for the next change (T-035).
        for button in [&page.exfat, &page.ext4, &page.understood] {
            let weak = Rc::downgrade(self);
            button.connect_toggled(move |_| {
                if let Some(window) = weak.upgrade()
                    && !window.format_updating.get()
                {
                    window.show_format();
                }
            });
        }
        let weak = Rc::downgrade(self);
        page.erase.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.start_format();
            }
        });
        let weak = Rc::downgrade(self);
        page.try_again.connect_clicked(move |_| {
            let Some(window) = weak.upgrade() else { return };
            if let Some(format) = window.format.borrow_mut().as_mut() {
                format.stage = FormatStage::Choosing;
            }
            window.format_page.understood.set_active(false);
            window.update_format_drives();
        });
        let weak = Rc::downgrade(self);
        page.back.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.close_format();
            }
        });
        let weak = Rc::downgrade(self);
        self.details.empty_prepare.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.open_format();
            }
        });
    }

    /// Opens the format view, with the selected medium's drive chosen if it may be formatted.
    fn open_format(self: &Rc<Self>) {
        if self.sticks.is_none() || self.format.borrow().is_some() {
            return;
        }
        let drive = self
            .selected_medium()
            .map(|medium| medium.drive.id)
            .filter(|id| self.formattable.borrow().contains(id));
        *self.notice.borrow_mut() = None;
        *self.format.borrow_mut() = Some(FormatState {
            stage: FormatStage::Choosing,
            choices: Vec::new(),
            drive: None,
            label_edited: false,
        });
        let page = &self.format_page;
        self.format_updating.set(true);
        page.exfat.set_active(true);
        page.understood.set_active(false);
        page.label.set_text("");
        self.format_updating.set(false);
        self.update_format_drives_selecting(drive.as_deref());
        self.show_screen("format");
    }

    /// Lists the drives that may be formatted again, keeping the chosen one if it is still
    /// there.
    fn update_format_drives(&self) {
        let chosen = self
            .format
            .borrow()
            .as_ref()
            .and_then(|format| format.drive.clone());
        self.update_format_drives_selecting(chosen.as_deref());
    }

    fn update_format_drives_selecting(&self, selected: Option<&str>) {
        let choosing = self
            .format
            .borrow()
            .as_ref()
            .is_some_and(|format| matches!(format.stage, FormatStage::Choosing));
        let Some(sticks) = &self.sticks else { return };
        if !choosing {
            return;
        }
        let babawus: HashSet<String> = self
            .media
            .borrow()
            .iter()
            .filter(|(_, checked)| {
                checked
                    .as_ref()
                    .is_some_and(|checked| checked.status.is_babawus())
            })
            .map(|(medium, _)| medium.volume.object_path.clone())
            .collect();
        let choices = drive_choices(&sticks.snapshot(), &babawus, self.language.get());
        self.format_updating.set(true);
        self.format_page.set_drives(&choices, selected);
        self.format_updating.set(false);
        if let Some(format) = self.format.borrow_mut().as_mut() {
            format.choices = choices;
        }
        self.format_drive_chosen();
    }

    /// The chosen drive may have changed. A different drive needs confirming again, and gets
    /// the suggested label unless the user has typed one.
    fn format_drive_chosen(&self) {
        let chosen = self.format_page.chosen_drive();
        let suggestion = {
            let mut format = self.format.borrow_mut();
            let Some(format) = format.as_mut() else {
                return;
            };
            if format.drive == chosen {
                None
            } else {
                format.drive = chosen.clone();
                let suggestion = match (&self.sticks, &chosen) {
                    (Some(sticks), Some(id)) if !format.label_edited => {
                        Some(suggested_label(&sticks.snapshot(), id))
                    }
                    _ => None,
                };
                Some(suggestion)
            }
        };
        if let Some(suggestion) = suggestion {
            self.format_updating.set(true);
            self.format_page.understood.set_active(false);
            if let Some(label) = suggestion {
                self.format_page.label.set_text(&label);
            }
            self.format_updating.set(false);
        }
        self.show_format();
    }

    fn formatting(&self) -> bool {
        self.format
            .borrow()
            .as_ref()
            .is_some_and(|format| matches!(format.stage, FormatStage::Working { .. }))
    }

    /// Formats the chosen drive, once the user has confirmed.
    fn start_format(self: &Rc<Self>) {
        let page = &self.format_page;
        let label = page.label.text().to_string();
        let file_system = page.format();
        let drive = {
            let mut format = self.format.borrow_mut();
            let Some(format) = format.as_mut() else {
                return;
            };
            let ready = matches!(format.stage, FormatStage::Choosing)
                && can_format(format.drive.is_some(), &label, page.understood.is_active());
            if !ready {
                return;
            }
            format.stage = FormatStage::Working {
                label: label.clone(),
            };
            format.drive.clone()
        };
        let Some(drive) = drive else { return };
        self.show_format();
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(window) = weak.upgrade() else { return };
            let Some(sticks) = &window.sticks else { return };
            let result = sticks.format(&drive, file_system, &label).await;
            let stage = match result {
                Ok(root) => FormatStage::Done {
                    label,
                    root,
                    format: file_system,
                },
                Err(error) => {
                    eprintln!("Babawus could not format {drive}: {error:?}");
                    let failure = Failure::of(&error);
                    FormatStage::Failed { label, failure }
                }
            };
            if let Some(format) = window.format.borrow_mut().as_mut() {
                format.stage = stage;
            }
            window.show_format();
        });
    }

    fn show_format(&self) {
        let language = self.language.get();
        let format = self.format.borrow();
        let Some(format) = format.as_ref() else {
            return;
        };
        let page = &self.format_page;
        match &format.stage {
            FormatStage::Choosing => {
                let label = page.label.text();
                let choice = format
                    .drive
                    .as_ref()
                    .and_then(|id| format.choices.iter().find(|choice| &choice.id == id));
                let warning = choice.map(|choice| warning(choice, language));
                let problem = label_problem(&label, language);
                let can = can_format(choice.is_some(), &label, page.understood.is_active());
                page.show_choice(problem.as_deref(), warning.as_deref(), can);
                page.show_ext4_note(language);
            }
            FormatStage::Working { label } => page.show_working(label, language),
            FormatStage::Done { label, .. } => {
                page.show_result(&result_screen(label, None, language));
            }
            FormatStage::Failed { label, failure } => {
                page.show_result(&result_screen(label, Some(failure), language));
            }
        }
        self.update_advanced_action();
    }

    /// Leaves the format view, unless formatting is going on. A newly formatted medium is
    /// selected on the start screen.
    fn close_format(self: &Rc<Self>) {
        if self.formatting() {
            return;
        }
        let Some(format) = self.format.take() else {
            return;
        };
        if let FormatStage::Done { root, .. } = &format.stage {
            let selected = self
                .media
                .borrow()
                .iter()
                .find(|(medium, _)| &medium.root == root)
                .map(|(medium, _)| medium.volume.object_path.clone());
            if selected.is_some() {
                *self.selected.borrow_mut() = selected;
            }
        }
        self.update_advanced_action();
        self.show_screen("start");
        self.media_changed();
    }

    /// Shows the screen called `name`, starting at the top (issue 118).
    fn show_screen(&self, name: &str) {
        if let Some(screen) = self.screens.child_by_name(name) {
            crate::scroll::to_top(&screen);
        }
        self.screens.set_visible_child_name(name);
    }

    fn close_info(&self) {
        *self.info.borrow_mut() = None;
        self.show_screen("start");
    }

    /// Ejects the drive with the udisks2 drive ID `drive_id`, whose medium is labelled `label`,
    /// and says how it went, on whichever screen is showing.
    fn eject_drive(self: &Rc<Self>, drive_id: String, label: String) {
        if self.ejecting.borrow().is_some() {
            return;
        }
        *self.ejecting.borrow_mut() = Some(drive_id.clone());
        *self.notice.borrow_mut() = Some(Notice {
            text: Text::Ejecting,
            label: label.clone(),
        });
        self.show_details();
        self.show_backup();
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(window) = weak.upgrade() else { return };
            let Some(sticks) = &window.sticks else { return };
            let result = sticks.eject(&drive_id).await;
            // The stick has left, so there is nothing more to do with it here.
            let gone = matches!(result, Ok(_) | Err(EjectError::Gone));
            let text = match result {
                Ok(Ejected::PoweredOff) => Text::Ejected,
                Ok(Ejected::UnmountedOnly { reason }) => {
                    if let Some(reason) = reason {
                        eprintln!("Babawus could not power off {label}: {reason}");
                    }
                    Text::EjectedUnmountedOnly
                }
                Err(EjectError::Busy) => Text::EjectBusy,
                Err(EjectError::Gone) => Text::EjectGone,
                Err(EjectError::Failed(message)) => {
                    eprintln!("Babawus could not eject {label}: {message}");
                    Text::EjectFailed
                }
            };
            *window.ejecting.borrow_mut() = None;
            *window.notice.borrow_mut() = None;
            window.show_details();
            window.show_backup();
            // How it went is a pop-up, on whichever screen is showing (issue 128).
            // After a successful eject from the verdict screen, the backup is done with too.
            let language = window.language.get();
            let message = text.fill(language, &[("label", &label)]);
            let weak = Rc::downgrade(&window);
            window.tell(&message, move || {
                if let Some(window) = weak.upgrade()
                    && gone
                    && window.screens.visible_child_name().as_deref() == Some("backup")
                {
                    window.back_to_start();
                }
            });
        });
    }

    /// Starts a backup to the selected medium, if it is Ready.
    fn start_backup(self: &Rc<Self>) {
        let Some(medium) = self.selected_medium() else {
            return;
        };
        let previous = self.media.borrow().iter().find_map(|(each, checked)| {
            match checked.as_ref().map(|checked| &checked.status) {
                Some(Status::Ready { last_backup }) if each == &medium => Some(*last_backup),
                _ => None,
            }
        });
        let ready = previous.is_some();
        let Some(format) = medium.volume.format() else {
            return;
        };
        let restoring = self.restoring.borrow().contains(&medium.volume.object_path);
        if !ready || restoring || self.backup.borrow().is_some() {
            return;
        }
        // Only shown in the log and the configuration, so a missing hostname stops nothing.
        let hostname = platform::hostname(&RealFs).unwrap_or_default();
        let backup = Backup {
            medium: Arc::new(RealFs),
            root: medium.root.clone(),
            format,
            system: Arc::new(RealFs),
            this: self.this.clone(),
            username: self.username.clone(),
            hostname,
            home: platform::home_folder(),
            now: Arc::new(SystemTime::now),
            state_file: Some(platform::last_backup_file()),
        };
        *self.notice.borrow_mut() = None;
        *self.backup.borrow_mut() = Some(BackupState {
            label: medium.volume.label.clone(),
            drive_id: medium.drive.id.clone(),
            previous: previous.flatten(),
            ended_at: None,
            running: Some(backup.start()),
            tracker: Tracker::default(),
            ended: None,
            close_when_done: false,
            question: gio::Cancellable::new(),
        });
        self.show_screen("backup");
        self.show_backup();
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(BACKUP_UPDATES, move || {
            let Some(window) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            window.take_messages()
        });
    }

    /// Takes the run's messages that have come since last time, and shows where it is now.
    /// Only the last of them matters for the screen.
    fn take_messages(&self) -> glib::ControlFlow {
        let mut ended = false;
        let mut close = false;
        if let Some(backup) = self.backup.borrow_mut().as_mut() {
            while let Some(running) = &backup.running {
                match running.messages.try_recv() {
                    Ok(Message::Progress(progress)) => backup.tracker.update(progress),
                    Ok(Message::Ended(result)) => {
                        backup.ended = Some(Some(result));
                        backup.running = None;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        backup.ended = Some(None);
                        backup.running = None;
                    }
                }
            }
            ended = backup.running.is_none();
            close = ended && backup.close_when_done;
            if ended {
                backup.question.cancel();
                backup.ended_at.get_or_insert_with(SystemTime::now);
            }
        }
        self.show_backup();
        if close {
            self.window.close();
        }
        if ended {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    }

    fn show_backup(&self) {
        let language = self.language.get();
        let backup = self.backup.borrow();
        let Some(backup) = backup.as_ref() else {
            return;
        };
        match &backup.ended {
            Some(ended) => {
                let context = Context {
                    label: &backup.label,
                    previous: backup.previous,
                    now: backup.ended_at.unwrap_or_else(SystemTime::now),
                    clock: local_clock,
                };
                let shown = verdict_screen(ended.as_ref(), &context, language);
                let notice = self
                    .notice
                    .borrow()
                    .as_ref()
                    .map(|notice| notice.text.fill(language, &[("label", &notice.label)]));
                let ejecting = self.ejecting.borrow().is_some();
                self.backup_page.show_verdict(
                    &backup.label,
                    &shown,
                    notice.as_deref(),
                    ejecting,
                    language,
                );
            }
            None => {
                let shown = backup.tracker.shown(language);
                self.backup_page
                    .show_progress(&backup.label, &shown, language);
            }
        }
    }

    /// Asks whether to stop the run, and stops it if so. `closing` closes the window once the
    /// run has ended.
    fn ask_to_stop(self: &Rc<Self>, closing: bool) {
        let already = self
            .backup
            .borrow()
            .as_ref()
            .is_none_or(|backup| backup.tracker.cancelling());
        if already {
            if closing && let Some(backup) = self.backup.borrow_mut().as_mut() {
                backup.close_when_done = true;
            }
            return;
        }
        let language = self.language.get();
        let dialog = AlertDialog::builder()
            .message(Text::StopQuestion.get(language))
            .detail(Text::StopDetail.get(language))
            .buttons([
                Text::KeepGoing.get(language),
                Text::StopBackup.get(language),
            ])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        let question = self
            .backup
            .borrow()
            .as_ref()
            .map(|backup| backup.question.clone());
        let weak = Rc::downgrade(self);
        dialog.choose(Some(&self.window), question.as_ref(), move |answer| {
            let Some(window) = weak.upgrade() else { return };
            if answer != Ok(1) {
                return;
            }
            if let Some(backup) = window.backup.borrow_mut().as_mut()
                && let Some(running) = &backup.running
            {
                running.cancel();
                backup.tracker.cancel();
                backup.close_when_done |= closing;
            }
            window.show_backup();
        });
    }

    /// Leaves the backup screen once the run has ended. The list is checked again, because the
    /// run changed the last backup.
    fn back_to_start(self: &Rc<Self>) {
        let ended = self
            .backup
            .borrow()
            .as_ref()
            .is_some_and(|backup| backup.running.is_none());
        if !ended {
            return;
        }
        *self.backup.borrow_mut() = None;
        self.show_screen("start");
        self.media_changed();
    }

    fn selected_medium(&self) -> Option<Medium> {
        let selected = self.selected.borrow();
        let selected = selected.as_ref()?;
        self.media
            .borrow()
            .iter()
            .find(|(medium, _)| &medium.volume.object_path == selected)
            .map(|(medium, _)| medium.clone())
    }
}

/// Makes the label under `widget` that shows `markup` as plain text show it as markup.
fn use_markup_for(widget: &gtk4::Widget, markup: &str) {
    if let Some(label) = widget.downcast_ref::<Label>()
        && label.label() == markup
    {
        label.set_use_markup(true);
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        use_markup_for(&current, markup);
        child = current.next_sibling();
    }
}

/// Makes every label inside `widget` unselectable.
fn unselectable(widget: &gtk4::Widget) {
    if let Some(label) = widget.downcast_ref::<Label>() {
        label.set_selectable(false);
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        unselectable(&current);
        child = current.next_sibling();
    }
}

/// What leaving the setup without saving does next.
enum AfterLeaving {
    /// Only goes back to the start page.
    Nothing,
    /// Closes the window too.
    CloseWindow,
    /// Turns Advanced off too (issue 165).
    AdvancedOff,
}

/// A row of action buttons that wraps like words onto centred lines when it runs out of room.
fn action_row() -> Box {
    let row = Box::new(Orientation::Horizontal, 0);
    row.set_layout_manager(Some(crate::wrap::WrapLayout::new(8)));
    row
}

/// A new row of the media list, named by the udisks2 object path of the medium's file system.
fn new_row(object_path: &str) -> RowWidgets {
    let stick = icons::image(&icons::stick(false), STICK_SIZE);
    let label = Label::builder().xalign(0.0).hexpand(true).build();
    label.add_css_class("babawus-row-label");
    let spinner = Spinner::builder().spinning(true).build();
    let eject_symbol = icons::image(&icons::eject(), EJECT_SIZE);
    let eject = Button::builder()
        .child(&eject_symbol)
        .valign(Align::Center)
        .action_name("win.eject-stick")
        .action_target(&object_path.to_variant())
        .build();
    eject.add_css_class("circular");
    eject.add_css_class("babawus-eject");
    let stick_line = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(SPACING)
        .build();
    stick_line.append(&stick);
    stick_line.append(&label);
    stick_line.append(&spinner);
    stick_line.append(&eject);
    let start_backup = Button::builder()
        .action_name("win.start-backup")
        .visible(false)
        .build();
    start_backup.add_css_class("suggested-action");
    start_backup.add_css_class("babawus-start-backup");
    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(4)
        .margin_end(4)
        .build();
    content.append(&stick_line);
    content.append(&start_backup);
    let row = ListBoxRow::builder().child(&content).build();
    row.add_css_class("babawus-stick-row");
    row.set_widget_name(object_path);
    RowWidgets {
        row,
        stick,
        label,
        spinner,
        eject,
        start_backup,
    }
}
