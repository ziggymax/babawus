//! The configuration editor's overview: this computer and account, and every entry on the
//! medium with each backup profile's numbers, with Save and Back. See "Configure USB media" in
//! Requirements.md and issues 83-85. What to show comes from
//! [`babawus_core::setup_screen`]; this module only puts it on the screen, and passes what the
//! user presses on to the window.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use babawus_core::config::{OLDER_VERSIONS, RUN_CHECK_DAYS};
use babawus_core::language::Language;
use babawus_core::setup_screen::{ComputerShown, HereAction, OverviewShown};
use babawus_core::start_screen::Mark;
use babawus_core::text::Text;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Align, Box, Button, Label, ListBox, Orientation, PolicyType, ScrolledWindow,
    SelectionMode, Separator, SpinButton, Spinner,
};

use crate::icons;

const MARGIN: i32 = 32;
const SPACING: i32 = 12;

/// Something the user pressed in a part of the page that is built anew each time.
#[derive(Clone, Debug)]
pub enum SetupEvent {
    Here(HereAction),
    /// Remove the computer with this ID, whose folder is named.
    RemoveComputer {
        id: String,
        folder: String,
    },
    RemoveAccount {
        id: String,
        folder: String,
        username: String,
    },
    /// A backup profile's numbers were changed (issue 136).
    Numbers {
        id: String,
        username: String,
        older_versions: u32,
        run_check_days: u32,
    },
}

/// A backup profile's two number fields.
struct NumberFields {
    id: String,
    username: String,
    older_versions: SpinButton,
    run_check_days: SpinButton,
}

/// What the page shows instead of the overview.
pub enum Instead<'a> {
    Loading,
    /// A message, and whether "Start a new setup…" is offered.
    Message(&'a str, bool),
}

pub struct SetupPage {
    pub root: Box,
    title: Label,
    heading: Label,
    loading: Box,
    loading_text: Label,
    message: Label,
    pub start_new: Button,

    editing: Box,
    banner: Label,
    banner_row: Box,
    here_heading: Label,
    here_text: Label,
    here_actions: Box,
    pub choose: Button,
    entries_heading: Label,
    entries: ListBox,
    /// What the list was last built from. It is only built again when that changes, so that a
    /// number field isn't replaced while it is being used.
    entries_built: RefCell<String>,
    numbers: RefCell<Vec<NumberFields>>,
    /// Set while the page sets the number fields itself, so that their signals are ignored.
    updating: Rc<Cell<bool>>,
    empty: Label,

    notice: Label,
    pub save: Button,
    pub back: Button,
    on_event: Rc<dyn Fn(SetupEvent)>,
}

fn label(class: Option<&str>) -> Label {
    let label = Label::builder().xalign(0.0).wrap(true).build();
    if let Some(class) = class {
        label.add_css_class(class);
    }
    label
}

fn column() -> Box {
    Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(SPACING)
        .build()
}

impl SetupPage {
    /// A new page, which passes what is pressed in its lists to `on_event`.
    pub fn new(on_event: impl Fn(SetupEvent) + 'static) -> Self {
        let title = label(Some("babawus-title"));
        let heading = label(Some("babawus-status"));
        let spinner = Spinner::builder().spinning(true).build();
        let loading_text = label(None);
        let loading = Box::new(Orientation::Horizontal, 8);
        loading.append(&spinner);
        loading.append(&loading_text);
        // In the normal style, like the rest of the page's text (`ui-review/UiReview.md` C06).
        let message = label(None);
        let start_new = Button::builder().halign(Align::Start).build();

        let banner = label(Some("babawus-next-step"));
        banner.set_hexpand(true);
        let banner_mark = icons::image(&icons::mark(Mark::Attention), 24);
        banner_mark.set_valign(Align::Start);
        let banner_row = Box::new(Orientation::Horizontal, SPACING);
        banner_row.append(&banner_mark);
        banner_row.append(&banner);

        let here_heading = label(Some("babawus-next-step"));
        let here_text = label(None);
        let here_actions = Box::new(Orientation::Horizontal, 8);
        let choose = Button::builder().halign(Align::Start).build();

        let entries_heading = label(Some("babawus-next-step"));
        let entries = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .build();
        entries.add_css_class("boxed-list");
        entries.set_show_separators(true);
        let empty = label(Some("dim-label"));

        let editing = column();
        editing.append(&banner_row);
        editing.append(&here_heading);
        editing.append(&here_text);
        editing.append(&here_actions);
        editing.append(&choose);
        editing.append(&Separator::new(Orientation::Horizontal));
        editing.append(&entries_heading);
        editing.append(&entries);
        editing.append(&empty);

        let page = column();
        page.set_margin_top(MARGIN);
        page.set_margin_bottom(SPACING);
        page.set_margin_start(MARGIN);
        page.set_margin_end(MARGIN);
        page.append(&title);
        page.append(&heading);
        page.append(&loading);
        page.append(&message);
        page.append(&start_new);
        page.append(&editing);
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .child(&page)
            .build();

        let notice = label(None);
        let save = Button::new();
        save.add_css_class("suggested-action");
        let back = Button::new();
        let buttons = Box::new(Orientation::Horizontal, 8);
        buttons.append(&save);
        buttons.append(&back);
        // The buttons stay in view while the rest scrolls.
        let bottom = column();
        bottom.set_margin_bottom(MARGIN);
        bottom.set_margin_start(MARGIN);
        bottom.set_margin_end(MARGIN);
        bottom.append(&Separator::new(Orientation::Horizontal));
        bottom.append(&notice);
        bottom.append(&buttons);
        let root = Box::new(Orientation::Vertical, 0);
        root.append(&scroll);
        root.append(&bottom);

        Self {
            root,
            title,
            heading,
            loading,
            loading_text,
            message,
            start_new,
            editing,
            banner,
            banner_row,
            here_heading,
            here_text,
            here_actions,
            choose,
            entries_heading,
            entries,
            entries_built: RefCell::default(),
            numbers: RefCell::default(),
            updating: Rc::default(),
            empty,
            notice,
            save,
            back,
            on_event: Rc::new(on_event),
        }
    }

    fn set_texts(&self, medium: &str, notice: Option<&str>, language: Language) {
        self.title.set_label(medium);
        self.heading.set_label(Text::SetupHeading.get(language));
        self.loading_text
            .set_label(Text::SetupLoading.get(language));
        self.start_new.set_label(Text::SetupStartNew.get(language));
        self.here_heading.set_label(Text::SetupHere.get(language));
        self.choose.set_label(Text::SetupChoose.get(language));
        self.choose
            .set_tooltip_text(Some(Text::SetupChooseTooltip.get(language)));
        self.entries_heading
            .set_label(Text::SetupEntries.get(language));
        self.save.set_label(Text::SetupSave.get(language));
        self.save
            .set_tooltip_text(Some(Text::SetupSaveTooltip.get(language)));
        self.back.set_label(Text::Back.get(language));
        self.back
            .set_tooltip_text(Some(Text::BackTooltip.get(language)));
        set_optional(&self.notice, notice);
    }

    /// Shows something other than the overview: a spinner, or a message.
    pub fn show_instead(
        &self,
        medium: &str,
        instead: Instead,
        notice: Option<&str>,
        language: Language,
    ) {
        self.set_texts(medium, notice, language);
        self.editing.set_visible(false);
        self.save.set_visible(false);
        self.loading
            .set_visible(matches!(instead, Instead::Loading));
        match instead {
            Instead::Loading => {
                set_optional(&self.message, None);
                self.start_new.set_visible(false);
            }
            Instead::Message(text, start_new) => {
                set_optional(&self.message, Some(text));
                self.start_new.set_visible(start_new);
            }
        }
    }

    /// Shows the overview. `busy` greys out everything that changes the stick while it is being
    /// written.
    pub fn show_overview(
        &self,
        medium: &str,
        shown: &OverviewShown,
        notice: Option<&str>,
        busy: bool,
        language: Language,
    ) {
        self.set_texts(medium, notice, language);
        self.loading.set_visible(busy);
        self.loading_text.set_label(Text::SetupSaving.get(language));
        set_optional(&self.message, None);
        self.start_new.set_visible(false);
        self.editing.set_visible(true);
        self.editing.set_sensitive(!busy);
        self.banner
            .set_label(shown.banner.as_deref().unwrap_or_default());
        self.banner_row.set_visible(shown.banner.is_some());

        self.here_text.set_label(&shown.here.text);
        while let Some(child) = self.here_actions.first_child() {
            self.here_actions.remove(&child);
        }
        for (index, (action, text)) in shown.here.actions.iter().enumerate() {
            let button = Button::with_label(text);
            if index == 0 {
                button.add_css_class("suggested-action");
            }
            let on_event = self.on_event.clone();
            let action = action.clone();
            button.connect_clicked(move |_| on_event(SetupEvent::Here(action.clone())));
            self.here_actions.append(&button);
        }
        self.here_actions
            .set_visible(!shown.here.actions.is_empty());
        self.choose.set_visible(shown.here.can_choose);

        let built = format!("{language:?}\n{}", without_numbers(&shown.computers));
        if *self.entries_built.borrow() != built {
            self.build_entries(&shown.computers, language);
            *self.entries_built.borrow_mut() = built;
        }
        self.updating.set(true);
        for fields in self.numbers.borrow().iter() {
            let account = shown
                .computers
                .iter()
                .filter(|computer| computer.id == fields.id)
                .flat_map(|computer| &computer.accounts)
                .find(|account| account.username == fields.username);
            if let Some(account) = account {
                set_number(&fields.older_versions, account.older_versions);
                set_number(&fields.run_check_days, account.run_check_days);
            }
        }
        self.updating.set(false);
        self.entries.set_visible(!shown.computers.is_empty());
        set_optional(&self.empty, shown.empty.as_deref());

        self.save.set_visible(true);
        self.save.set_sensitive(shown.can_save && !busy);
        self.back.set_sensitive(!busy);
    }

    /// Builds the list of computers and their backup profiles anew.
    fn build_entries(&self, computers: &[ComputerShown], language: Language) {
        self.entries.remove_all();
        let mut numbers = Vec::new();
        for computer in computers {
            let folder = computer.folder.clone();
            let title = label(Some("babawus-next-step"));
            title.set_label(&computer.title);
            title.set_hexpand(true);
            let remove = self.remove_button(
                SetupEvent::RemoveComputer {
                    id: computer.id.clone(),
                    folder: folder.clone(),
                },
                Text::SetupRemoveComputerTooltip,
                language,
            );
            let top = Box::new(Orientation::Horizontal, SPACING);
            top.append(&title);
            top.append(&remove);
            let rows = Box::new(Orientation::Vertical, 4);
            rows.set_margin_top(8);
            rows.set_margin_bottom(8);
            rows.set_margin_start(8);
            rows.set_margin_end(8);
            rows.append(&top);
            for account in &computer.accounts {
                let text = label(None);
                text.set_label(&account.text);
                text.set_hexpand(true);
                let remove = self.remove_button(
                    SetupEvent::RemoveAccount {
                        id: computer.id.clone(),
                        folder: folder.clone(),
                        username: account.username.clone(),
                    },
                    Text::SetupRemoveAccountTooltip,
                    language,
                );
                let row = Box::new(Orientation::Horizontal, SPACING);
                row.append(&text);
                row.append(&remove);
                let fields = NumberFields {
                    id: computer.id.clone(),
                    username: account.username.clone(),
                    older_versions: number_field(&OLDER_VERSIONS),
                    run_check_days: number_field(&RUN_CHECK_DAYS),
                };
                fields
                    .older_versions
                    .set_tooltip_text(Some(Text::SetupOlderVersionsDetail.get(language)));
                fields
                    .run_check_days
                    .set_tooltip_text(Some(Text::SetupRunCheckDaysDetail.get(language)));
                let numbers_row = Box::new(Orientation::Horizontal, 8);
                // On one line each, beside their fields.
                let older_label = label(Some("dim-label"));
                older_label.set_wrap(false);
                older_label.set_label(Text::SetupOlderVersions.get(language));
                let days_label = label(Some("dim-label"));
                days_label.set_wrap(false);
                days_label.set_label(Text::SetupRunCheckDays.get(language));
                days_label.set_margin_start(SPACING * 2);
                numbers_row.append(&older_label);
                numbers_row.append(&fields.older_versions);
                numbers_row.append(&days_label);
                numbers_row.append(&fields.run_check_days);
                for spin in [&fields.older_versions, &fields.run_check_days] {
                    let on_event = self.on_event.clone();
                    let updating = self.updating.clone();
                    let (id, username) = (fields.id.clone(), fields.username.clone());
                    let (older, days) =
                        (fields.older_versions.clone(), fields.run_check_days.clone());
                    spin.connect_value_changed(move |_| {
                        if updating.get() {
                            return;
                        }
                        on_event(SetupEvent::Numbers {
                            id: id.clone(),
                            username: username.clone(),
                            older_versions: older.value_as_int() as u32,
                            run_check_days: days.value_as_int() as u32,
                        });
                    });
                }
                let profile = Box::new(Orientation::Vertical, 4);
                profile.set_margin_start(SPACING * 2);
                profile.set_margin_top(4);
                profile.append(&row);
                profile.append(&numbers_row);
                rows.append(&profile);
                numbers.push(fields);
            }
            self.entries.append(&rows);
        }
        *self.numbers.borrow_mut() = numbers;
    }

    fn remove_button(&self, event: SetupEvent, tooltip: Text, language: Language) -> Button {
        let button = Button::with_label(Text::SetupRemove.get(language));
        button.set_valign(Align::Center);
        button.set_tooltip_text(Some(tooltip.get(language)));
        let on_event = self.on_event.clone();
        button.connect_clicked(move |_| on_event(event.clone()));
        button
    }
}

fn set_optional(label: &Label, text: Option<&str>) {
    label.set_label(text.unwrap_or_default());
    label.set_visible(text.is_some());
}

/// The computers as shown, without the numbers, which change without building the list again.
fn without_numbers(computers: &[ComputerShown]) -> String {
    let mut text = String::new();
    for computer in computers {
        text.push_str(&format!("{}\t{}\n", computer.id, computer.title));
        for account in &computer.accounts {
            text.push_str(&format!("\t{}\t{}\n", account.username, account.text));
        }
    }
    text
}

/// A field for a whole number in `range`.
fn number_field(range: &std::ops::RangeInclusive<u32>) -> SpinButton {
    let adjustment = Adjustment::new(
        f64::from(*range.start()),
        f64::from(*range.start()),
        f64::from(*range.end()),
        1.0,
        10.0,
        0.0,
    );
    let spin = SpinButton::new(Some(&adjustment), 1.0, 0);
    spin.set_numeric(true);
    spin.set_valign(Align::Center);
    spin
}

fn set_number(spin: &SpinButton, value: u32) {
    if spin.value_as_int() as u32 != value {
        spin.set_value(f64::from(value));
    }
}
