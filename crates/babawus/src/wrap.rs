//! A row of buttons that wraps like words: each keeps its own width and sits close to the one
//! before, and one that doesn't fit starts a new line. GTK 4.14 has no such container, and its
//! `FlowBox` lines its children up in columns, which spreads them out (issue 125).

use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{Orientation, SizeRequestMode, Widget, glib};

mod imp {
    use std::cell::Cell;

    use super::*;

    #[derive(Default)]
    pub struct WrapLayout {
        pub spacing: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WrapLayout {
        const NAME: &'static str = "BabawusWrapLayout";
        type Type = super::WrapLayout;
        type ParentType = gtk4::LayoutManager;
    }

    impl ObjectImpl for WrapLayout {}

    impl LayoutManagerImpl for WrapLayout {
        fn request_mode(&self, _widget: &Widget) -> SizeRequestMode {
            SizeRequestMode::HeightForWidth
        }

        fn measure(
            &self,
            widget: &Widget,
            orientation: Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let spacing = self.spacing.get();
            let children = shown_children(widget);
            if orientation == Orientation::Horizontal {
                // At its narrowest, each child is on a line of its own; at its widest, all are
                // on one line.
                let widths: Vec<(i32, i32)> = children
                    .iter()
                    .map(|child| {
                        let (minimum, natural, _, _) = child.measure(Orientation::Horizontal, -1);
                        (minimum, natural)
                    })
                    .collect();
                let minimum = widths.iter().map(|(minimum, _)| *minimum).max();
                let natural: i32 = widths.iter().map(|(_, natural)| natural).sum::<i32>()
                    + spacing * (widths.len() as i32 - 1).max(0);
                return (minimum.unwrap_or(0), natural, -1, -1);
            }
            let width = if for_size < 0 { i32::MAX } else { for_size };
            let lines = lines(&children, width, spacing);
            let height = lines.iter().map(|line| line.height).sum::<i32>()
                + spacing * (lines.len() as i32 - 1).max(0);
            (height, height, -1, -1)
        }

        fn allocate(&self, widget: &Widget, width: i32, _height: i32, _baseline: i32) {
            let spacing = self.spacing.get();
            let children = shown_children(widget);
            let mut y = 0;
            for line in lines(&children, width, spacing) {
                let mut x = 0;
                for (child, child_width) in line.children {
                    let rectangle = gtk4::Allocation::new(x, y, child_width, line.height);
                    child.size_allocate(&rectangle, -1);
                    x += child_width + spacing;
                }
                y += line.height + spacing;
            }
        }
    }
}

glib::wrapper! {
    pub struct WrapLayout(ObjectSubclass<imp::WrapLayout>) @extends gtk4::LayoutManager;
}

impl WrapLayout {
    /// A layout with `spacing` pixels between children, and between lines.
    pub fn new(spacing: i32) -> Self {
        let layout: Self = glib::Object::new();
        layout.imp().spacing.set(spacing);
        layout
    }
}

/// One line of the layout: its children with their widths, and its height.
struct Line {
    children: Vec<(Widget, i32)>,
    height: i32,
}

fn shown_children(widget: &Widget) -> Vec<Widget> {
    let mut children = Vec::new();
    let mut child = widget.first_child();
    while let Some(current) = child {
        if current.should_layout() {
            children.push(current.clone());
        }
        child = current.next_sibling();
    }
    children
}

/// Fills lines of `width` pixels with `children` in order, each at its natural width, or the
/// whole width if that is less.
fn lines(children: &[Widget], width: i32, spacing: i32) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    let mut used: i32 = 0;
    for child in children {
        let (_, natural, _, _) = child.measure(Orientation::Horizontal, -1);
        let child_width = natural.min(width);
        if lines.is_empty() || used.saturating_add(spacing + child_width) > width {
            lines.push(Line {
                children: Vec::new(),
                height: 0,
            });
            used = child_width;
        } else {
            used += spacing + child_width;
        }
        let (_, height, _, _) = child.measure(Orientation::Vertical, child_width);
        let line = lines.last_mut().expect("a line was just pushed");
        line.height = line.height.max(height);
        line.children.push((child.clone(), child_width));
    }
    lines
}
