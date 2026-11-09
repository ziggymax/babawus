//! The configuration editor's overview: the number of older versions, this computer and
//! account, and every entry on the medium, with Save and Back. See "Configure USB media" in
//! Requirements.md and Issues.md items 83-85. What to show comes from
//! [`babawus_core::setup_screen`]; this module only puts it on the screen, and passes what the
//! poweruser presses on to the window.

use std::rc::Rc;

use babawus_core::settings::Language;
use babawus_core::setup_screen::{HereAction, OverviewShown};
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

/// Something the poweruser pressed in a part of the page that is built anew each time.
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
    older_versions_label: Label,
    pub older_versions: SpinButton,
    older_versions_detail: Label,
    here_heading: Label,
    here_text: Label,
    here_actions: Box,
    pub choose: Button,
    entries_heading: Label,
    entries: ListBox,
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
        let message = label(Some("babawus-status"));
        let start_new = Button::builder().halign(Align::Start).build();

        let banner = label(Some("babawus-next-step"));
        banner.set_hexpand(true);
        let banner_mark = icons::image(&icons::mark(Mark::Attention), 24);
        banner_mark.set_valign(Align::Start);
        let banner_row = Box::new(Orientation::Horizontal, SPACING);
        banner_row.append(&banner_mark);
        banner_row.append(&banner);

        let older_versions_label = label(Some("babawus-next-step"));
        older_versions_label.set_hexpand(false);
        let older_versions = SpinButton::new(
            Some(&Adjustment::new(10.0, 1.0, 99.0, 1.0, 10.0, 0.0)),
            1.0,
            0,
        );
        older_versions.set_numeric(true);
        older_versions.set_valign(Align::Center);
        let older_row = Box::new(Orientation::Horizontal, SPACING);
        older_row.append(&older_versions_label);
        older_row.append(&older_versions);
        let older_versions_detail = label(Some("dim-label"));

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
        editing.append(&older_row);
        editing.append(&older_versions_detail);
        editing.append(&Separator::new(Orientation::Horizontal));
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
            older_versions_label,
            older_versions,
            older_versions_detail,
            here_heading,
            here_text,
            here_actions,
            choose,
            entries_heading,
            entries,
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
        self.older_versions_label
            .set_label(Text::SetupOlderVersions.get(language));
        self.older_versions_detail
            .set_label(Text::SetupOlderVersionsDetail.get(language));
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
        if self.older_versions.value_as_int() as u32 != shown.older_versions {
            self.older_versions
                .set_value(f64::from(shown.older_versions));
        }

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

        self.entries.remove_all();
        for computer in &shown.computers {
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
                text.set_margin_start(SPACING * 2);
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
                rows.append(&row);
            }
            self.entries.append(&rows);
        }
        self.entries.set_visible(!shown.computers.is_empty());
        set_optional(&self.empty, shown.empty.as_deref());

        self.save.set_visible(true);
        self.save.set_sensitive(shown.can_save && !busy);
        self.back.set_sensitive(!busy);
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
