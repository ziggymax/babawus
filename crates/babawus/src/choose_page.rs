//! "Choose what to back up…": a tree of the home folder with a tick box on each row, the name
//! patterns, and live totals, with Done and Cancel. See Issues.md items 82 and 86-89. What each
//! row shows comes from [`babawus_core::choose_screen`]; this module puts it on the screen and
//! keeps the choice until the window takes it back.
//!
//! Folders are read when they are opened, on the main thread: one folder is quick to list. The
//! totals walk everything chosen, so they are counted off the main thread, starting over on
//! each change.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use babawus_core::choose_screen::{
    Choice, Entry, children, home_entry, pattern_problem_text, totals_text,
};
use babawus_core::fs::{FileKind, RealFs};
use babawus_core::settings::Language;
use babawus_core::start_screen::Mark;
use babawus_core::text::Text;
use gtk4::prelude::*;
use gtk4::{
    Align, Box, Button, CheckButton, Entry as TextEntry, Image, Label, ListItem, ListView,
    NoSelection, Orientation, PolicyType, ScrolledWindow, Separator, SignalListItemFactory,
    Spinner, Switch, TreeExpander, TreeListModel, TreeListRow, gio, glib,
};

use crate::icons;

const MARGIN: i32 = 32;
const SPACING: i32 = 12;
const TREE_HEIGHT: i32 = 320;

/// The widgets of one row of the tree. GTK reuses them for whichever row is in view.
struct RowWidgets {
    check: CheckButton,
    name: Label,
    note: Label,
    icon: Image,
    entry: RefCell<Option<Entry>>,
}

/// What the tree's rows need, shared with GTK's callbacks.
struct Shared {
    choice: RefCell<Choice>,
    /// What the choice was when the screen opened, to tell whether it changed.
    original: RefCell<Choice>,
    home: PathBuf,
    show_hidden: Cell<bool>,
    language: Cell<Language>,
    /// Set while rows are filled in, so that their tick boxes' signals are ignored.
    updating: Cell<bool>,
    /// The widgets of each row GTK has made, by its list item.
    rows: RefCell<HashMap<ListItem, Rc<RowWidgets>>>,
    /// Called after every change, so that the page can count again.
    changed: RefCell<Option<std::boxed::Box<dyn Fn()>>>,
}

impl Shared {
    /// Fills in `row` for `entry`.
    fn show_row(&self, row: &RowWidgets, entry: &Entry) {
        let language = self.language.get();
        let shown = self
            .choice
            .borrow()
            .row(&RealFs, &self.home, entry, language);
        self.updating.set(true);
        row.check.set_active(shown.ticked);
        row.check.set_sensitive(shown.can_toggle);
        self.updating.set(false);
        let name = glib::markup_escape_text(&entry.name);
        if shown.crossed {
            row.name.set_markup(&format!("<s>{name}</s>"));
        } else {
            row.name.set_markup(&name);
        }
        row.note
            .set_label(shown.note.as_deref().unwrap_or_default());
        row.note.set_visible(shown.note.is_some());
        let icon = match entry.kind {
            FileKind::Dir => "folder",
            FileKind::File => "text-x-generic",
            FileKind::Other => "emblem-symbolic-link",
        };
        row.icon.set_icon_name(Some(icon));
    }

    /// Shows every row in view again, after the choice changed.
    fn refresh(&self) {
        let rows: Vec<Rc<RowWidgets>> = self.rows.borrow().values().cloned().collect();
        for row in rows {
            let entry = row.entry.borrow().clone();
            if let Some(entry) = entry {
                self.show_row(&row, &entry);
            }
        }
        if let Some(changed) = self.changed.borrow().as_ref() {
            changed();
        }
    }
}

pub struct ChoosePage {
    pub root: Box,
    title: Label,
    intro: Label,
    hidden_label: Label,
    hidden: Switch,
    tree_scroll: ScrolledWindow,
    patterns_heading: Label,
    patterns_detail: Label,
    patterns: Box,
    pattern_entry: TextEntry,
    pattern_add: Button,
    pattern_problem: Label,
    counting: Spinner,
    totals: Label,
    warning: Label,
    warning_row: Box,
    pub done: Button,
    pub cancel: Button,
    shared: Rc<Shared>,
    /// The free space on the medium, if known.
    free: Cell<Option<u64>>,
    /// Stops the count that is going on, if any.
    count_cancel: RefCell<Arc<AtomicBool>>,
    /// Counts the counts, so that an older one's result is dropped.
    count_generation: Cell<u64>,
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

impl ChoosePage {
    pub fn new() -> Rc<Self> {
        let title = label(Some("babawus-title"));
        let intro = label(None);
        let hidden_label = label(None);
        let hidden = Switch::builder().valign(Align::Center).build();
        let hidden_row = Box::new(Orientation::Horizontal, SPACING);
        hidden_row.append(&hidden);
        hidden_row.append(&hidden_label);
        let tree_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .min_content_height(TREE_HEIGHT)
            .vexpand(true)
            .build();
        tree_scroll.add_css_class("frame");

        let counting = Spinner::builder().spinning(true).build();
        let totals = label(Some("babawus-next-step"));
        let totals_row = Box::new(Orientation::Horizontal, 8);
        totals_row.append(&counting);
        totals_row.append(&totals);
        let warning = label(None);
        warning.set_hexpand(true);
        let warning_mark = icons::image(&icons::mark(Mark::Attention), 20);
        warning_mark.set_valign(Align::Start);
        let warning_row = Box::new(Orientation::Horizontal, 8);
        warning_row.append(&warning_mark);
        warning_row.append(&warning);

        let patterns_heading = label(Some("babawus-next-step"));
        let patterns_detail = label(Some("dim-label"));
        let patterns = Box::new(Orientation::Vertical, 4);
        let pattern_entry = TextEntry::builder().width_chars(20).build();
        let pattern_add = Button::new();
        let add_row = Box::new(Orientation::Horizontal, 8);
        add_row.append(&pattern_entry);
        add_row.append(&pattern_add);
        let pattern_problem = label(Some("dim-label"));

        let page = column();
        page.set_margin_top(MARGIN);
        page.set_margin_bottom(SPACING);
        page.set_margin_start(MARGIN);
        page.set_margin_end(MARGIN);
        page.append(&title);
        page.append(&intro);
        page.append(&hidden_row);
        page.append(&tree_scroll);
        page.append(&totals_row);
        page.append(&warning_row);
        page.append(&Separator::new(Orientation::Horizontal));
        page.append(&patterns_heading);
        page.append(&patterns_detail);
        page.append(&patterns);
        page.append(&add_row);
        page.append(&pattern_problem);
        let scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .child(&page)
            .build();

        let done = Button::new();
        done.add_css_class("suggested-action");
        let cancel = Button::new();
        let buttons = Box::new(Orientation::Horizontal, 8);
        buttons.append(&done);
        buttons.append(&cancel);
        let bottom = column();
        bottom.set_margin_bottom(MARGIN);
        bottom.set_margin_start(MARGIN);
        bottom.set_margin_end(MARGIN);
        bottom.append(&Separator::new(Orientation::Horizontal));
        bottom.append(&buttons);
        let root = Box::new(Orientation::Vertical, 0);
        root.append(&scroll);
        root.append(&bottom);

        let page = Rc::new(Self {
            root,
            title,
            intro,
            hidden_label,
            hidden,
            tree_scroll,
            patterns_heading,
            patterns_detail,
            patterns,
            pattern_entry,
            pattern_add,
            pattern_problem,
            counting,
            totals,
            warning,
            warning_row,
            done,
            cancel,
            shared: Rc::new(Shared {
                choice: RefCell::new(Choice::default()),
                original: RefCell::new(Choice::default()),
                home: glib::home_dir(),
                show_hidden: Cell::new(false),
                language: Cell::new(Language::English),
                updating: Cell::new(false),
                rows: RefCell::new(HashMap::new()),
                changed: RefCell::new(None),
            }),
            free: Cell::new(None),
            count_cancel: RefCell::new(Arc::new(AtomicBool::new(false))),
            count_generation: Cell::new(0),
        });
        page.connect_signals();
        page
    }

    fn connect_signals(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        *self.shared.changed.borrow_mut() = Some(std::boxed::Box::new(move || {
            if let Some(page) = weak.upgrade() {
                page.show_patterns();
                page.recount();
            }
        }));
        let weak = Rc::downgrade(self);
        self.hidden.connect_active_notify(move |switch| {
            let Some(page) = weak.upgrade() else { return };
            page.shared.show_hidden.set(switch.is_active());
            page.build_tree();
        });
        let weak = Rc::downgrade(self);
        let add = move || {
            let Some(page) = weak.upgrade() else { return };
            let text = page.pattern_entry.text();
            let added = page.shared.choice.borrow_mut().add_pattern(&text);
            match added {
                Ok(()) => {
                    page.pattern_entry.set_text("");
                    page.pattern_problem.set_visible(false);
                    page.shared.refresh();
                }
                Err(problem) => {
                    let language = page.shared.language.get();
                    page.pattern_problem
                        .set_label(pattern_problem_text(problem, language));
                    page.pattern_problem.set_visible(true);
                }
            }
        };
        let add = Rc::new(add);
        let on_click = add.clone();
        self.pattern_add.connect_clicked(move |_| on_click());
        self.pattern_entry.connect_activate(move |_| add());
    }

    /// Opens the screen for `user`, with what they back up now, and `free` bytes on the medium.
    pub fn open(
        self: &Rc<Self>,
        user: &str,
        choice: Choice,
        free: Option<u64>,
        language: Language,
    ) {
        *self.shared.choice.borrow_mut() = choice.clone();
        *self.shared.original.borrow_mut() = choice;
        self.free.set(free);
        self.shared.show_hidden.set(false);
        self.hidden.set_active(false);
        self.pattern_entry.set_text("");
        self.pattern_problem.set_visible(false);
        self.title
            .set_label(&Text::ChooseHeading.fill(language, &[("user", user)]));
        self.set_language(language);
    }

    /// Sets every text again, for example after the language changed.
    pub fn set_language(self: &Rc<Self>, language: Language) {
        self.shared.language.set(language);
        self.intro.set_label(Text::ChooseIntro.get(language));
        self.hidden_label
            .set_label(Text::ChooseShowHidden.get(language));
        self.patterns_heading
            .set_label(Text::ChoosePatterns.get(language));
        self.patterns_detail
            .set_label(Text::ChoosePatternsDetail.get(language));
        self.pattern_add
            .set_label(Text::ChoosePatternAdd.get(language));
        self.done.set_label(Text::ChooseDone.get(language));
        self.done
            .set_tooltip_text(Some(Text::ChooseDoneTooltip.get(language)));
        self.cancel.set_label(Text::Cancel.get(language));
        self.cancel
            .set_tooltip_text(Some(Text::ChooseCancelTooltip.get(language)));
        self.build_tree();
        self.show_patterns();
        self.recount();
    }

    /// The choice as it stands.
    pub fn choice(&self) -> Choice {
        self.shared.choice.borrow().clone()
    }

    /// Whether the choice differs from what it was when the screen opened.
    pub fn changed(&self) -> bool {
        *self.shared.choice.borrow() != *self.shared.original.borrow()
    }

    /// Stops counting, when the screen is left.
    pub fn close(&self) {
        self.count_cancel.borrow().store(true, Ordering::Relaxed);
        self.tree_scroll.set_child(None::<&gtk4::Widget>);
        self.shared.rows.borrow_mut().clear();
    }

    /// Builds the tree again from the home folder, with the home folder open.
    fn build_tree(&self) {
        let root = gio::ListStore::new::<glib::BoxedAnyObject>();
        root.append(&glib::BoxedAnyObject::new(home_entry(
            self.shared.language.get(),
        )));
        let shared = Rc::downgrade(&self.shared);
        let tree = TreeListModel::new(root, false, false, move |item| {
            let shared = shared.upgrade()?;
            let entry = item
                .downcast_ref::<glib::BoxedAnyObject>()?
                .borrow::<Entry>()
                .clone();
            if entry.kind != FileKind::Dir {
                return None;
            }
            let store = gio::ListStore::new::<glib::BoxedAnyObject>();
            let listed = children(
                &RealFs,
                &shared.home,
                &entry.relative,
                shared.show_hidden.get(),
                &shared.choice.borrow(),
            );
            // A folder that can't be read shows as empty when opened.
            for child in listed.unwrap_or_default() {
                store.append(&glib::BoxedAnyObject::new(child));
            }
            Some(store.upcast())
        });
        let factory = SignalListItemFactory::new();
        let shared = self.shared.clone();
        factory.connect_setup(move |_, item| {
            let Some(item) = item.downcast_ref::<ListItem>() else {
                return;
            };
            let check = CheckButton::new();
            let icon = Image::new();
            icon.set_pixel_size(20);
            let name = Label::builder().xalign(0.0).build();
            let note = Label::builder().xalign(0.0).hexpand(true).build();
            note.add_css_class("dim-label");
            let content = Box::new(Orientation::Horizontal, 8);
            content.append(&check);
            content.append(&icon);
            content.append(&name);
            content.append(&note);
            let expander = TreeExpander::new();
            expander.set_child(Some(&content));
            item.set_child(Some(&expander));
            let row = Rc::new(RowWidgets {
                check: check.clone(),
                name,
                note,
                icon,
                entry: RefCell::new(None),
            });
            shared.rows.borrow_mut().insert(item.clone(), row.clone());
            let weak = Rc::downgrade(&shared);
            let row_for_toggle = row.clone();
            check.connect_toggled(move |_| {
                let Some(shared) = weak.upgrade() else { return };
                if shared.updating.get() {
                    return;
                }
                let entry = row_for_toggle.entry.borrow().clone();
                if let Some(entry) = entry {
                    shared
                        .choice
                        .borrow_mut()
                        .toggle(&RealFs, &shared.home, &entry);
                }
                shared.refresh();
            });
        });
        let shared = self.shared.clone();
        factory.connect_teardown(move |_, item| {
            if let Some(item) = item.downcast_ref::<ListItem>() {
                shared.rows.borrow_mut().remove(item);
            }
        });
        let shared = self.shared.clone();
        factory.connect_bind(move |_, item| {
            let Some(item) = item.downcast_ref::<ListItem>() else {
                return;
            };
            let Some(list_row) = item.item().and_downcast::<TreeListRow>() else {
                return;
            };
            let Some(expander) = item.child().and_downcast::<TreeExpander>() else {
                return;
            };
            expander.set_list_row(Some(&list_row));
            let Some(object) = list_row.item().and_downcast::<glib::BoxedAnyObject>() else {
                return;
            };
            let entry = object.borrow::<Entry>().clone();
            let Some(row) = shared.rows.borrow().get(item).cloned() else {
                return;
            };
            shared.show_row(&row, &entry);
            *row.entry.borrow_mut() = Some(entry);
        });
        let view = ListView::new(Some(NoSelection::new(Some(tree.clone()))), Some(factory));
        self.tree_scroll.set_child(Some(&view));
        if let Some(home) = tree.item(0).and_downcast::<TreeListRow>() {
            home.set_expanded(true);
        }
    }

    /// Lists the name patterns, each with "Remove".
    fn show_patterns(&self) {
        let language = self.shared.language.get();
        while let Some(child) = self.patterns.first_child() {
            self.patterns.remove(&child);
        }
        let patterns = self.shared.choice.borrow().exclude_patterns.clone();
        for pattern in patterns {
            let text = label(None);
            text.set_label(&pattern);
            text.set_hexpand(true);
            let remove = Button::with_label(Text::ChoosePatternRemove.get(language));
            let weak = Rc::downgrade(&self.shared);
            remove.connect_clicked(move |_| {
                let Some(shared) = weak.upgrade() else { return };
                shared.choice.borrow_mut().remove_pattern(&pattern);
                shared.refresh();
            });
            let row = Box::new(Orientation::Horizontal, SPACING);
            row.set_margin_start(SPACING);
            row.append(&text);
            row.append(&remove);
            self.patterns.append(&row);
        }
    }

    /// Counts the totals again off the main thread, stopping a count that is going on.
    fn recount(self: &Rc<Self>) {
        self.count_cancel.borrow().store(true, Ordering::Relaxed);
        let cancel = Arc::new(AtomicBool::new(false));
        *self.count_cancel.borrow_mut() = cancel.clone();
        let generation = self.count_generation.get() + 1;
        self.count_generation.set(generation);
        self.show_totals(None);
        let choice = self.choice();
        let home = self.shared.home.clone();
        let counting = gio::spawn_blocking(move || choice.count(&RealFs, &home, &cancel));
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            // Cancelled, or failed by panicking, which has been reported already.
            let Ok(Some(counted)) = counting.await else {
                return;
            };
            if let Some(page) = weak.upgrade()
                && page.count_generation.get() == generation
            {
                page.show_totals(Some(counted));
            }
        });
    }

    fn show_totals(&self, totals: Option<babawus_core::choose_screen::Totals>) {
        let language = self.shared.language.get();
        let (text, too_big) = totals_text(totals, self.free.get(), language);
        self.counting.set_visible(totals.is_none());
        self.totals.set_label(&text);
        self.warning
            .set_label(too_big.as_deref().unwrap_or_default());
        self.warning_row.set_visible(too_big.is_some());
    }
}
