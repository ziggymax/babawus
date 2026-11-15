//! The pictures Babawus draws: the logo, the sticks and the marks. They are built into the
//! program, so nothing has to be installed for them.

use std::cell::RefCell;
use std::collections::HashMap;

use babawus_core::start_screen::Mark;
use babawus_core::verdict_screen::VerdictMark;
use gtk4::gdk::Texture;
use gtk4::gdk_pixbuf::Pixbuf;
use gtk4::{Image, gio, glib};

/// The logo with the name below the elephant, for the start screen only.
const LOGO_WITH_NAME: &[u8] = include_bytes!("../../../assets/BabawusLogo-plain.svg");
/// The elephant alone, for everywhere else, such as About.
const LOGO: &[u8] = include_bytes!("../../../assets/BabawusLogo-NoText-plain.svg");
const STICK_BABAWUS: &[u8] = include_bytes!("../assets/ui/stick-babawus.svg");
const STICK_OTHER: &[u8] = include_bytes!("../assets/ui/stick-other.svg");
const EJECT: &[u8] = include_bytes!("../assets/ui/eject.svg");
const MARK_READY: &[u8] = include_bytes!("../assets/ui/mark-ready.svg");
const MARK_ATTENTION: &[u8] = include_bytes!("../assets/ui/mark-attention.svg");
const MARK_UNUSABLE: &[u8] = include_bytes!("../assets/ui/mark-unusable.svg");
const MARK_FAILED: &[u8] = include_bytes!("../assets/ui/mark-failed.svg");

/// Drawn at twice the size shown, so that it stays sharp on a screen with scaling.
const SCALE: i32 = 2;

/// The logo with the name, for the upper right corner of the start screen.
pub fn logo_with_name() -> Texture {
    texture(LOGO_WITH_NAME, 64)
}

/// The logo without the name, for every other place.
pub fn logo() -> Texture {
    texture(LOGO, 64)
}

pub fn stick(babawus: bool) -> Texture {
    texture(if babawus { STICK_BABAWUS } else { STICK_OTHER }, 48)
}

/// The eject symbol in its own white circle, for each row of the stick list.
pub fn eject() -> Texture {
    texture(EJECT, 36)
}

pub fn mark(mark: Mark) -> Texture {
    let svg = match mark {
        Mark::Ready => MARK_READY,
        Mark::Attention => MARK_ATTENTION,
        Mark::Unusable => MARK_UNUSABLE,
    };
    texture(svg, 32)
}

pub fn verdict_mark(mark: VerdictMark) -> Texture {
    let svg = match mark {
        VerdictMark::Good => MARK_READY,
        VerdictMark::Attention => MARK_ATTENTION,
        VerdictMark::Failed => MARK_FAILED,
    };
    texture(svg, 48)
}

/// An image of `size` pixels, for a texture from this module.
pub fn image(texture: &Texture, size: i32) -> Image {
    let image = Image::from_paintable(Some(texture));
    image.set_pixel_size(size);
    image
}

thread_local! {
    /// The pictures drawn so far, by the address of their SVG and their size.
    static DRAWN: RefCell<HashMap<(usize, i32), Texture>> = RefCell::new(HashMap::new());
}

/// The SVG `svg`, drawn to fit a square of `size` pixels, times [`SCALE`]. Each is only drawn
/// once.
fn texture(svg: &'static [u8], size: i32) -> Texture {
    let key = (svg.as_ptr() as usize, size);
    DRAWN.with_borrow_mut(|drawn| drawn.entry(key).or_insert_with(|| draw(svg, size)).clone())
}

fn draw(svg: &'static [u8], size: i32) -> Texture {
    let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from_static(svg));
    let pixbuf = Pixbuf::from_stream_at_scale(
        &stream,
        size * SCALE,
        size * SCALE,
        true,
        None::<&gio::Cancellable>,
    )
    .expect("the built-in pictures can be drawn");
    Texture::for_pixbuf(&pixbuf)
}

#[cfg(test)]
mod tests {
    /// The icon the package installs for the menu and the panel (issue 139).
    const MENU_ICON: &str = include_str!("../assets/io.github.ziggymax.Babawus.svg");

    /// The menu icon is the logo without the name, with a margin of 1/16 of its size on each
    /// side, as Mint-Y's app icons have. Nothing else may differ, so a new logo needs a new
    /// menu icon too.
    #[test]
    fn the_menu_icon_is_the_logo_with_a_margin() {
        let logo = std::str::from_utf8(super::LOGO).unwrap();
        let size = "   width=\"90mm\"\n   height=\"90mm\"\n   viewBox=\"0 0 90.000001 90\"";
        let with_margin = "   width=\"102.857143mm\"\n   height=\"102.857143mm\"\n   \
                           viewBox=\"-6.4285715 -6.4285715 102.857143 102.857143\"";
        assert_eq!(
            logo.matches(size).count(),
            1,
            "the logo is no longer 90 × 90 mm"
        );
        assert_eq!(MENU_ICON, logo.replace(size, with_margin));
    }
}
