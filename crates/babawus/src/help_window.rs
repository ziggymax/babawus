//! The Help window: the built-in Help in a window of its own, which isn't modal, with a search
//! that finds as you type. The text and the search come from [`babawus_core::help`]. See
//! "Read the Help" in Requirements.md, and issues 151 and 154.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use babawus_core::help::{self, Block};
use babawus_core::language::Language;
use babawus_core::text::Text;
use gtk4::prelude::*;
use gtk4::{
    Application, Box as GtkBox, Button, EventControllerKey, Label, Orientation, PolicyType,
    ScrolledWindow, SearchEntry, TextBuffer, TextTag, TextView, WrapMode, gdk, glib,
};

const WIDTH: i32 = 640;
const HEIGHT: i32 = 700;
const MARGIN: i32 = 12;
/// A match is scrolled to this far down the window, so that the lines above it show too.
const MATCH_POSITION: f64 = 0.3;

const HEADING: &str = "heading";
const ENTRY: &str = "entry";
const BULLET: &str = "bullet";
const PARAGRAPH: &str = "paragraph";
const MATCH: &str = "match";
const CURRENT: &str = "current";
/// Bold, as Pango counts weights.
const BOLD: i32 = 700;

/// The Help window.
pub struct HelpWindow {
    pub window: gtk4::Window,
    search: SearchEntry,
    count: Label,
    previous: Button,
    next: Button,
    view: TextView,
    language: Cell<Language>,
    /// Where the current search's matches are, as character offsets and lengths.
    matches: RefCell<Vec<(usize, usize)>>,
    /// The match shown now, an index into `matches`.
    current: Cell<usize>,
}

impl HelpWindow {
    /// Opens the Help in `language`.
    pub fn open(app: &Application, language: Language) -> Rc<Self> {
        let buffer = TextBuffer::new(None);
        add_tags(&buffer);
        let view = TextView::builder()
            .buffer(&buffer)
            .editable(false)
            .cursor_visible(false)
            .wrap_mode(WrapMode::WordChar)
            .left_margin(MARGIN * 2)
            .right_margin(MARGIN * 2)
            .top_margin(MARGIN)
            .bottom_margin(MARGIN * 2)
            .build();
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .child(&view)
            .build();
        let search = SearchEntry::builder().hexpand(true).build();
        let count = Label::builder().width_chars(12).xalign(1.0).build();
        count.add_css_class("dim-label");
        // Arrows that go to the match before and the next, as in GNOME's own search bars.
        let previous = Button::from_icon_name("go-up-symbolic");
        let next = Button::from_icon_name("go-down-symbolic");
        let arrows = GtkBox::new(Orientation::Horizontal, 0);
        arrows.add_css_class("linked");
        arrows.append(&previous);
        arrows.append(&next);
        let row = GtkBox::new(Orientation::Horizontal, MARGIN);
        row.set_margin_top(MARGIN);
        row.set_margin_bottom(MARGIN);
        row.set_margin_start(MARGIN);
        row.set_margin_end(MARGIN);
        row.append(&search);
        row.append(&count);
        row.append(&arrows);
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.append(&row);
        content.append(&gtk4::Separator::new(Orientation::Horizontal));
        content.append(&scroll);
        let window = gtk4::Window::builder()
            .application(app)
            .default_width(WIDTH)
            .default_height(HEIGHT)
            .child(&content)
            .build();
        let help_window = Rc::new(Self {
            window,
            search,
            count,
            previous,
            next,
            view,
            language: Cell::new(language),
            matches: RefCell::new(Vec::new()),
            current: Cell::new(0),
        });
        help_window.connect_search();
        help_window.set_texts();
        help_window.window.present();
        help_window.search.grab_focus();
        help_window
    }

    /// Shows the Help in `language` instead, keeping the search.
    pub fn set_language(&self, language: Language) {
        self.language.set(language);
        self.set_texts();
    }

    fn connect_search(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        self.search.connect_search_changed(move |_| {
            if let Some(help_window) = weak.upgrade() {
                help_window.find();
            }
        });
        let weak = Rc::downgrade(self);
        self.search.connect_activate(move |_| {
            if let Some(help_window) = weak.upgrade() {
                help_window.step(true);
            }
        });
        let weak = Rc::downgrade(self);
        self.search.connect_next_match(move |_| {
            if let Some(help_window) = weak.upgrade() {
                help_window.step(true);
            }
        });
        let weak = Rc::downgrade(self);
        self.search.connect_previous_match(move |_| {
            if let Some(help_window) = weak.upgrade() {
                help_window.step(false);
            }
        });
        // Shift+Enter goes back, which the search field doesn't do by itself.
        let keys = EventControllerKey::new();
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, modifiers| {
            let enter = matches!(key, gdk::Key::Return | gdk::Key::KP_Enter);
            if enter
                && modifiers.contains(gdk::ModifierType::SHIFT_MASK)
                && let Some(help_window) = weak.upgrade()
            {
                help_window.step(false);
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.search.add_controller(keys);
        for (button, forward) in [(&self.previous, false), (&self.next, true)] {
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(help_window) = weak.upgrade() {
                    help_window.step(forward);
                }
            });
        }
    }

    /// Sets the window's texts and the Help itself, and searches again.
    fn set_texts(&self) {
        let language = self.language.get();
        self.window.set_title(Some(Text::HelpTitle.get(language)));
        self.search
            .set_placeholder_text(Some(Text::HelpSearch.get(language)));
        self.search
            .set_tooltip_text(Some(Text::HelpSearchTooltip.get(language)));
        self.previous
            .set_tooltip_text(Some(Text::HelpPrevious.get(language)));
        self.next
            .set_tooltip_text(Some(Text::HelpNext.get(language)));
        let buffer = self.view.buffer();
        buffer.set_text("");
        let mut end = buffer.end_iter();
        let mut first = true;
        for block in help::blocks(Text::Help.get(language)) {
            if !first {
                buffer.insert(&mut end, "\n");
            }
            first = false;
            let (text, tag) = match &block {
                Block::Heading(text) => (text.clone(), HEADING),
                Block::Entry(text) => (text.clone(), ENTRY),
                Block::Bullet(text) => (format!("•  {text}"), BULLET),
                Block::Paragraph(text) => (text.clone(), PARAGRAPH),
            };
            buffer.insert_with_tags_by_name(&mut end, &text, &[tag]);
        }
        self.find();
    }

    /// Finds what is in the search field, highlights every match, and shows the first.
    fn find(&self) {
        let buffer = self.view.buffer();
        let (start, end) = buffer.bounds();
        buffer.remove_tag_by_name(MATCH, &start, &end);
        buffer.remove_tag_by_name(CURRENT, &start, &end);
        let text = buffer.text(&start, &end, false);
        let needle = self.search.text();
        let found = help::find(&text, &needle);
        let first = help::first_to_show(&text, &needle, &found);
        for &(offset, length) in &found {
            let (from, to) = range(&buffer, offset, length);
            buffer.apply_tag_by_name(MATCH, &from, &to);
        }
        *self.matches.borrow_mut() = found;
        self.current.set(first);
        self.show_current();
    }

    /// Goes to the next match, or the one before, round from the end to the start.
    fn step(&self, forward: bool) {
        let total = self.matches.borrow().len();
        if total == 0 {
            return;
        }
        let current = self.current.get();
        let next = if forward {
            (current + 1) % total
        } else {
            (current + total - 1) % total
        };
        self.current.set(next);
        self.show_current();
    }

    /// Marks the current match, scrolls to it, and says which one it is.
    fn show_current(&self) {
        let language = self.language.get();
        let buffer = self.view.buffer();
        let (start, end) = buffer.bounds();
        buffer.remove_tag_by_name(CURRENT, &start, &end);
        let matches = self.matches.borrow();
        self.previous.set_sensitive(!matches.is_empty());
        self.next.set_sensitive(!matches.is_empty());
        if matches.is_empty() {
            let searching = !self.search.text().trim().is_empty();
            self.count.set_label(if searching {
                Text::HelpNotFound.get(language)
            } else {
                ""
            });
            return;
        }
        let current = self.current.get();
        let (offset, length) = matches[current];
        let (from, to) = range(&buffer, offset, length);
        buffer.apply_tag_by_name(CURRENT, &from, &to);
        let mark = match buffer.mark(CURRENT) {
            Some(mark) => {
                buffer.move_mark(&mark, &from);
                mark
            }
            None => buffer.create_mark(Some(CURRENT), &from, true),
        };
        scroll_into_view(&self.view, &mark);
        self.count.set_label(&Text::HelpFound.fill(
            language,
            &[
                ("current", &(current + 1).to_string()),
                ("total", &matches.len().to_string()),
            ],
        ));
    }
}

/// How often, and how far apart, [`scroll_into_view`] tries again.
const SCROLL_TRIES: u32 = 20;
const SCROLL_RETRY: std::time::Duration = std::time::Duration::from_millis(50);

/// Scrolls `view` so that `mark` is in sight. Right after a new text is set, such as after a change
/// of language, the text isn't laid out yet, and a scroll lands short of the mark, even for a mark
/// rather than an iterator; so it tries again until the mark is in sight, for up to a second
/// (T-088).
fn scroll_into_view(view: &TextView, mark: &gtk4::TextMark) {
    view.scroll_to_mark(mark, 0.0, true, 0.0, MATCH_POSITION);
    let weak_view = view.downgrade();
    let mark = mark.clone();
    let tries = Cell::new(0);
    glib::timeout_add_local(SCROLL_RETRY, move || {
        let Some(view) = weak_view.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if mark.is_deleted() {
            return glib::ControlFlow::Break;
        }
        let shown = view.visible_rect();
        let place = view.iter_location(&view.buffer().iter_at_mark(&mark));
        let in_sight = shown.height() > 0
            && place.y() >= shown.y()
            && place.y() + place.height() <= shown.y() + shown.height();
        tries.set(tries.get() + 1);
        if in_sight || tries.get() >= SCROLL_TRIES {
            return glib::ControlFlow::Break;
        }
        view.scroll_to_mark(&mark, 0.0, true, 0.0, MATCH_POSITION);
        glib::ControlFlow::Continue
    });
}

/// The iterators around `length` characters from `offset`.
fn range(buffer: &TextBuffer, offset: usize, length: usize) -> (gtk4::TextIter, gtk4::TextIter) {
    let from = buffer.iter_at_offset(offset as i32);
    let to = buffer.iter_at_offset((offset + length) as i32);
    (from, to)
}

fn add_tags(buffer: &TextBuffer) {
    let table = buffer.tag_table();
    let heading = TextTag::builder()
        .name(HEADING)
        .weight(BOLD)
        .scale(1.3)
        .pixels_above_lines(18)
        .pixels_below_lines(6)
        .build();
    let entry = TextTag::builder()
        .name(ENTRY)
        .weight(BOLD)
        .pixels_above_lines(12)
        .build();
    let bullet = TextTag::builder()
        .name(BULLET)
        .left_margin(MARGIN * 3)
        .indent(-MARGIN)
        .pixels_above_lines(2)
        .build();
    let paragraph = TextTag::builder()
        .name(PARAGRAPH)
        .pixels_above_lines(6)
        .build();
    let found = TextTag::builder()
        .name(MATCH)
        .background("#fce94f")
        .foreground("#000000")
        .build();
    let current = TextTag::builder()
        .name(CURRENT)
        .background("#f57900")
        .foreground("#000000")
        .build();
    for tag in [heading, entry, bullet, paragraph, found, current] {
        table.add(&tag);
    }
}
