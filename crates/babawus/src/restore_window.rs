//! The Restore window: the user's backup on a stick as a tree with tick boxes, a side pane with
//! the selected file's versions, where the files go, and "Restore". It isn't modal, there is at
//! most one per stick, and it closes when its stick goes. See "Restore files" in
//! Requirements.md and issue 161. The texts come from [`babawus_core::restore_screen`], and the
//! restoring itself is [`babawus_core::restore`].
//!
//! Folders are read when they are opened, on the main thread: one folder is quick to list. The
//! plan and the copying run off the main thread.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use babawus_core::fs::{FileKind, RealFs};
use babawus_core::language::Language;
use babawus_core::restore::{
    Backup, Choice, Copy, Item, Outcome, Plan, Progress, Resolution, Version,
};
use babawus_core::restore_screen::{progress_text, question, result_text, row_note, version_text};
use babawus_core::text::Text;
use babawus_core::time::local_clock;
use babawus_core::versions::Format;
use gtk4::prelude::*;
use gtk4::{
    AlertDialog, Align, Box, Button, CheckButton, FileDialog, Image, Label, ListBox, ListItem,
    ListView, Orientation, Paned, PolicyType, ProgressBar, ScrolledWindow, SelectionMode,
    Separator, SignalListItemFactory, SingleSelection, TreeExpander, TreeListModel, TreeListRow,
    gio, glib,
};

const WIDTH: i32 = 1000;
const HEIGHT: i32 = 700;
const MARGIN: i32 = 24;
const SPACING: i32 = 12;
/// Where the divider between the tree and the versions starts.
const TREE_WIDTH: i32 = 600;
/// How often the progress is shown while files are restored.
const PROGRESS_UPDATES: Duration = Duration::from_millis(200);
/// How often the window checks whether a backup to its stick has started or ended.
const BUSY_CHECKS: Duration = Duration::from_secs(1);

/// The widgets of one row of the tree. GTK reuses them for whichever row is in view.
struct RowWidgets {
    check: CheckButton,
    name: Label,
    note: Label,
    icon: Image,
    item: RefCell<Option<Item>>,
}

/// What the window's parts need, shared with GTK's callbacks.
struct Shared {
    backup: Backup<'static>,
    choice: RefCell<Choice>,
    language: Cell<Language>,
    /// Set while rows are filled in, so that their tick boxes' signals are ignored.
    updating: Cell<bool>,
    rows: RefCell<HashMap<ListItem, Rc<RowWidgets>>>,
    /// The file whose versions are shown, and its versions.
    selected: RefCell<Option<(Item, Vec<Version>)>>,
    /// Called after every change to the choice.
    changed: RefCell<Option<std::boxed::Box<dyn Fn()>>>,
}

impl Shared {
    /// The time of the older version picked for the file `relative`, if any.
    fn picked_time(&self, relative: &Path) -> Option<SystemTime> {
        let Copy::Older(entry) = self.choice.borrow().copy_of(relative) else {
            return None;
        };
        let versions = self.backup.versions(relative).ok()?;
        versions
            .into_iter()
            .find(|version| version.copy == Copy::Older(entry.clone()))
            .map(|version| version.time)
    }

    fn show_row(&self, row: &RowWidgets, item: &Item) {
        let language = self.language.get();
        let is_dir = item.kind == FileKind::Dir;
        let shown = self.choice.borrow().row(&item.relative, is_dir);
        let picked = if is_dir {
            None
        } else {
            self.picked_time(&item.relative).map(local_clock)
        };
        self.updating.set(true);
        row.check.set_active(shown.ticked && !shown.partly);
        row.check.set_inconsistent(shown.partly);
        row.check.set_sensitive(shown.can_toggle);
        self.updating.set(false);
        let name = glib::markup_escape_text(&item.name);
        if shown.crossed {
            row.name.set_markup(&format!("<s>{name}</s>"));
        } else {
            row.name.set_markup(&name);
        }
        let note = row_note(item, &shown, picked, language);
        row.note.set_label(note.as_deref().unwrap_or_default());
        row.note.set_visible(note.is_some());
        row.icon
            .set_icon_name(Some(if is_dir { "folder" } else { "text-x-generic" }));
    }

    fn refresh(&self) {
        let rows: Vec<Rc<RowWidgets>> = self.rows.borrow().values().cloned().collect();
        for row in rows {
            let item = row.item.borrow().clone();
            if let Some(item) = item {
                self.show_row(&row, &item);
            }
        }
        if let Some(changed) = self.changed.borrow().as_ref() {
            changed();
        }
    }
}

/// Where the files go.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Destination {
    BackInPlace,
    Folder(PathBuf),
}

pub struct RestoreWindow {
    pub window: gtk4::Window,
    label: String,
    user: String,
    heading: Label,
    intro: Label,
    tree_scroll: ScrolledWindow,
    tree: RefCell<Option<TreeListModel>>,
    versions_heading: Label,
    versions_hint: Label,
    versions: ListBox,
    /// Never shown: the versions are grouped with it, so that a file with only one version gets
    /// a round radio button too, not a square checkbox (T-102).
    version_group: CheckButton,
    where_heading: Label,
    back_in_place: CheckButton,
    into_folder: CheckButton,
    folder: Label,
    choose: Button,
    progress_row: Box,
    progress_text: Label,
    progress: ProgressBar,
    cancel: Button,
    result: Label,
    details: Label,
    open_folder: Button,
    restore: Button,
    shared: Rc<Shared>,
    destination: RefCell<Destination>,
    /// The suggested folder, until the user chooses another.
    suggested: RefCell<PathBuf>,
    /// The folder the last restore went to, for "Open folder".
    restored_to: RefCell<Option<PathBuf>>,
    /// Stops the restore going on, if any.
    running: RefCell<Option<Arc<AtomicBool>>>,
    /// Whether a backup to this stick is running, asked of the main window.
    backup_running: std::boxed::Box<dyn Fn() -> bool>,
    /// Tells the main window whether a restore from this stick is running.
    on_busy: std::boxed::Box<dyn Fn(bool)>,
}

fn label(class: Option<&str>) -> Label {
    let label = Label::builder().xalign(0.0).wrap(true).build();
    if let Some(class) = class {
        label.add_css_class(class);
    }
    label
}

impl RestoreWindow {
    /// Opens a Restore window for the stick labelled `label`, whose backup for `user` is in the
    /// folder `root`. `backup_running` says whether a backup to the stick is running, and
    /// `on_busy` is told when a restore starts and ends.
    #[allow(clippy::too_many_arguments)]
    pub fn open(
        app: &gtk4::Application,
        label_text: &str,
        user: &str,
        root: PathBuf,
        format: Format,
        language: Language,
        backup_running: std::boxed::Box<dyn Fn() -> bool>,
        on_busy: std::boxed::Box<dyn Fn(bool)>,
    ) -> Rc<Self> {
        let heading = label(Some("babawus-title"));
        let intro = label(None);
        let tree_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .build();
        tree_scroll.add_css_class("frame");
        let versions_heading = label(Some("babawus-next-step"));
        let versions_hint = label(Some("dim-label"));
        let versions = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .build();
        let versions_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vexpand(true)
            .child(&versions)
            .build();
        let side = Box::new(Orientation::Vertical, SPACING);
        side.set_margin_start(SPACING);
        side.append(&versions_heading);
        side.append(&versions_hint);
        side.append(&versions_scroll);
        let paned = Paned::builder()
            .orientation(Orientation::Horizontal)
            .start_child(&tree_scroll)
            .end_child(&side)
            .position(TREE_WIDTH)
            .shrink_start_child(false)
            .shrink_end_child(false)
            .vexpand(true)
            .build();

        let where_heading = label(Some("babawus-next-step"));
        let back_in_place = CheckButton::new();
        let into_folder = CheckButton::new();
        into_folder.set_group(Some(&back_in_place));
        back_in_place.set_active(true);
        let folder = Label::builder()
            .xalign(0.0)
            .ellipsize(gtk4::pango::EllipsizeMode::Middle)
            .hexpand(true)
            .build();
        let choose = Button::new();
        let folder_row = Box::new(Orientation::Horizontal, 8);
        folder_row.append(&into_folder);
        folder_row.append(&folder);
        folder_row.append(&choose);

        let progress_text = Label::builder().xalign(0.0).build();
        let progress = ProgressBar::builder()
            .hexpand(true)
            .valign(Align::Center)
            .build();
        let cancel = Button::new();
        let progress_row = Box::new(Orientation::Horizontal, SPACING);
        progress_row.append(&progress_text);
        progress_row.append(&progress);
        progress_row.append(&cancel);
        progress_row.set_visible(false);
        let result = label(Some("babawus-next-step"));
        result.set_visible(false);
        let details = label(None);
        details.set_selectable(true);
        details.set_visible(false);
        let details_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(120)
            .child(&details)
            .build();

        let open_folder = Button::new();
        open_folder.set_visible(false);
        let restore = Button::new();
        restore.add_css_class("suggested-action");
        let buttons = Box::new(Orientation::Horizontal, 8);
        buttons.append(&restore);
        buttons.append(&open_folder);

        let page = Box::new(Orientation::Vertical, SPACING);
        page.set_margin_top(MARGIN);
        page.set_margin_bottom(MARGIN);
        page.set_margin_start(MARGIN);
        page.set_margin_end(MARGIN);
        page.append(&heading);
        page.append(&intro);
        page.append(&paned);
        page.append(&Separator::new(Orientation::Horizontal));
        page.append(&where_heading);
        page.append(&back_in_place);
        page.append(&folder_row);
        page.append(&progress_row);
        page.append(&result);
        page.append(&details_scroll);
        page.append(&buttons);

        let window = gtk4::Window::builder()
            .application(app)
            .default_width(WIDTH)
            .default_height(HEIGHT)
            .child(&page)
            .build();
        let backup = Backup {
            medium: &RealFs,
            root,
            format,
            system: &RealFs,
            home: glib::home_dir(),
        };
        let restore_window = Rc::new(Self {
            window,
            label: label_text.to_owned(),
            user: user.to_owned(),
            heading,
            intro,
            tree_scroll,
            tree: RefCell::new(None),
            versions_heading,
            versions_hint,
            versions,
            version_group: CheckButton::new(),
            where_heading,
            back_in_place,
            into_folder,
            folder,
            choose,
            progress_row,
            progress_text,
            progress,
            cancel,
            result,
            details,
            open_folder,
            restore,
            shared: Rc::new(Shared {
                backup,
                choice: RefCell::new(Choice::default()),
                language: Cell::new(language),
                updating: Cell::new(false),
                rows: RefCell::new(HashMap::new()),
                selected: RefCell::new(None),
                changed: RefCell::new(None),
            }),
            destination: RefCell::new(Destination::BackInPlace),
            suggested: RefCell::new(PathBuf::new()),
            restored_to: RefCell::new(None),
            running: RefCell::new(None),
            backup_running,
            on_busy,
        });
        restore_window.suggest_folder();
        restore_window.connect_signals();
        restore_window.set_language(language);
        restore_window.window.present();
        restore_window
    }

    fn connect_signals(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        *self.shared.changed.borrow_mut() = Some(std::boxed::Box::new(move || {
            if let Some(window) = weak.upgrade() {
                window.update_restore_button();
                window.show_versions();
            }
        }));
        let weak = Rc::downgrade(self);
        self.back_in_place.connect_toggled(move |button| {
            if let Some(window) = weak.upgrade()
                && button.is_active()
            {
                *window.destination.borrow_mut() = Destination::BackInPlace;
            }
        });
        let weak = Rc::downgrade(self);
        self.into_folder.connect_toggled(move |button| {
            if let Some(window) = weak.upgrade()
                && button.is_active()
            {
                let folder = PathBuf::from(window.folder.label().as_str());
                *window.destination.borrow_mut() = Destination::Folder(folder);
            }
        });
        let weak = Rc::downgrade(self);
        self.choose.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.choose_folder();
            }
        });
        let weak = Rc::downgrade(self);
        self.restore.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade() {
                window.start();
            }
        });
        let weak = Rc::downgrade(self);
        self.cancel.connect_clicked(move |_| {
            if let Some(window) = weak.upgrade()
                && let Some(cancel) = window.running.borrow().as_ref()
            {
                cancel.store(true, Ordering::Relaxed);
            }
        });
        let weak = Rc::downgrade(self);
        self.open_folder.connect_clicked(move |_| {
            let Some(window) = weak.upgrade() else { return };
            if let Some(folder) = window.restored_to.borrow().as_ref() {
                let uri = gio::File::for_path(folder).uri();
                // The file manager not opening changes nothing about the restore.
                let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
            }
        });
        let weak = Rc::downgrade(self);
        self.window.connect_close_request(move |_| {
            let Some(window) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            if window.running.borrow().is_none() {
                return glib::Propagation::Proceed;
            }
            window.ask_to_stop();
            glib::Propagation::Stop
        });
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(BUSY_CHECKS, move || {
            let Some(window) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            window.update_restore_button();
            glib::ControlFlow::Continue
        });
    }

    /// Sets every text again, for example after the language changed.
    pub fn set_language(self: &Rc<Self>, language: Language) {
        self.shared.language.set(language);
        self.window.set_title(Some(
            &Text::RestoreTitle.fill(language, &[("label", &self.label)]),
        ));
        self.heading
            .set_label(&Text::RestoreHeading.fill(language, &[("user", &self.user)]));
        self.intro.set_label(Text::RestoreIntro.get(language));
        self.where_heading
            .set_label(Text::RestoreWhereHeading.get(language));
        self.back_in_place
            .set_label(Some(Text::RestoreBackInPlace.get(language)));
        self.back_in_place
            .set_tooltip_text(Some(Text::RestoreBackInPlaceTooltip.get(language)));
        self.into_folder
            .set_label(Some(Text::RestoreIntoFolder.get(language)));
        self.into_folder
            .set_tooltip_text(Some(Text::RestoreIntoFolderTooltip.get(language)));
        self.choose.set_label(Text::RestoreChoose.get(language));
        self.choose
            .set_tooltip_text(Some(Text::RestoreChooseTooltip.get(language)));
        self.cancel.set_label(Text::Cancel.get(language));
        self.cancel
            .set_tooltip_text(Some(Text::RestoreCancelTooltip.get(language)));
        self.open_folder
            .set_label(Text::RestoreOpenFolder.get(language));
        self.open_folder
            .set_tooltip_text(Some(Text::RestoreOpenFolderTooltip.get(language)));
        self.restore.set_label(Text::RestoreButton.get(language));
        self.update_restore_button();
        self.build_tree();
        self.show_versions();
    }

    /// Suggests a new folder in the home folder, named after the stick and the time.
    fn suggest_folder(&self) {
        let language = self.shared.language.get();
        let name = Text::RestoreSuggested.fill(language, &[("label", &self.label)]);
        let folder = self
            .shared
            .backup
            .suggested_folder(&name, local_clock(SystemTime::now()));
        self.folder.set_label(&folder.to_string_lossy());
        self.folder
            .set_tooltip_text(Some(&folder.to_string_lossy()));
        *self.suggested.borrow_mut() = folder.clone();
        if self.into_folder.is_active() {
            *self.destination.borrow_mut() = Destination::Folder(folder);
        }
    }

    /// Lets the user pick or make another folder, in the system's folder dialog.
    fn choose_folder(self: &Rc<Self>) {
        let language = self.shared.language.get();
        let dialog = FileDialog::builder()
            .title(Text::RestoreChooseTitle.get(language))
            .modal(true)
            .initial_folder(&gio::File::for_path(&self.shared.backup.home))
            .build();
        let weak = Rc::downgrade(self);
        dialog.select_folder(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |chosen| {
                let Some(window) = weak.upgrade() else { return };
                let Some(path) = chosen.ok().and_then(|file| file.path()) else {
                    return;
                };
                if !window.shared.backup.may_restore_to(&path) {
                    window.show_message(Text::RestoreOnStick.get(window.shared.language.get()));
                    return;
                }
                window.folder.set_label(&path.to_string_lossy());
                window
                    .folder
                    .set_tooltip_text(Some(&path.to_string_lossy()));
                *window.destination.borrow_mut() = Destination::Folder(path);
                window.into_folder.set_active(true);
            },
        );
    }

    /// Shows `text` below the tree, in place of a result.
    fn show_message(&self, text: &str) {
        self.result.set_label(text);
        self.result.set_visible(true);
        self.details.set_visible(false);
        self.open_folder.set_visible(false);
    }

    /// Greys out "Restore" while nothing is ticked, a backup to the stick runs, or a restore
    /// runs, and says why in its tooltip.
    fn update_restore_button(&self) {
        let language = self.shared.language.get();
        let busy = self.running.borrow().is_some();
        let backup = (self.backup_running)();
        let empty = self.shared.choice.borrow().is_empty();
        self.restore.set_sensitive(!busy && !backup && !empty);
        let tooltip = if backup {
            Text::RestoreBackupRunning
        } else if empty {
            Text::RestoreNothingTicked
        } else {
            Text::RestoreButtonTooltip
        };
        self.restore.set_tooltip_text(Some(tooltip.get(language)));
    }

    /// Builds the tree again from the top of the backup, with the home folder open.
    fn build_tree(&self) {
        let language = self.shared.language.get();
        let root = gio::ListStore::new::<glib::BoxedAnyObject>();
        root.append(&glib::BoxedAnyObject::new(Item {
            name: Text::CoverageHome.get(language).to_owned(),
            relative: PathBuf::new(),
            kind: FileKind::Dir,
            on_computer: true,
            only_older: false,
        }));
        let shared = Rc::downgrade(&self.shared);
        let tree = TreeListModel::new(root, false, false, move |object| {
            let shared = shared.upgrade()?;
            let item = object
                .downcast_ref::<glib::BoxedAnyObject>()?
                .borrow::<Item>()
                .clone();
            if item.kind != FileKind::Dir {
                return None;
            }
            let store = gio::ListStore::new::<glib::BoxedAnyObject>();
            // A folder that can't be read shows as empty when opened.
            for child in shared.backup.children(&item.relative).unwrap_or_default() {
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
                item: RefCell::new(None),
            });
            shared.rows.borrow_mut().insert(item.clone(), row.clone());
            let weak = Rc::downgrade(&shared);
            let row_for_toggle = row.clone();
            check.connect_toggled(move |_| {
                let Some(shared) = weak.upgrade() else { return };
                if shared.updating.get() {
                    return;
                }
                let item = row_for_toggle.item.borrow().clone();
                if let Some(item) = item {
                    shared
                        .choice
                        .borrow_mut()
                        .toggle(&item.relative, item.kind == FileKind::Dir);
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
            let entry = object.borrow::<Item>().clone();
            let Some(row) = shared.rows.borrow().get(item).cloned() else {
                return;
            };
            shared.show_row(&row, &entry);
            *row.item.borrow_mut() = Some(entry);
        });
        let selection = SingleSelection::builder()
            .model(&tree)
            .autoselect(false)
            .can_unselect(true)
            .build();
        let weak = Rc::downgrade(&self.shared);
        let versions_changed = self.shared.clone();
        selection.connect_selected_item_notify(move |selection| {
            let Some(shared) = weak.upgrade() else { return };
            let item = selection
                .selected_item()
                .and_downcast::<TreeListRow>()
                .and_then(|row| row.item().and_downcast::<glib::BoxedAnyObject>())
                .map(|object| object.borrow::<Item>().clone())
                .filter(|item| item.kind == FileKind::File);
            *shared.selected.borrow_mut() = item.map(|item| {
                let versions = shared.backup.versions(&item.relative).unwrap_or_default();
                (item, versions)
            });
            if let Some(changed) = versions_changed.changed.borrow().as_ref() {
                changed();
            }
        });
        // Nothing is selected at first, so the side pane asks for a file.
        selection.set_selected(gtk4::INVALID_LIST_POSITION);
        let open = self.open_folders();
        let view = ListView::new(Some(selection), Some(factory));
        self.tree_scroll.set_child(Some(&view));
        // Folders that were open stay open, after a restore or a change of language. Opening a
        // folder adds its rows right after it, so one pass down the list reaches folders inside
        // open ones too.
        let mut index = 0;
        while index < tree.n_items() {
            if let Some(row) = tree.item(index).and_downcast::<TreeListRow>()
                && (index == 0 || row_path(&row).is_some_and(|path| open.contains(&path)))
            {
                row.set_expanded(true);
            }
            index += 1;
        }
        *self.tree.borrow_mut() = Some(tree);
        *self.shared.selected.borrow_mut() = None;
    }

    /// The folders open in the tree as shown.
    fn open_folders(&self) -> Vec<PathBuf> {
        let tree = self.tree.borrow();
        let Some(tree) = tree.as_ref() else {
            return Vec::new();
        };
        (0..tree.n_items())
            .filter_map(|index| tree.item(index).and_downcast::<TreeListRow>())
            .filter(TreeListRow::is_expanded)
            .filter_map(|row| row_path(&row))
            .collect()
    }

    /// Shows the selected file's versions in the side pane, with the one that is picked chosen.
    fn show_versions(&self) {
        let language = self.shared.language.get();
        while let Some(child) = self.versions.first_child() {
            self.versions.remove(&child);
        }
        let selected = self.shared.selected.borrow().clone();
        let Some((item, versions)) = selected else {
            self.versions_heading
                .set_label(Text::RestoreVersionsHeading.get(language));
            self.versions_hint
                .set_label(Text::RestoreVersionsHint.get(language));
            self.versions_hint.set_visible(true);
            return;
        };
        self.versions_heading
            .set_label(&Text::RestoreVersionsOf.fill(language, &[("name", &item.name)]));
        self.versions_hint.set_visible(false);
        let choice = self.shared.choice.borrow().clone();
        let row = choice.row(&item.relative, false);
        let wanted = choice.copy_of(&item.relative);
        // Without a newest copy, the newest older version is what a tick restores.
        let current = versions
            .iter()
            .find(|version| version.copy == wanted)
            .or(versions.first())
            .map(|version| version.copy.clone());
        for version in versions {
            let button = CheckButton::with_label(&version_text(&version, local_clock, language));
            button.set_tooltip_text(Some(Text::RestoreVersionTooltip.get(language)));
            button.set_sensitive(!row.left_out_above);
            button.set_group(Some(&self.version_group));
            button.set_active(row.ticked && current.as_ref() == Some(&version.copy));
            let weak = Rc::downgrade(&self.shared);
            let relative = item.relative.clone();
            let copy = version.copy.clone();
            button.connect_toggled(move |button| {
                let Some(shared) = weak.upgrade() else { return };
                if !button.is_active() || shared.updating.get() {
                    return;
                }
                shared.choice.borrow_mut().pick(&relative, copy.clone());
                // Showing the versions again happens after this signal, not inside it.
                let weak = Rc::downgrade(&shared);
                glib::idle_add_local_once(move || {
                    if let Some(shared) = weak.upgrade() {
                        shared.refresh();
                    }
                });
            });
            self.versions.append(&button);
        }
    }

    /// Works out what to restore, asks about files already there, and restores.
    fn start(self: &Rc<Self>) {
        if self.running.borrow().is_some() || (self.backup_running)() {
            return;
        }
        let target = match self.destination.borrow().clone() {
            Destination::BackInPlace => self.shared.backup.home.clone(),
            Destination::Folder(folder) => folder,
        };
        let language = self.shared.language.get();
        if !self.shared.backup.may_restore_to(&target) {
            self.show_message(Text::RestoreOnStick.get(language));
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        self.set_busy(Some(cancel.clone()));
        self.progress_text
            .set_label(Text::RestoreFinding.get(language));
        self.progress.set_fraction(0.0);
        let choice = self.shared.choice.borrow().clone();
        let root = self.shared.backup.root.clone();
        let format = self.shared.backup.format;
        let planning =
            gio::spawn_blocking(move || backup_at(root, format).plan(&choice, &target, &cancel));
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let plan = planning.await.ok().flatten();
            let Some(window) = weak.upgrade() else { return };
            match plan {
                Some(plan) if !window.cancelled() => window.ask_and_restore(plan),
                _ => window.finished(None, 0),
            }
        });
    }

    fn cancelled(&self) -> bool {
        self.running
            .borrow()
            .as_ref()
            .is_some_and(|cancel| cancel.load(Ordering::Relaxed))
    }

    /// Asks once about the files already there, if any, and then restores.
    fn ask_and_restore(self: &Rc<Self>, plan: Plan) {
        let already = plan.already_there();
        if already == 0 {
            self.run(plan, Resolution::Skip);
            return;
        }
        let language = self.shared.language.get();
        let (heading, detail) = question(already, language);
        let dialog = AlertDialog::builder()
            .message(heading)
            .detail(detail)
            .buttons([
                Text::RestoreReplace.get(language),
                Text::RestoreKeepBoth.get(language),
                Text::RestoreSkip.get(language),
                Text::Cancel.get(language),
            ])
            .cancel_button(3)
            .default_button(1)
            .modal(true)
            .build();
        let weak = Rc::downgrade(self);
        dialog.choose(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |answer| {
                let Some(window) = weak.upgrade() else { return };
                let resolution = match answer {
                    Ok(0) => Resolution::Replace,
                    Ok(1) => Resolution::KeepBoth,
                    Ok(2) => Resolution::Skip,
                    _ => {
                        window.set_busy(None);
                        return;
                    }
                };
                window.run(plan, resolution);
            },
        );
    }

    /// Restores `plan` off the main thread, showing the progress until it has finished.
    fn run(self: &Rc<Self>, plan: Plan, resolution: Resolution) {
        let Some(cancel) = self.running.borrow().clone() else {
            return;
        };
        let total = plan.files.len();
        let bytes = plan.bytes().max(1);
        let progress = Arc::new(Progress::default());
        let target = plan.target.clone();
        let root = self.shared.backup.root.clone();
        let format = self.shared.backup.format;
        let counted = progress.clone();
        let restoring = gio::spawn_blocking(move || {
            backup_at(root, format).restore(&plan, resolution, &local_clock, &counted, &cancel)
        });
        let language = self.shared.language.get();
        self.progress_text
            .set_label(&progress_text(0, total, language));
        let weak = Rc::downgrade(self);
        let shown = progress.clone();
        glib::timeout_add_local(PROGRESS_UPDATES, move || {
            let Some(window) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            if window.running.borrow().is_none() {
                return glib::ControlFlow::Break;
            }
            let done = shown.files.load(Ordering::Relaxed);
            let copied = shown.bytes.load(Ordering::Relaxed);
            window.progress_text.set_label(&progress_text(
                done,
                total,
                window.shared.language.get(),
            ));
            window
                .progress
                .set_fraction((copied as f64 / bytes as f64).min(1.0));
            glib::ControlFlow::Continue
        });
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let outcome = restoring.await.ok();
            if let Some(window) = weak.upgrade() {
                *window.restored_to.borrow_mut() = Some(target);
                window.finished(outcome, total);
            }
        });
    }

    /// Marks the window as restoring, with `cancel` to stop it, or as done with `None`.
    fn set_busy(&self, cancel: Option<Arc<AtomicBool>>) {
        let busy = cancel.is_some();
        *self.running.borrow_mut() = cancel;
        self.progress_row.set_visible(busy);
        self.tree_scroll.set_sensitive(!busy);
        self.versions.set_sensitive(!busy);
        self.back_in_place.set_sensitive(!busy);
        self.into_folder.set_sensitive(!busy);
        self.choose.set_sensitive(!busy);
        if busy {
            self.result.set_visible(false);
            self.details.set_visible(false);
            self.open_folder.set_visible(false);
        }
        self.update_restore_button();
        (self.on_busy)(busy);
    }

    /// Shows how the restore went, clears the ticks, and reads the tree again, since what is on
    /// the computer has changed.
    fn finished(self: &Rc<Self>, outcome: Option<Outcome>, total: usize) {
        self.set_busy(None);
        let language = self.shared.language.get();
        let Some(outcome) = outcome else {
            self.show_message(Text::RestoreStopped.get(language));
            return;
        };
        let shown = result_text(&outcome, total, language);
        self.result.set_label(&shown.summary);
        self.result.set_visible(true);
        let mut lines = Vec::new();
        lines.extend(shown.full);
        lines.extend(shown.skipped);
        lines.extend(shown.failed_heading);
        lines.extend(shown.failed.into_iter().map(|line| format!("  {line}")));
        self.details.set_label(&lines.join("\n"));
        self.details.set_visible(!lines.is_empty());
        self.open_folder.set_visible(outcome.restored > 0);
        *self.shared.choice.borrow_mut() = Choice::default();
        // A suggested folder that is used now isn't new any more.
        if *self.destination.borrow() == Destination::Folder(self.suggested.borrow().clone()) {
            self.suggest_folder();
        }
        self.build_tree();
        self.show_versions();
        self.update_restore_button();
    }

    /// Asks whether to stop the restore when the window is closed during one.
    fn ask_to_stop(self: &Rc<Self>) {
        let language = self.shared.language.get();
        let dialog = AlertDialog::builder()
            .message(Text::RestoreStopQuestion.get(language))
            .detail(Text::RestoreStopDetail.get(language))
            .buttons([
                Text::RestoreKeepGoing.get(language),
                Text::RestoreStop.get(language),
            ])
            .cancel_button(0)
            .default_button(0)
            .modal(true)
            .build();
        let weak = Rc::downgrade(self);
        dialog.choose(
            Some(&self.window),
            None::<&gio::Cancellable>,
            move |answer| {
                let Some(window) = weak.upgrade() else { return };
                if answer != Ok(1) {
                    return;
                }
                if let Some(cancel) = window.running.borrow_mut().take() {
                    cancel.store(true, Ordering::Relaxed);
                }
                (window.on_busy)(false);
                window.window.close();
            },
        );
    }

    /// Stops a restore that is going on, when the stick has gone or Babawus closes.
    pub fn stop(&self) {
        if let Some(cancel) = self.running.borrow_mut().take() {
            cancel.store(true, Ordering::Relaxed);
            (self.on_busy)(false);
        }
    }
}

/// The backup in the folder `root` on a stick in `format`, for a thread of its own.
fn backup_at(root: PathBuf, format: Format) -> Backup<'static> {
    Backup {
        medium: &RealFs,
        root,
        format,
        system: &RealFs,
        home: glib::home_dir(),
    }
}

/// The path, relative to the home folder, of the item in `row`.
fn row_path(row: &TreeListRow) -> Option<PathBuf> {
    let object = row.item().and_downcast::<glib::BoxedAnyObject>()?;
    Some(object.borrow::<Item>().relative.clone())
}
