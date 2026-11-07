//! The start screen: the media list on the left, the details of the selected medium on the
//! right, with the actions for it below them. See "Start Babawus and see status" in
//! Requirements.md. What to show comes from [`babawus_core::start_screen`]; this module only
//! puts it on the screen.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::{Rc, Weak};
use std::time::SystemTime;

use babawus_core::fs::RealFs;
use babawus_core::identity::this_computer_id;
use babawus_core::lock::ThisProcess;
use babawus_core::media::{Medium, Viewer};
use babawus_core::settings::{Language, SETTINGS_FILE, Settings};
use babawus_core::start_screen::{self, Action, Checked, Shown};
use babawus_core::text::Text;
use babawus_core::udisks::{EjectError, Ejected, Udisks};
use gtk4::prelude::*;
use gtk4::{
    AboutDialog, Align, Application, ApplicationWindow, Box, Button, HeaderBar, Image, Label,
    ListBox, ListBoxRow, MenuButton, Orientation, Overlay, PolicyType, ScrolledWindow,
    SelectionMode, Separator, Spinner, Stack, gio, glib,
};

use crate::icons;

const MARGIN: i32 = 24;
const SPACING: i32 = 12;
const LIST_WIDTH: i32 = 260;
const LOGO_SIZE: i32 = 64;
/// Leaves room for the logo in the upper right corner of the details.
const LOGO_ROOM: i32 = LOGO_SIZE + MARGIN;

const CSS: &str = "
.babawus-row-label { font-size: 15pt; font-weight: bold; }
.babawus-title { font-size: 20pt; font-weight: bold; }
.babawus-status { font-size: 13pt; }
.babawus-last-backup { font-size: 13pt; }
.babawus-plug-in { font-size: 16pt; }
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

/// One row of the media list.
struct RowWidgets {
    row: ListBoxRow,
    stick: Image,
    label: Label,
    mark: Image,
    spinner: Spinner,
}

/// The parts of the details that change.
struct DetailsWidgets {
    pages: Stack,
    plug_in: Label,
    title: Label,
    mark: Image,
    spinner: Spinner,
    status: Label,
    about: Label,
    last_backup: Label,
    skipped: Image,
    explanation: Label,
    buttons: HashMap<Action, Button>,
    poweruser: Box,
    notice: Label,
    logo: Image,
}

struct Window {
    window: ApplicationWindow,
    title: Label,
    advanced_marker: Label,
    menu: MenuButton,
    list: ListBox,
    rows: RefCell<Vec<RowWidgets>>,
    details: DetailsWidgets,

    udisks: Option<Udisks>,
    this: ThisProcess,
    username: String,
    settings_path: std::path::PathBuf,

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
}

/// Opens the start screen.
pub fn show(app: &Application) {
    add_css();
    let settings_path = glib::user_config_dir().join(SETTINGS_FILE);
    let language = current_language(&settings_path);
    let udisks = Udisks::connect()
        .inspect_err(|error| eprintln!("Babawus could not connect to udisks2: {error}"))
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

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(820)
        .default_height(500)
        .titlebar(&header)
        .child(&content)
        .build();

    let this_window = Rc::new(Window {
        window,
        title,
        advanced_marker,
        menu,
        list,
        rows: RefCell::new(Vec::new()),
        details,
        udisks,
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
    });
    this_window.connect_signals();
    this_window.add_actions();
    this_window.set_texts();
    this_window.media_changed();
    if let Some(udisks) = &this_window.udisks {
        let weak = Rc::downgrade(&this_window);
        udisks.connect_changed(move || {
            if let Some(window) = weak.upgrade() {
                window.media_changed();
            }
        });
    }
    // The window keeps its state alive until it is destroyed. The handlers only hold weak
    // references, so this is the one strong one.
    let state = RefCell::new(Some(this_window.clone()));
    this_window.window.connect_destroy(move |_| {
        state.take();
    });
    this_window.window.present();
}

/// The language to show at the start: the user's choice, otherwise the system's.
pub fn current_language(settings_path: &std::path::Path) -> Language {
    let settings = Settings::load(&RealFs, settings_path);
    let system_names = glib::language_names();
    settings.language(system_names.iter().map(|name| name.as_str()))
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

/// The details pane, and the widgets in it that change.
fn details_widgets() -> (DetailsWidgets, Overlay) {
    let title = Label::builder().xalign(0.0).build();
    title.add_css_class("babawus-title");
    let mark = icons::image(&icons::mark(start_screen::Mark::Ready), 24);
    let spinner = Spinner::builder().spinning(true).build();
    let status = Label::new(None);
    status.add_css_class("babawus-status");
    let status_row = Box::new(Orientation::Horizontal, 8);
    status_row.append(&mark);
    status_row.append(&spinner);
    status_row.append(&status);
    let about = wrapping_label();
    about.add_css_class("dim-label");

    let last_backup = Label::new(None);
    last_backup.add_css_class("babawus-last-backup");
    let skipped = icons::image(&icons::mark(start_screen::Mark::Attention), 18);
    let last_backup_row = Box::new(Orientation::Horizontal, 8);
    last_backup_row.append(&last_backup);
    last_backup_row.append(&skipped);
    let explanation = wrapping_label();

    let actions = Box::new(Orientation::Horizontal, 8);
    let poweruser = Box::new(Orientation::Horizontal, 8);
    let mut buttons = HashMap::new();
    for action in Action::ALL {
        let button = Button::new();
        if action == Action::StartBackup {
            button.add_css_class("suggested-action");
        }
        if action.for_poweruser() {
            poweruser.append(&button);
        } else {
            actions.append(&button);
        }
        buttons.insert(action, button);
    }

    let medium_page = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(SPACING)
        .build();
    let heading = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_end(LOGO_ROOM)
        .build();
    heading.append(&title);
    heading.append(&status_row);
    heading.append(&about);
    medium_page.append(&heading);
    medium_page.append(&last_backup_row);
    medium_page.append(&explanation);
    let spacer = Box::builder().vexpand(true).build();
    medium_page.append(&spacer);
    medium_page.append(&Separator::new(Orientation::Horizontal));
    medium_page.append(&actions);
    medium_page.append(&poweruser);

    let plug_in = Label::builder()
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .valign(Align::Center)
        .halign(Align::Center)
        .build();
    plug_in.add_css_class("babawus-plug-in");

    let pages = Stack::builder().vexpand(true).build();
    pages.add_named(&medium_page, Some("medium"));
    pages.add_named(&plug_in, Some("empty"));

    let notice = wrapping_label();
    notice.set_visible(false);
    let pane = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(SPACING)
        .margin_top(MARGIN)
        .margin_bottom(MARGIN)
        .margin_start(MARGIN)
        .margin_end(MARGIN)
        .hexpand(true)
        .build();
    pane.append(&pages);
    pane.append(&notice);

    let logo = icons::image(&icons::logo(), LOGO_SIZE);
    logo.set_halign(Align::End);
    logo.set_valign(Align::Start);
    logo.set_margin_top(MARGIN / 2);
    logo.set_margin_end(MARGIN / 2);
    let overlay = Overlay::builder().child(&pane).build();
    overlay.add_overlay(&logo);

    let widgets = DetailsWidgets {
        pages,
        plug_in,
        title,
        mark,
        spinner,
        status,
        about,
        last_backup,
        skipped,
        explanation,
        buttons,
        poweruser,
        notice,
        logo,
    };
    (widgets, overlay)
}

impl Window {
    fn connect_signals(self: &Rc<Self>) {
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
            &self.language.get().name().to_variant(),
        );
        let weak = Rc::downgrade(self);
        language.connect_activate(move |action, parameter| {
            let Some(window) = weak.upgrade() else { return };
            let Some(name) = parameter.and_then(|parameter| parameter.str()) else {
                return;
            };
            let Some(chosen) = Language::ALL.into_iter().find(|each| each.name() == name) else {
                return;
            };
            action.set_state(&name.to_variant());
            window.choose_language(chosen);
        });
        self.window.add_action(&language);

        let advanced = gio::SimpleAction::new_stateful("advanced", None, &false.to_variant());
        let weak = Rc::downgrade(self);
        advanced.connect_activate(move |action, _| {
            let Some(window) = weak.upgrade() else { return };
            let on = !window.advanced.get();
            action.set_state(&on.to_variant());
            window.advanced.set(on);
            window.set_texts();
        });
        self.window.add_action(&advanced);

        let about = gio::SimpleAction::new("about", None);
        let weak = Rc::downgrade(self);
        about.connect_activate(move |_, _| {
            if let Some(window) = weak.upgrade() {
                window.show_about();
            }
        });
        self.window.add_action(&about);
    }

    fn choose_language(&self, language: Language) {
        self.language.set(language);
        let settings = Settings {
            language: Some(language),
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
        let languages = gio::Menu::new();
        for each in Language::ALL {
            let item = gio::MenuItem::new(Some(each.name()), None);
            item.set_action_and_target_value(Some("win.language"), Some(&each.name().to_variant()));
            languages.append_item(&item);
        }
        let menu = gio::Menu::new();
        menu.append_submenu(Some(Text::MenuLanguage.get(language)), &languages);
        menu.append(Some(Text::MenuAdvanced.get(language)), Some("win.advanced"));
        menu.append(Some(Text::MenuAbout.get(language)), Some("win.about"));
        menu
    }

    fn show_about(&self) {
        let language = self.language.get();
        AboutDialog::builder()
            .transient_for(&self.window)
            .modal(true)
            .program_name("Babawus")
            .version(babawus_core::VERSION)
            .comments(Text::AboutComments.get(language))
            .logo(&icons::logo())
            .website("https://github.com/ziggymax/babawus")
            .build()
            .present();
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
        self.update_rows();
        self.show_details();
    }

    /// udisks2 reported a change, or Babawus has just started: the list is read again, and each
    /// medium is checked off the main thread.
    fn media_changed(self: &Rc<Self>) {
        let Some(udisks) = &self.udisks else {
            return;
        };
        let snapshot = udisks.snapshot();
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
        let to_check: Vec<Medium> = media.iter().map(|(medium, _)| medium.clone()).collect();
        *self.media.borrow_mut() = media;
        self.update_rows();
        self.show_details();

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
            row.spinner.set_visible(shown.mark.is_none());
            row.mark.set_visible(shown.mark.is_some());
            if let Some(mark) = shown.mark {
                row.mark.set_paintable(Some(&icons::mark(mark)));
            }
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
        match &*self.notice.borrow() {
            Some(notice) => {
                let text = notice.text.fill(language, &[("label", &notice.label)]);
                widgets.notice.set_label(&text);
                widgets.notice.set_visible(true);
            }
            None => widgets.notice.set_visible(false),
        }
        let media = self.media.borrow();
        let selected = self.selected.borrow();
        let Some((medium, checked)) = selected.as_ref().and_then(|selected| {
            media
                .iter()
                .find(|(medium, _)| &medium.volume.object_path == selected)
        }) else {
            let text = if self.udisks.is_none() {
                Text::NoUdisks
            } else if media.is_empty() {
                Text::PlugIn
            } else {
                Text::ChooseMedium
            };
            widgets.plug_in.set_label(text.get(language));
            widgets.pages.set_visible_child_name("empty");
            return;
        };
        widgets.pages.set_visible_child_name("medium");
        let shown = Shown {
            medium,
            checked: checked.as_ref(),
            formattable: self.formattable.borrow().contains(&medium.drive.id),
        };
        let details =
            start_screen::details(&shown, SystemTime::now(), language, self.advanced.get());
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
        let ejecting = self.ejecting.borrow().is_some();
        for (action, button) in &widgets.buttons {
            match details.buttons.iter().find(|shown| shown.action == *action) {
                Some(shown) => {
                    button.set_visible(true);
                    let busy = *action == Action::Eject && ejecting;
                    button.set_sensitive(shown.enabled && !busy);
                    button.set_tooltip_text(Some(&shown.tooltip));
                }
                None => button.set_visible(false),
            }
        }
        widgets.poweruser.set_visible(self.advanced.get());
    }

    fn act(self: &Rc<Self>, action: Action) {
        // The buttons for the other actions are greyed out until their screens are built.
        if action == Action::Eject {
            self.eject();
        }
    }

    fn eject(self: &Rc<Self>) {
        let Some(medium) = self.selected_medium() else {
            return;
        };
        if self.ejecting.borrow().is_some() {
            return;
        }
        let label = medium.volume.label.clone();
        *self.ejecting.borrow_mut() = Some(medium.drive.id.clone());
        *self.notice.borrow_mut() = Some(Notice {
            text: Text::Ejecting,
            label: label.clone(),
        });
        self.show_details();
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Some(window) = weak.upgrade() else { return };
            let Some(udisks) = &window.udisks else { return };
            let result = udisks.eject(&medium.drive.id).await;
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
            *window.notice.borrow_mut() = Some(Notice { text, label });
            window.show_details();
        });
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

/// A new row of the media list, named by the udisks2 object path of the medium's file system.
fn new_row(object_path: &str) -> RowWidgets {
    let stick = icons::image(&icons::stick(false), 32);
    let label = Label::builder().xalign(0.0).hexpand(true).build();
    label.add_css_class("babawus-row-label");
    let mark = icons::image(&icons::mark(start_screen::Mark::Ready), 24);
    let spinner = Spinner::builder().spinning(true).build();
    let content = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(SPACING)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(4)
        .margin_end(4)
        .build();
    content.append(&stick);
    content.append(&label);
    content.append(&mark);
    content.append(&spinner);
    let row = ListBoxRow::builder().child(&content).build();
    row.set_widget_name(object_path);
    RowWidgets {
        row,
        stick,
        label,
        mark,
        spinner,
    }
}
