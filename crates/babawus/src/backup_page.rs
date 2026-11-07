//! The backup screen: what a run is doing, with Cancel, and how it ended. See "Provide feedback
//! during a backup operation" in Requirements.md. What to show comes from
//! [`babawus_core::backup_screen`]; this module only puts it on the screen.

use babawus_core::backup_screen::Shown;
use babawus_core::settings::Language;
use babawus_core::text::Text;
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, Label, Orientation, ProgressBar, Separator, Spinner};

const MARGIN: i32 = 32;
const SPACING: i32 = 12;

pub struct BackupPage {
    pub root: Box,
    title: Label,
    spinner: Spinner,
    status: Label,
    overview: Label,
    files_left: Label,
    bytes_left: Label,
    bar: ProgressBar,
    pub cancel: Button,
    ended: Label,
    pub back: Button,
}

fn label(class: Option<&str>) -> Label {
    let label = Label::builder().xalign(0.0).wrap(true).build();
    if let Some(class) = class {
        label.add_css_class(class);
    }
    label
}

impl BackupPage {
    pub fn new() -> Self {
        let title = label(Some("babawus-title"));
        let spinner = Spinner::builder().spinning(true).build();
        let status = label(Some("babawus-status"));
        let status_row = Box::new(Orientation::Horizontal, 8);
        status_row.append(&spinner);
        status_row.append(&status);
        let overview = label(Some("babawus-last-backup"));
        let files_left = label(None);
        let bytes_left = label(None);
        let bar = ProgressBar::new();
        let ended = label(Some("babawus-plug-in"));

        let cancel = Button::new();
        cancel.set_halign(Align::Start);
        let back = Button::new();
        back.set_halign(Align::Start);
        back.add_css_class("suggested-action");

        let root = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(SPACING)
            .margin_top(MARGIN)
            .margin_bottom(MARGIN)
            .margin_start(MARGIN)
            .margin_end(MARGIN)
            .build();
        root.append(&title);
        root.append(&status_row);
        root.append(&overview);
        root.append(&bar);
        root.append(&files_left);
        root.append(&bytes_left);
        root.append(&ended);
        root.append(&Box::builder().vexpand(true).build());
        root.append(&Separator::new(Orientation::Horizontal));
        let buttons = Box::new(Orientation::Horizontal, 8);
        buttons.append(&cancel);
        buttons.append(&back);
        root.append(&buttons);

        Self {
            root,
            title,
            spinner,
            status,
            overview,
            files_left,
            bytes_left,
            bar,
            cancel,
            ended,
            back,
        }
    }

    /// Shows a run that is still going.
    pub fn show_progress(&self, label: &str, shown: &Shown, language: Language) {
        self.set_buttons(language);
        self.title.set_label(label);
        self.spinner.set_visible(shown.busy);
        self.status.set_visible(true);
        self.status.set_label(&shown.status);
        set_optional(&self.overview, shown.overview.as_deref());
        set_optional(&self.files_left, shown.files_left.as_deref());
        set_optional(&self.bytes_left, shown.bytes_left.as_deref());
        match shown.fraction {
            Some(fraction) => {
                self.bar.set_fraction(fraction);
                self.bar.set_visible(true);
            }
            None => self.bar.set_visible(false),
        }
        self.cancel.set_visible(true);
        self.cancel.set_sensitive(shown.can_cancel);
        self.ended.set_visible(false);
        self.back.set_visible(false);
    }

    /// Shows how a run ended.
    // TODO(T-015): Show the verdict screen instead.
    pub fn show_ended(&self, label: &str, text: &str, language: Language) {
        self.set_buttons(language);
        self.title.set_label(label);
        for widget in [
            &self.status,
            &self.overview,
            &self.files_left,
            &self.bytes_left,
        ] {
            widget.set_visible(false);
        }
        self.spinner.set_visible(false);
        self.bar.set_visible(false);
        self.cancel.set_visible(false);
        self.ended.set_label(text);
        self.ended.set_visible(true);
        self.back.set_visible(true);
    }

    fn set_buttons(&self, language: Language) {
        self.cancel.set_label(Text::Cancel.get(language));
        self.cancel
            .set_tooltip_text(Some(Text::CancelTooltip.get(language)));
        self.back.set_label(Text::Back.get(language));
        self.back
            .set_tooltip_text(Some(Text::BackTooltip.get(language)));
    }
}

fn set_optional(label: &Label, text: Option<&str>) {
    label.set_label(text.unwrap_or_default());
    label.set_visible(text.is_some());
}
