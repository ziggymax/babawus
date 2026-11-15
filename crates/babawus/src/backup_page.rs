//! The backup screen: what a run is doing, with Cancel, and then the verdict. See "Provide
//! feedback during a backup operation" and "Get a clear verdict" in Requirements.md. What to show
//! comes from [`babawus_core::backup_screen`] and [`babawus_core::verdict_screen`]; this module
//! only puts it on the screen.

use babawus_core::backup_screen::Shown;
use babawus_core::language::Language;
use babawus_core::text::Text;
use babawus_core::verdict_screen::VerdictShown;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, Image, Label, Orientation, PolicyType, ProgressBar, ScrolledWindow, Separator,
    Spinner,
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
        let reason = label(None);
        let skipped = label(None);
        skipped.set_margin_start(SPACING);
        let next_step = label(Some("babawus-next-step"));
        let copied = label(None);
        let last_backup = label(None);
        let notice = label(None);
        let verdict = column();
        verdict.append(&verdict_row);
        verdict.append(&reason);
        verdict.append(&skipped);
        verdict.append(&next_step);
        verdict.append(&copied);
        verdict.append(&last_backup);

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
    /// while it goes on.
    pub fn show_verdict(
        &self,
        label: &str,
        shown: &VerdictShown,
        notice: Option<&str>,
        ejecting: bool,
        language: Language,
    ) {
        self.set_buttons(label, language);
        self.title.set_label(label);
        // The verdict starts at the top once, not on every later update.
        if !self.verdict.is_visible() {
            crate::scroll::to_top(&self.root);
        }
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
        set_optional(&self.notice, notice);
        self.cancel.set_visible(false);
        // Once the medium has been ejected, or tried to be, the notice says what to do.
        self.eject
            .set_visible(shown.offer_eject && (notice.is_none() || ejecting));
        self.eject.set_sensitive(!ejecting);
        self.back.set_visible(true);
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
    }
}

fn set_optional(label: &Label, text: Option<&str>) {
    label.set_label(text.unwrap_or_default());
    label.set_visible(text.is_some());
}
