//! The log window: a stick's log, under Advanced, one row per line, with one run highlighted.
//! It isn't modal, and follows changes to the log until it is closed. What to show comes from
//! [`babawus_core::log_view`]. See "See a stick's log" in Requirements.md and issue 135.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::time::Duration;

use babawus_core::fs::RealFs;
use babawus_core::language::Language;
use babawus_core::log_view::{Colours, LogLine, Style, is_dark_theme, read_log_lines};
use babawus_core::medium::LOG_FILE;
use babawus_core::text::Text;
use babawus_core::verdict_screen::VerdictMark;
use gtk4::prelude::*;
use gtk4::{
    Application, PolicyType, ScrolledWindow, TextBuffer, TextTag, TextView, WrapMode, gio, glib,
};

const WIDTH: i32 = 1100;
const HEIGHT: i32 = 640;
/// A change to the log is read after this pause, so that a backup writing many entries in a row
/// doesn't make the window read the log for each of them.
const SETTLE: Duration = Duration::from_millis(300);
/// The highlighted run is scrolled to this far down the window, so that a line or two above it
/// shows too.
const RUN_POSITION: f64 = 0.2;
/// How long the window waits for a new text to be laid out before scrolling in it.
const LAID_OUT: Duration = Duration::from_millis(150);
/// Room below each line, inside its own background. Some fixed-width fonts, such as DejaVu Sans
/// Mono on Ubuntu, draw the underscore just below the line, where the next line's shading or
/// highlight would be painted over it (T-148, issue 178).
const BELOW_LINES: i32 = 2;

const HIGHLIGHT: &str = "highlight";
/// A fixed-width font for the whole log. The text view's own setting for it relies on the
/// theme, which Mint-Y doesn't follow.
const MONOSPACE: &str = "monospace";
const SHADED: &str = "shaded";

/// A log window for one stick.
pub struct LogWindow {
    pub window: gtk4::Window,
    view: TextView,
    scroll: ScrolledWindow,
    /// The root folder of the stick's file system.
    root: PathBuf,
    language: Language,
    /// The run to highlight.
    run_id: RefCell<String>,
    /// The lines shown now.
    lines: RefCell<Vec<LogLine>>,
    /// The log has changed, and will be read again after a pause.
    reading_soon: Cell<bool>,
    /// Scroll to the highlighted run once the log has been read, instead of keeping the place.
    scroll_to_run: Cell<bool>,
    /// The styles have the colours for a dark theme. Changing them lays the log out again, so it
    /// is only done when the theme turns from light to dark or back.
    dark: Cell<bool>,
    /// Kept so that changes keep being reported while the window is open.
    _monitor: Option<gio::FileMonitor>,
}

impl LogWindow {
    /// Opens a log window for the stick labelled `label`, whose file system is at `root`, with
    /// the run `run_id` highlighted.
    pub fn open(
        app: &Application,
        label: &str,
        root: PathBuf,
        run_id: &str,
        language: Language,
    ) -> Rc<Self> {
        let buffer = TextBuffer::new(None);
        add_tags(&buffer);
        let view = TextView::builder()
            .buffer(&buffer)
            .editable(false)
            .cursor_visible(false)
            .monospace(true)
            .wrap_mode(WrapMode::None)
            .pixels_below_lines(BELOW_LINES)
            .left_margin(12)
            .right_margin(12)
            .top_margin(8)
            .bottom_margin(8)
            .build();
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vscrollbar_policy(PolicyType::Automatic)
            .child(&view)
            .build();
        let window = gtk4::Window::builder()
            .application(app)
            .title(Text::LogWindowTitle.fill(language, &[("label", label)]))
            .default_width(WIDTH)
            .default_height(HEIGHT)
            .child(&scroll)
            .build();
        let file = gio::File::for_path(root.join(LOG_FILE));
        let monitor = file
            .monitor_file(gio::FileMonitorFlags::NONE, None::<&gio::Cancellable>)
            .ok();
        let log_window = Rc::new(Self {
            window,
            view,
            scroll,
            root,
            language,
            run_id: RefCell::new(run_id.to_owned()),
            lines: RefCell::new(Vec::new()),
            reading_soon: Cell::new(false),
            scroll_to_run: Cell::new(true),
            dark: Cell::new(false),
            _monitor: monitor.clone(),
        });
        if let Some(monitor) = monitor {
            let weak = Rc::downgrade(&log_window);
            monitor.connect_changed(move |_, _, _, event| {
                if matches!(
                    event,
                    gio::FileMonitorEvent::Changed
                        | gio::FileMonitorEvent::ChangesDoneHint
                        | gio::FileMonitorEvent::Created
                        | gio::FileMonitorEvent::Deleted
                        | gio::FileMonitorEvent::Renamed
                        | gio::FileMonitorEvent::MovedIn
                ) && let Some(log_window) = weak.upgrade()
                {
                    log_window.read_soon();
                }
            });
        }
        // The colours suit the theme, which the view knows once it is in the window, and checks
        // again when it is shown, and when the theme changes while it is open (issue 189).
        log_window.paint();
        let weak = Rc::downgrade(&log_window);
        log_window.window.connect_map(move |_| {
            if let Some(log_window) = weak.upgrade() {
                log_window.paint();
            }
        });
        if let Some(settings) = gtk4::Settings::default() {
            let weak = Rc::downgrade(&log_window);
            let handler = settings.connect_notify_local(None, move |_, _| {
                let weak = weak.clone();
                // The view takes the new theme's colours once the change has been handled.
                glib::idle_add_local_once(move || {
                    if let Some(log_window) = weak.upgrade() {
                        log_window.paint();
                    }
                });
            });
            let settings = settings.clone();
            let handler = Cell::new(Some(handler));
            log_window.window.connect_destroy(move |_| {
                if let Some(handler) = handler.take() {
                    settings.disconnect(handler);
                }
            });
        }
        log_window.read();
        log_window.window.present();
        log_window
    }

    /// Gives the styles the colours that suit the theme: the set for a dark theme if it writes
    /// in a light colour.
    fn paint(&self) {
        let text = self.view.color();
        let dark = is_dark_theme((text.red(), text.green(), text.blue()));
        if self.dark.replace(dark) != dark {
            paint_tags(&self.view.buffer(), Colours::of(dark));
        }
    }

    /// Highlights the run `run_id` instead, scrolls to it, and brings the window to the front.
    pub fn show_run(&self, run_id: &str) {
        *self.run_id.borrow_mut() = run_id.to_owned();
        self.highlight();
        self.scroll_to_run();
        self.window.present();
    }

    /// Reads the log after a pause, unless that is planned already.
    fn read_soon(self: &Rc<Self>) {
        if self.reading_soon.replace(true) {
            return;
        }
        let weak: Weak<Self> = Rc::downgrade(self);
        glib::timeout_add_local_once(SETTLE, move || {
            if let Some(log_window) = weak.upgrade() {
                log_window.reading_soon.set(false);
                log_window.read();
            }
        });
    }

    /// Reads the log off the main thread, and shows it.
    fn read(self: &Rc<Self>) {
        let root = self.root.clone();
        let reading = gio::spawn_blocking(move || read_log_lines(&RealFs, &root).ok());
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let lines = reading.await.ok().flatten();
            if let Some(log_window) = weak.upgrade() {
                log_window.show(lines);
            }
        });
    }

    /// Shows `lines`, or that the log couldn't be read, keeping the place unless the run is
    /// still to be scrolled to.
    fn show(&self, lines: Option<Vec<LogLine>>) {
        let buffer = self.view.buffer();
        let vertical = self.scroll.vadjustment().value();
        let horizontal = self.scroll.hadjustment().value();
        buffer.set_text("");
        let mut end = buffer.end_iter();
        match &lines {
            None => buffer.insert(&mut end, Text::LogUnreadable.get(self.language)),
            Some(lines) => {
                for (index, line) in lines.iter().enumerate() {
                    if index > 0 {
                        buffer.insert(&mut end, "\n");
                    }
                    for (text, style) in &line.pieces {
                        match tag_name(*style) {
                            Some(tag) => buffer.insert_with_tags_by_name(&mut end, text, &[tag]),
                            None => buffer.insert(&mut end, text),
                        }
                    }
                }
                buffer.apply_tag_by_name(MONOSPACE, &buffer.start_iter(), &buffer.end_iter());
                for (index, line) in lines.iter().enumerate() {
                    if line.shaded {
                        let (start, end) = line_bounds(&buffer, index);
                        buffer.apply_tag_by_name(SHADED, &start, &end);
                    }
                }
            }
        }
        *self.lines.borrow_mut() = lines.unwrap_or_default();
        self.highlight();
        if self.scroll_to_run.get() {
            self.scroll_to_run();
        } else {
            // The new text is laid out before the place can be set back.
            let scroll = self.scroll.clone();
            glib::timeout_add_local_once(LAID_OUT, move || {
                scroll.vadjustment().set_value(vertical);
                scroll.hadjustment().set_value(horizontal);
            });
        }
    }

    /// Highlights the lines of the chosen run.
    fn highlight(&self) {
        let buffer = self.view.buffer();
        buffer.remove_tag_by_name(HIGHLIGHT, &buffer.start_iter(), &buffer.end_iter());
        let run_id = self.run_id.borrow();
        for (index, line) in self.lines.borrow().iter().enumerate() {
            if line.run_id.as_deref() == Some(run_id.as_str()) {
                let (start, end) = line_bounds(&buffer, index);
                buffer.apply_tag_by_name(HIGHLIGHT, &start, &end);
            }
        }
    }

    /// Scrolls to the chosen run, once the text has been laid out, unless all its lines are in
    /// view already (issue 190).
    fn scroll_to_run(&self) {
        let run_id = self.run_id.borrow().clone();
        let (first, last) = {
            let lines = self.lines.borrow();
            let mut indexes = lines
                .iter()
                .enumerate()
                .filter(|(_, line)| line.run_id.as_deref() == Some(run_id.as_str()))
                .map(|(index, _)| index);
            let Some(first) = indexes.next() else {
                return;
            };
            (first, indexes.next_back().unwrap_or(first))
        };
        self.scroll_to_run.set(false);
        let buffer = self.view.buffer();
        let start = buffer.create_mark(None, &line_bounds(&buffer, first).0, true);
        let end = buffer.create_mark(None, &line_bounds(&buffer, last).0, true);
        let view = self.view.clone();
        let scroll = self.scroll.clone();
        // Only once the text has been laid out do its lines have their places.
        glib::timeout_add_local_once(LAID_OUT, move || {
            let buffer = view.buffer();
            let top = view.iter_location(&buffer.iter_at_mark(&start));
            let bottom = view.iter_location(&buffer.iter_at_mark(&end));
            buffer.delete_mark(&start);
            buffer.delete_mark(&end);
            let top = f64::from(top.y());
            let bottom = f64::from(bottom.y() + bottom.height());
            let vertical = scroll.vadjustment();
            let page = vertical.page_size();
            // The run is scrolled to only if it isn't all in view, so that a short log stays
            // put. Looking at its first line only left a run at the bottom edge with its other
            // lines out of sight (issue 190).
            if top < vertical.value() || bottom > vertical.value() + page {
                let most = (vertical.upper() - page).max(0.0);
                vertical.set_value((top - RUN_POSITION * page).clamp(0.0, most));
            }
            scroll.hadjustment().set_value(0.0);
        });
    }
}

/// The start and the end of the line numbered `index`, including its line ending, so that a
/// background reaches the window's edge.
fn line_bounds(buffer: &TextBuffer, index: usize) -> (gtk4::TextIter, gtk4::TextIter) {
    let line = i32::try_from(index).unwrap_or(i32::MAX);
    let start = buffer
        .iter_at_line(line)
        .unwrap_or_else(|| buffer.end_iter());
    let end = buffer
        .iter_at_line(line.saturating_add(1))
        .unwrap_or_else(|| buffer.end_iter());
    (start, end)
}

fn tag_name(style: Style) -> Option<&'static str> {
    Some(match style {
        Style::Plain => return None,
        Style::Time => "time",
        Style::RunId => "run-id",
        Style::Start => "start",
        Style::Error => "error",
        Style::End(VerdictMark::Good) => "end-good",
        Style::End(VerdictMark::Attention) => "end-attention",
        Style::End(VerdictMark::Failed) => "end-failed",
        Style::Key => "key",
        Style::Value => "value",
        Style::Text => "text",
        Style::Garbled => "garbled",
    })
}

/// The styles, without their colours, which [`paint_tags`] gives them. The run's backgrounds come
/// last, so that the highlight wins over the shading.
fn add_tags(buffer: &TextBuffer) {
    let table = buffer.tag_table();
    let tag = |name: &str| TextTag::builder().name(name);
    let tags = [
        tag(MONOSPACE).family("Monospace").build(),
        tag("time").build(),
        tag("run-id").build(),
        tag("start").weight(700).build(),
        tag("error").weight(700).build(),
        tag("end-good").weight(700).build(),
        tag("end-attention").weight(700).build(),
        tag("end-failed").weight(700).build(),
        tag("key").build(),
        tag("value").build(),
        tag("text").style(gtk4::pango::Style::Italic).build(),
        tag("garbled").style(gtk4::pango::Style::Italic).build(),
        tag(SHADED).build(),
        tag(HIGHLIGHT).build(),
    ];
    for tag in &tags {
        table.add(tag);
    }
    paint_tags(buffer, Colours::of(false));
}

/// Gives the styles the `colours`.
fn paint_tags(buffer: &TextBuffer, colours: Colours) {
    let table = buffer.tag_table();
    let foregrounds = [
        ("time", colours.time),
        ("run-id", colours.run_id),
        ("start", colours.start),
        ("error", colours.error),
        ("end-good", colours.end_good),
        ("end-attention", colours.end_attention),
        ("end-failed", colours.end_failed),
        ("key", colours.key),
        ("text", colours.text),
        ("garbled", colours.garbled),
    ];
    for (name, colour) in foregrounds {
        if let Some(tag) = table.lookup(name) {
            tag.set_foreground(Some(colour));
        }
    }
    for (name, colour) in [(SHADED, colours.shaded), (HIGHLIGHT, colours.highlight)] {
        if let Some(tag) = table.lookup(name) {
            tag.set_paragraph_background(Some(colour));
        }
    }
}
