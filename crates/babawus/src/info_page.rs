//! The history and "What's backed up" screens: a list about one medium, with Back. See "View the
//! history of a medium" and "See which files and folders a medium covers" in Requirements.md.
//! What to show comes from [`babawus_core::history_screen`] and
//! [`babawus_core::coverage_screen`]; this module only puts it on the screen.

use babawus_core::coverage_screen::{CoverageShown, Picture};
use babawus_core::history_screen::HistoryRow;
use babawus_core::settings::Language;
use babawus_core::start_screen::Mark;
use babawus_core::text::Text;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, Image, Label, ListBox, Orientation, PolicyType, ScrolledWindow,
    SelectionMode, Separator, Spinner,
};

use crate::icons;

const MARGIN: i32 = 32;
const SPACING: i32 = 12;
const PICTURE_SIZE: i32 = 24;

pub struct InfoPage {
    pub root: Box,
    title: Label,
    heading: Label,
    loading: Box,
    loading_text: Label,
    intro: Label,
    list: ListBox,
    left_out_heading: Label,
    left_out: Label,
    pub back: Button,
}

fn label(class: Option<&str>) -> Label {
    let label = Label::builder().xalign(0.0).wrap(true).build();
    if let Some(class) = class {
        label.add_css_class(class);
    }
    label
}

impl InfoPage {
    pub fn new() -> Self {
        let title = label(Some("babawus-title"));
        let heading = label(Some("babawus-status"));
        let spinner = Spinner::builder().spinning(true).build();
        let loading_text = label(None);
        let loading = Box::new(Orientation::Horizontal, 8);
        loading.append(&spinner);
        loading.append(&loading_text);
        let intro = label(None);
        let list = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .build();
        list.add_css_class("boxed-list");
        list.set_show_separators(true);
        let left_out_heading = label(Some("babawus-next-step"));
        let left_out = label(None);
        left_out.set_margin_start(SPACING);

        let page = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(SPACING)
            .margin_top(MARGIN)
            .margin_bottom(SPACING)
            .margin_start(MARGIN)
            .margin_end(MARGIN)
            .build();
        page.append(&title);
        page.append(&heading);
        page.append(&loading);
        page.append(&intro);
        page.append(&list);
        page.append(&left_out_heading);
        page.append(&left_out);
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .child(&page)
            .build();

        let back = Button::new();
        back.set_halign(Align::Start);
        // Back stays in view while the rest scrolls.
        let bottom = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(SPACING)
            .margin_bottom(MARGIN)
            .margin_start(MARGIN)
            .margin_end(MARGIN)
            .build();
        bottom.append(&Separator::new(Orientation::Horizontal));
        bottom.append(&back);
        let root = Box::new(Orientation::Vertical, 0);
        root.append(&scroll);
        root.append(&bottom);

        Self {
            root,
            title,
            heading,
            loading,
            loading_text,
            intro,
            list,
            left_out_heading,
            left_out,
            back,
        }
    }

    /// Shows the medium labelled `label` under `heading`, with nothing below yet.
    fn start(&self, medium: &str, heading: Text, language: Language) {
        self.title.set_label(medium);
        self.heading.set_label(heading.get(language));
        self.back.set_label(Text::Back.get(language));
        self.back
            .set_tooltip_text(Some(Text::BackTooltip.get(language)));
        self.loading.set_visible(false);
        self.list.remove_all();
        self.list.set_visible(false);
        set_optional(&self.intro, None);
        set_optional(&self.left_out_heading, None);
        set_optional(&self.left_out, None);
    }

    /// Shows a spinner while the medium is read.
    pub fn show_loading(&self, medium: &str, heading: Text, language: Language) {
        self.start(medium, heading, language);
        self.loading_text.set_label(Text::Loading.get(language));
        self.loading.set_visible(true);
    }

    /// Shows the history. `rows` is `None` if the log could not be read.
    pub fn show_history(&self, medium: &str, rows: Option<&[HistoryRow]>, language: Language) {
        self.start(medium, Text::HistoryHeading, language);
        let rows = match rows {
            None => return set_optional(&self.intro, Some(Text::HistoryUnreadable.get(language))),
            Some([]) => return set_optional(&self.intro, Some(Text::HistoryEmpty.get(language))),
            Some(rows) => rows,
        };
        for row in rows {
            let mark = icons::image(&icons::verdict_mark(row.mark), PICTURE_SIZE);
            mark.set_valign(Align::Start);
            let verdict = label(Some("babawus-next-step"));
            verdict.set_label(&row.verdict);
            let when = label(Some("dim-label"));
            when.set_label(&row.when);
            when.set_xalign(1.0);
            when.set_hexpand(true);
            let top = Box::new(Orientation::Horizontal, SPACING);
            top.append(&verdict);
            top.append(&when);
            let text = Box::new(Orientation::Vertical, 2);
            text.set_hexpand(true);
            text.append(&top);
            let lines = [&row.reason, &row.copied, &row.skipped]
                .into_iter()
                .flatten();
            for line in lines.chain([&row.who]) {
                let line_label = label(None);
                line_label.set_label(line);
                text.append(&line_label);
            }
            self.list.append(&list_row(&mark, &text));
        }
        self.list.set_visible(true);
    }

    /// Shows what is backed up.
    pub fn show_coverage(&self, medium: &str, shown: &CoverageShown, language: Language) {
        self.start(medium, Text::CoverageHeading, language);
        set_optional(&self.intro, Some(&shown.intro));
        for item in &shown.items {
            let picture = match item.picture {
                Picture::Folder => themed("folder", PICTURE_SIZE),
                Picture::File => themed("text-x-generic", PICTURE_SIZE),
                Picture::NotBackedUp => icons::image(&icons::mark(Mark::Attention), PICTURE_SIZE),
            };
            picture.set_valign(Align::Start);
            let text = Box::new(Orientation::Vertical, 2);
            text.set_hexpand(true);
            let name = label(Some("babawus-next-step"));
            name.set_label(&item.name);
            text.append(&name);
            if let Some(note) = &item.note {
                let note_label = label(Some("dim-label"));
                note_label.set_label(note);
                text.append(&note_label);
            }
            self.list.append(&list_row(&picture, &text));
        }
        self.list.set_visible(!shown.items.is_empty());
        set_optional(&self.left_out_heading, shown.left_out_heading.as_deref());
        let left_out: Vec<String> = shown
            .left_out
            .iter()
            .map(|each| format!("• {each}"))
            .collect();
        let left_out = (!left_out.is_empty()).then(|| left_out.join("\n"));
        set_optional(&self.left_out, left_out.as_deref());
    }
}

/// A row of the list: a picture, and texts beside it.
fn list_row(picture: &Image, text: &Box) -> Box {
    let row = Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(SPACING)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    row.append(picture);
    row.append(text);
    row
}

/// An icon from the desktop's theme.
fn themed(name: &str, size: i32) -> Image {
    let image = Image::from_icon_name(name);
    image.set_pixel_size(size);
    image
}

fn set_optional(label: &Label, text: Option<&str>) {
    label.set_label(text.unwrap_or_default());
    label.set_visible(text.is_some());
}
