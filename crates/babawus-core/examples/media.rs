//! Prints the media list, as udisks2 and the media themselves report it, for checking by hand.
//!
//! `cargo run -p babawus-core --example media` prints it once. With `--follow`, it prints it
//! again after every change, until it is stopped with Ctrl+C.

use babawus_core::fs::RealFs;
use babawus_core::identity::this_computer_id;
use babawus_core::lock::ThisProcess;
use babawus_core::media::{Snapshot, Viewer};
use babawus_core::udisks::Udisks;
use gio::glib;

fn main() {
    let follow = std::env::args().any(|arg| arg == "--follow");
    let udisks = Udisks::connect().expect("connect to udisks2");
    let computer_id = this_computer_id(&RealFs)
        .expect("read the computer ID")
        .unwrap_or_default();
    let this = ThisProcess::read(&RealFs, &computer_id).expect("read this process");
    let username = glib::user_name().to_string_lossy().into_owned();
    let print = move |snapshot: &Snapshot| {
        let viewer = Viewer {
            system: &RealFs,
            this: &this,
            username: &username,
        };
        println!("{} USB media:", snapshot.media().len());
        for medium in snapshot.media() {
            let drive = &medium.drive;
            let volume = &medium.volume;
            println!(
                "  {} ({}, {} bytes) on {} {}, {} bytes, drive ID {}",
                volume.label,
                volume.file_system,
                volume.size,
                drive.vendor,
                drive.model,
                drive.size,
                drive.id,
            );
            println!(
                "    {}{}: {:?}",
                medium.root.display(),
                if volume.read_only { ", read-only" } else { "" },
                viewer.status(&RealFs, &medium),
            );
        }
    };
    print(&udisks.snapshot());
    if follow {
        let udisks = std::rc::Rc::new(udisks);
        let weak = std::rc::Rc::downgrade(&udisks);
        udisks.connect_changed(move || {
            if let Some(udisks) = weak.upgrade() {
                print(&udisks.snapshot());
            }
        });
        glib::MainLoop::new(None, false).run();
    }
}
