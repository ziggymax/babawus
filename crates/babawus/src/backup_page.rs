//! The backup screen: what a run is doing, with Cancel, and then the verdict. See "Provide
//! feedback during a backup operation" and "Get a clear verdict" in Requirements.md. What to show
//! comes from [`babawus_core::backup_screen`] and [`babawus_core::verdict_screen`]; this module
//! only puts it on the screen.

use babawus_core::backup_screen::Shown;
use babawus_core::settings::Language;
use babawus_core::text::Text;
use babawus_core::verdict_screen::VerdictShown;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, Expander, Image, Label, Orientation, PolicyType, ProgressBar,
    ScrolledWindow, Separator, Spinner, TextView, WrapMode,
};

use crate::icons;

const MARGIN: i32 = 32;
const SPACING: i32 = 12;

pub struct BackupPage {
    pub root: Box,
    title: Label,

    // While the run goes on.
    progress: Box,
    spinner: Spinner,
    status: Label,
    overview: Label,
    files_left: Label,
    bytes_left: Label,
    bar: ProgressBar,
    pub cancel: Button,

    // Once it has ended.
    verdict: Box,
    mark: Image,
    verdict_title: Label,
    reason: Label,
    skipped: Label,
    next_step: Label,
    copied: Label,
    last_backup: Label,
    details: Expander,
    details_text: TextView,
    pub copy: Button,
    copied_note: Label,
    notice: Label,
    pub eject: Button,
    pub back: Button,
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
        let progress = column();
        progress.append(&status_row);
        progress.append(&overview);
        progress.append(&bar);
        progress.append(&files_left);
        progress.append(&bytes_left);

        let mark = icons::image(
            &icons::verdict_mark(babawus_core::verdict_screen::VerdictMark::Good),
            32,
        );
        let verdict_title = label(Some("babawus-verdict"));
        let verdict_row = Box::new(Orientation::Horizontal, SPACING);
        verdict_row.append(&mark);
        verdict_row.append(&verdict_title);
        let reason = label(Some("babawus-status"));
        let skipped = label(None);
        skipped.set_margin_start(SPACING);
        let next_step = label(Some("babawus-next-step"));
        let copied = label(None);
        let last_backup = label(None);
        let details_text = TextView::builder()
            .editable(false)
            .monospace(true)
            .wrap_mode(WrapMode::WordChar)
            .left_margin(8)
            .right_margin(8)
            .top_margin(8)
            .bottom_margin(8)
            .build();
        details_text.add_css_class("babawus-details");
        let copy = Button::new();
        copy.set_halign(Align::Start);
        let copied_note = label(Some("dim-label"));
        let copy_row = Box::new(Orientation::Horizontal, SPACING);
        copy_row.append(&copy);
        copy_row.append(&copied_note);
        let details_box = column();
        details_box.set_margin_top(8);
        details_box.append(&details_text);
        details_box.append(&copy_row);
        let details = Expander::builder().child(&details_box).build();
        let notice = label(None);
        let verdict = column();
        verdict.append(&verdict_row);
        verdict.append(&reason);
        verdict.append(&skipped);
        verdict.append(&next_step);
        verdict.append(&copied);
        verdict.append(&last_backup);
        verdict.append(&details);

        let cancel = Button::new();
        let eject = Button::new();
        eject.add_css_class("suggested-action");
        let back = Button::new();
        let buttons = Box::new(Orientation::Horizontal, 8);
        buttons.append(&cancel);
        buttons.append(&eject);
        buttons.append(&back);

        let page = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(SPACING)
            .margin_top(MARGIN)
            .margin_bottom(SPACING)
            .margin_start(MARGIN)
            .margin_end(MARGIN)
            .build();
        page.append(&title);
        page.append(&progress);
        page.append(&verdict);
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .child(&page)
            .build();
        // The buttons stay in view while the rest scrolls.
        let bottom = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(SPACING)
            .margin_bottom(MARGIN)
            .margin_start(MARGIN)
            .margin_end(MARGIN)
            .build();
        bottom.append(&Separator::new(Orientation::Horizontal));
        // What came of ejecting stays in view, next to the button that did it.
        bottom.append(&notice);
        bottom.append(&buttons);
        let root = Box::new(Orientation::Vertical, 0);
        root.append(&scroll);
        root.append(&bottom);

        Self {
            root,
            title,
            progress,
            spinner,
            status,
            overview,
            files_left,
            bytes_left,
            bar,
            cancel,
            verdict,
            mark,
            verdict_title,
            reason,
            skipped,
            next_step,
            copied,
            last_backup,
            details,
            details_text,
            copy,
            copied_note,
            notice,
            eject,
            back,
        }
    }

    /// Shows a run that is still going.
    pub fn show_progress(&self, label: &str, shown: &Shown, language: Language) {
        self.set_buttons(label, language);
        self.title.set_label(label);
        self.progress.set_visible(true);
        self.verdict.set_visible(false);
        self.spinner.set_visible(shown.busy);
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
        self.notice.set_visible(false);
        self.eject.set_visible(false);
        self.back.set_visible(false);
    }

    /// Shows the verdict. `notice` is what came of ejecting, if anything, and `ejecting` is set
    /// while it goes on. `copied` says the details have just been copied.
    pub fn show_verdict(
        &self,
        label: &str,
        shown: &VerdictShown,
        notice: Option<&str>,
        ejecting: bool,
        copied: bool,
        language: Language,
    ) {
        self.set_buttons(label, language);
        self.title.set_label(label);
        self.progress.set_visible(false);
        self.verdict.set_visible(true);
        self.mark
            .set_paintable(Some(&icons::verdict_mark(shown.mark)));
        self.verdict_title.set_label(&shown.title);
        set_optional(&self.reason, shown.reason.as_deref());
        let skipped: Vec<String> = shown
            .skipped
            .iter()
            .map(|group| format!("• {group}"))
            .collect();
        let skipped = (!skipped.is_empty()).then(|| skipped.join("\n"));
        set_optional(&self.skipped, skipped.as_deref());
        set_optional(&self.next_step, shown.next_step.as_deref());
        set_optional(&self.copied, shown.copied.as_deref());
        self.last_backup.set_label(&shown.last_backup);
        self.details.set_visible(!shown.details.is_empty());
        self.details_text.buffer().set_text(&shown.details);
        set_optional(
            &self.copied_note,
            copied.then(|| Text::DetailsCopied.get(language)),
        );
        set_optional(&self.notice, notice);
        self.cancel.set_visible(false);
        // Once the medium has been ejected, or tried to be, the notice says what to do.
        self.eject
            .set_visible(shown.offer_eject && (notice.is_none() || ejecting));
        self.eject.set_sensitive(!ejecting);
        self.back.set_visible(true);
    }

    /// The details as shown, to copy them.
    pub fn details_text(&self) -> gtk4::TextBuffer {
        self.details_text.buffer()
    }

    fn set_buttons(&self, label: &str, language: Language) {
        self.cancel.set_label(Text::Cancel.get(language));
        self.cancel
            .set_tooltip_text(Some(Text::CancelTooltip.get(language)));
        self.eject
            .set_label(&Text::EjectNow.fill(language, &[("label", label)]));
        self.eject
            .set_tooltip_text(Some(Text::EjectNowTooltip.get(language)));
        self.back.set_label(Text::Back.get(language));
        self.back
            .set_tooltip_text(Some(Text::BackTooltip.get(language)));
        self.details
            .set_label(Some(Text::DetailsTitle.get(language)));
        self.details
            .set_tooltip_text(Some(Text::DetailsTooltip.get(language)));
        self.copy.set_label(Text::CopyDetails.get(language));
        self.copy
            .set_tooltip_text(Some(Text::CopyDetailsTooltip.get(language)));
    }
}

fn set_optional(label: &Label, text: Option<&str>) {
    label.set_label(text.unwrap_or_default());
    label.set_visible(text.is_some());
}
