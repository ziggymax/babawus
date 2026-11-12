//! The log window: a stick's log for the poweruser, one row per line, with one run highlighted.
//! It isn't modal, and follows changes to the log until it is closed. What to show comes from
//! [`babawus_core::log_view`]. See "See a stick's log" in Requirements.md and Issues.md
//! item 135.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::time::Duration;

use babawus_core::fs::RealFs;
use babawus_core::log_view::{LogLine, Style, read_log_lines};
use babawus_core::medium::LOG_FILE;
use babawus_core::settings::Language;
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
        log_window.read();
        log_window.window.present();
        log_window
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

    /// Scrolls to the first line of the chosen run, once the text has been laid out.
    fn scroll_to_run(&self) {
        let run_id = self.run_id.borrow().clone();
        let Some(index) = self
            .lines
            .borrow()
            .iter()
            .position(|line| line.run_id.as_deref() == Some(run_id.as_str()))
        else {
            return;
        };
        self.scroll_to_run.set(false);
        let buffer = self.view.buffer();
        let (start, _) = line_bounds(&buffer, index);
        let mark = buffer.create_mark(None, &start, true);
        let view = self.view.clone();
        let scroll = self.scroll.clone();
        // Only once the text has been laid out do its lines have their places.
        glib::timeout_add_local_once(LAID_OUT, move || {
            let buffer = view.buffer();
            let place = view.iter_location(&buffer.iter_at_mark(&mark));
            buffer.delete_mark(&mark);
            let (top, height) = (f64::from(place.y()), f64::from(place.height()));
            let vertical = scroll.vadjustment();
            let page = vertical.page_size();
            // The run is scrolled to only if it isn't in view, so that a short log stays put.
            if top < vertical.value() || top + height > vertical.value() + page {
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

/// The styles. The run's backgrounds come last, so that the highlight wins over the shading.
fn add_tags(buffer: &TextBuffer) {
    let table = buffer.tag_table();
    let tag = |name: &str| TextTag::builder().name(name);
    let tags = [
        tag(MONOSPACE).family("Monospace").build(),
        tag("time").foreground("#5e5c64").build(),
        tag("run-id").foreground("#77767b").build(),
        tag("start").foreground("#1c71d8").weight(700).build(),
        tag("error").foreground("#c01c28").weight(700).build(),
        tag("end-good").foreground("#26a269").weight(700).build(),
        tag("end-attention")
            .foreground("#b5690b")
            .weight(700)
            .build(),
        tag("end-failed").foreground("#c01c28").weight(700).build(),
        tag("key").foreground("#77767b").build(),
        tag("value").build(),
        tag("text")
            .foreground("#77767b")
            .style(gtk4::pango::Style::Italic)
            .build(),
        tag("garbled")
            .foreground("#9a9996")
            .style(gtk4::pango::Style::Italic)
            .build(),
        tag(SHADED).paragraph_background("#f0f0f0").build(),
        tag(HIGHLIGHT).paragraph_background("#fff1a8").build(),
    ];
    for tag in &tags {
        table.add(tag);
    }
}
