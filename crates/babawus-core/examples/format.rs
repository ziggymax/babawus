//! Formats a USB medium for checking by hand. This erases everything on the drive.
//!
//! `cargo run -p babawus-core --example format` lists the drives that may be formatted.
//! `cargo run -p babawus-core --example format -- <drive ID> exfat|ext4 <label> --erase-everything`
//! formats one, after the same checks as Babawus.

use std::time::Instant;

use babawus_core::format::suggest_label;
use babawus_core::platform::Sticks;
use babawus_core::versions::Format;
use gio::glib;

const CONFIRM: &str = "--erase-everything";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let udisks = Sticks::connect().expect("connect to udisks2");
    let snapshot = udisks.snapshot();
    let [id, format, label, confirm] = args.as_slice() else {
        println!("Drives that may be formatted:");
        for target in snapshot.formattable() {
            let drive = &target.drive;
            println!(
                "  {} ({} {}, {} bytes, {:?}, {} mounted)",
                drive.id,
                drive.vendor,
                drive.model,
                drive.size,
                target.table,
                target.mounted.len()
            );
        }
        let labels = snapshot.volumes.iter().map(|volume| volume.label.as_str());
        println!("Suggested label: {}", suggest_label(labels));
        println!("To format one: <drive ID> exfat|ext4 <label> {CONFIRM}");
        return;
    };
    let format = match format.as_str() {
        "exfat" => Format::ExFat,
        "ext4" => Format::Ext4,
        other => panic!("{other} is not exfat or ext4"),
    };
    assert_eq!(confirm, CONFIRM, "confirm with {CONFIRM}");

    println!("Formatting…");
    let start = Instant::now();
    let result = glib::MainContext::default().block_on(udisks.format(id, format, label));
    let seconds = start.elapsed().as_secs_f64();
    match result {
        Ok(root) => println!("Done in {seconds:.1} s, mounted at {}", root.display()),
        Err(error) => println!(
            "Failed after {seconds:.1} s, nothing erased: {}: {error:?}",
            error.nothing_erased()
        ),
    }
}
