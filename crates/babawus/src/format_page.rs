//! The format view: choose a drive, a label and a file system, confirm, then wait while it is
//! formatted, and see how it went. See "Format USB media" in Requirements.md and issues
//! 78-81. What to show comes from [`babawus_core::format_screen`]; this module only puts it on
//! the screen. The window keeps what the poweruser has chosen in the widgets themselves.

use babawus_core::format_screen::{DriveChoice, ResultShown, ext4_allowed, ext4_note};
use babawus_core::settings::Language;
use babawus_core::start_screen::Mark;
use babawus_core::text::Text;
use babawus_core::verdict_screen::VerdictMark;
use babawus_core::versions::Format;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, CheckButton, Entry, Image, Label, ListBox, ListBoxRow, Orientation,
    PolicyType, ScrolledWindow, SelectionMode, Separator, Spinner, Stack,
};

use crate::icons;

const MARGIN: i32 = 32;
const SPACING: i32 = 12;

/// Which part of the format view is showing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Choose,
    Working,
    Result,
}

pub struct FormatPage {
    pub root: Box,
    parts: Stack,
    title: Label,

    // Choosing.
    intro: Label,
    pub drives: ListBox,
    no_drives: Label,
    label_heading: Label,
    pub label: Entry,
    label_problem: Label,
    file_system_heading: Label,
    pub exfat: CheckButton,
    exfat_detail: Label,
    pub ext4: CheckButton,
    ext4_detail: Label,
    /// Why Ext4 can't be chosen, or that it asks for a password.
    ext4_note: Label,
    /// Whether this account is an administrator, who may choose Ext4 (issue 108).
    administrator: bool,
    warning: Label,
    warning_row: Box,
    pub understood: CheckButton,
    pub erase: Button,

    // Formatting.
    working: Label,
    working_detail: Label,

    // Done or failed.
    result_mark: Image,
    result_title: Label,
    result_detail: Label,
    result_message: Label,
    pub add: Button,
    pub try_again: Button,

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

/// A choice with a line in plain words below it.
fn choice(button: &CheckButton, detail: &Label) -> Box {
    detail.add_css_class("dim-label");
    detail.set_margin_start(28);
    let choice = Box::new(Orientation::Vertical, 2);
    choice.append(button);
    choice.append(detail);
    choice
}

impl FormatPage {
    /// `administrator` says whether this account may choose Ext4.
    pub fn new(administrator: bool) -> Self {
        let title = label(Some("babawus-title"));

        let intro = label(None);
        let drives = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .build();
        drives.add_css_class("boxed-list");
        drives.set_show_separators(true);
        let no_drives = label(Some("babawus-status"));
        let label_heading = label(Some("babawus-next-step"));
        let label_entry = Entry::builder()
            .max_length(babawus_core::format::MAX_LABEL as i32)
            .width_chars(14)
            .halign(Align::Start)
            .build();
        let label_problem = label(Some("dim-label"));
        let file_system_heading = label(Some("babawus-next-step"));
        let exfat = CheckButton::new();
        let ext4 = CheckButton::new();
        ext4.set_group(Some(&exfat));
        exfat.set_active(true);
        let exfat_detail = label(None);
        let ext4_detail = label(None);
        let ext4_note = label(None);
        ext4_note.set_margin_start(28);
        ext4_note.set_visible(false);
        ext4.set_sensitive(ext4_allowed(administrator));
        let warning = label(Some("babawus-next-step"));
        warning.set_hexpand(true);
        let warning_mark = icons::image(&icons::mark(Mark::Attention), 24);
        warning_mark.set_valign(Align::Start);
        let warning_row = Box::new(Orientation::Horizontal, SPACING);
        warning_row.append(&warning_mark);
        warning_row.append(&warning);
        let understood = CheckButton::new();
        let choose = column();
        choose.append(&intro);
        choose.append(&drives);
        choose.append(&no_drives);
        choose.append(&label_heading);
        choose.append(&label_entry);
        choose.append(&label_problem);
        choose.append(&file_system_heading);
        choose.append(&choice(&exfat, &exfat_detail));
        let ext4_choice = choice(&ext4, &ext4_detail);
        ext4_choice.append(&ext4_note);
        choose.append(&ext4_choice);
        choose.append(&warning_row);
        choose.append(&understood);

        let spinner = Spinner::builder().spinning(true).build();
        let working = label(Some("babawus-status"));
        let working_row = Box::new(Orientation::Horizontal, 8);
        working_row.append(&spinner);
        working_row.append(&working);
        let working_detail = label(Some("babawus-next-step"));
        let working_part = column();
        working_part.append(&working_row);
        working_part.append(&working_detail);

        let result_mark = icons::image(&icons::verdict_mark(VerdictMark::Good), 32);
        let result_title = label(Some("babawus-verdict"));
        let result_row = Box::new(Orientation::Horizontal, SPACING);
        result_row.append(&result_mark);
        result_row.append(&result_title);
        let result_detail = label(Some("babawus-status"));
        let result_message = label(Some("dim-label"));
        result_message.set_selectable(true);
        let result_part = column();
        result_part.append(&result_row);
        result_part.append(&result_detail);
        result_part.append(&result_message);

        let parts = Stack::new();
        parts.add_named(&choose, Some("choose"));
        parts.add_named(&working_part, Some("working"));
        parts.add_named(&result_part, Some("result"));

        let page = column();
        page.set_margin_top(MARGIN);
        page.set_margin_bottom(SPACING);
        page.set_margin_start(MARGIN);
        page.set_margin_end(MARGIN);
        page.append(&title);
        page.append(&parts);
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .child(&page)
            .build();

        let erase = Button::new();
        erase.add_css_class("destructive-action");
        let add = Button::new();
        add.add_css_class("suggested-action");
        let try_again = Button::new();
        let back = Button::new();
        let buttons = Box::new(Orientation::Horizontal, 8);
        buttons.append(&erase);
        buttons.append(&add);
        buttons.append(&try_again);
        buttons.append(&back);
        // The buttons stay in view while the rest scrolls.
        let bottom = column();
        bottom.set_margin_bottom(MARGIN);
        bottom.set_margin_start(MARGIN);
        bottom.set_margin_end(MARGIN);
        bottom.append(&Separator::new(Orientation::Horizontal));
        bottom.append(&buttons);
        let root = Box::new(Orientation::Vertical, 0);
        root.append(&scroll);
        root.append(&bottom);

        Self {
            root,
            parts,
            title,
            intro,
            drives,
            no_drives,
            label_heading,
            label: label_entry,
            label_problem,
            file_system_heading,
            exfat,
            exfat_detail,
            ext4,
            ext4_detail,
            ext4_note,
            administrator,
            warning,
            warning_row,
            understood,
            erase,
            working,
            working_detail,
            result_mark,
            result_title,
            result_detail,
            result_message,
            add,
            try_again,
            back,
        }
    }

    /// Sets the texts that don't depend on what is chosen.
    pub fn set_texts(&self, language: Language) {
        self.title.set_label(Text::FormatHeading.get(language));
        self.intro.set_label(Text::FormatIntro.get(language));
        self.no_drives.set_label(Text::FormatNoDrives.get(language));
        self.label_heading
            .set_label(Text::FormatLabel.get(language));
        self.file_system_heading
            .set_label(Text::FormatFileSystem.get(language));
        self.exfat.set_label(Some(Text::FormatExFat.get(language)));
        self.exfat_detail
            .set_label(Text::FormatExFatDetail.get(language));
        self.ext4.set_label(Some(Text::FormatExt4.get(language)));
        self.ext4_detail
            .set_label(Text::FormatExt4Detail.get(language));
        self.understood
            .set_label(Some(Text::FormatUnderstood.get(language)));
        self.erase.set_label(Text::FormatErase.get(language));
        self.erase
            .set_tooltip_text(Some(Text::FormatEraseTooltip.get(language)));
        self.working_detail
            .set_label(Text::FormatWorkingDetail.get(language));
        self.add.set_label(Text::AddComputer.get(language));
        self.add
            .set_tooltip_text(Some(Text::AddComputerTooltip.get(language)));
        self.try_again.set_label(Text::TryAgain.get(language));
        self.try_again
            .set_tooltip_text(Some(Text::TryAgainTooltip.get(language)));
        self.back.set_label(Text::Back.get(language));
        self.back
            .set_tooltip_text(Some(Text::BackTooltip.get(language)));
    }

    /// Puts `choices` in the list, and selects the drive with the ID `selected` if it is there.
    /// Each row is named by its drive ID.
    pub fn set_drives(&self, choices: &[DriveChoice], selected: Option<&str>) {
        self.drives.remove_all();
        for choice in choices {
            let name = label(Some("babawus-next-step"));
            name.set_label(&choice.name);
            let contents = label(Some("dim-label"));
            contents.set_label(&choice.contents);
            let text = Box::new(Orientation::Vertical, 2);
            text.set_hexpand(true);
            text.append(&name);
            text.append(&contents);
            let content = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(SPACING)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(8)
                .margin_end(8)
                .build();
            content.append(&icons::image(&icons::stick(false), 32));
            content.append(&text);
            let row = ListBoxRow::builder().child(&content).build();
            row.set_widget_name(&choice.id);
            self.drives.append(&row);
            if Some(choice.id.as_str()) == selected {
                self.drives.select_row(Some(&row));
            }
        }
        self.drives.set_visible(!choices.is_empty());
        self.no_drives.set_visible(choices.is_empty());
    }

    /// The udisks2 drive ID of the chosen drive.
    pub fn chosen_drive(&self) -> Option<String> {
        self.drives
            .selected_row()
            .map(|row| row.widget_name().to_string())
    }

    pub fn format(&self) -> Format {
        if self.ext4.is_active() {
            Format::Ext4
        } else {
            Format::ExFat
        }
    }

    /// Shows the line under the Ext4 choice that goes with this account and what is chosen.
    pub fn show_ext4_note(&self, language: Language) {
        let note = ext4_note(self.administrator, self.ext4.is_active(), language);
        set_optional(&self.ext4_note, note);
    }

    /// Shows what goes with the current choice: the problem with the label, the warning, and
    /// whether "Erase and format" can be pressed.
    pub fn show_choice(
        &self,
        label_problem: Option<&str>,
        warning: Option<&str>,
        can_format: bool,
    ) {
        self.show(Part::Choose);
        set_optional(&self.label_problem, label_problem);
        self.warning.set_label(warning.unwrap_or_default());
        self.warning_row.set_visible(warning.is_some());
        self.understood.set_visible(warning.is_some());
        self.erase.set_sensitive(can_format);
    }

    pub fn show_working(&self, medium: &str, language: Language) {
        self.show(Part::Working);
        self.working
            .set_label(&Text::FormatWorking.fill(language, &[("label", medium)]));
    }

    pub fn show_result(&self, shown: &ResultShown) {
        self.show(Part::Result);
        let mark = if shown.succeeded {
            VerdictMark::Good
        } else {
            VerdictMark::Failed
        };
        self.result_mark
            .set_paintable(Some(&icons::verdict_mark(mark)));
        self.result_title.set_label(&shown.title);
        self.result_detail.set_label(&shown.detail);
        set_optional(&self.result_message, shown.message.as_deref());
        self.add.set_visible(shown.succeeded);
        self.try_again.set_visible(!shown.succeeded);
    }

    fn show(&self, part: Part) {
        let name = match part {
            Part::Choose => "choose",
            Part::Working => "working",
            Part::Result => "result",
        };
        // Only a change of part starts at the top: this runs on every keystroke too.
        if self.parts.visible_child_name().as_deref() != Some(name) {
            crate::scroll::to_top(&self.root);
            self.parts.set_visible_child_name(name);
        }
        self.erase.set_visible(part == Part::Choose);
        self.add.set_visible(false);
        self.try_again.set_visible(false);
        // Formatting can't be stopped, so there is no way back while it runs.
        self.back.set_visible(part != Part::Working);
    }
}

fn set_optional(label: &Label, text: Option<&str>) {
    label.set_label(text.unwrap_or_default());
    label.set_visible(text.is_some());
}
