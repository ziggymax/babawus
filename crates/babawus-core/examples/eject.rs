//! Ejects a USB medium for checking by hand: it is unmounted and, if possible, powered off.
//!
//! `cargo run -p babawus-core --example eject` lists the USB media.
//! `cargo run -p babawus-core --example eject -- <drive ID>` ejects one.

use std::time::Instant;

use babawus_core::udisks::Udisks;
use gio::glib;

fn main() {
    let udisks = Udisks::connect().expect("connect to udisks2");
    let Some(id) = std::env::args().nth(1) else {
        println!("USB media:");
        let snapshot = udisks.snapshot();
        for drive in snapshot.drives.iter().filter(|drive| drive.is_usb_medium()) {
            let power = if drive.can_power_off {
                "can power off"
            } else {
                "can't power off"
            };
            println!("  {} ({} {}, {power})", drive.id, drive.vendor, drive.model);
        }
        return;
    };
    let start = Instant::now();
    let result = glib::MainContext::default().block_on(udisks.eject(&id));
    println!("{result:?} after {:.1} s", start.elapsed().as_secs_f64());
}
