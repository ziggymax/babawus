//! Starting pages at the top. A page that scrolls starts at the top whenever it opens or
//! changes part, so that its heading is always seen first (Issues.md item 118).

use gtk4::prelude::*;
use gtk4::{ScrolledWindow, Widget};

/// Scrolls every scrolled area inside `widget`, `widget` included, back to the top.
pub fn to_top(widget: &impl IsA<Widget>) {
    let widget = widget.as_ref();
    if let Some(scroll) = widget.downcast_ref::<ScrolledWindow>() {
        let adjustment = scroll.vadjustment();
        adjustment.set_value(adjustment.lower());
    }
    let mut child = widget.first_child();
    while let Some(current) = child {
        to_top(&current);
        child = current.next_sibling();
    }
}
