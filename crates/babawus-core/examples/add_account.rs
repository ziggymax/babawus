//! Adds this computer and account to a Babawus medium, for checking by hand until the
//! configuration editor exists (T-018).
//!
//! `cargo run -p babawus-core --example add_account -- <medium root> <selected>...` adds them if
//! they aren't there yet, and sets what is selected, relative to the home folder, for example
//! `Documents` or `Pictures/2026`.

use std::path::PathBuf;

use babawus_core::config::{Loaded, folders_on_medium, load, save};
use babawus_core::fs::RealFs;
use babawus_core::identity::this_computer_id;
use babawus_core::platform;
use babawus_core::versions::Format;

fn main() {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(args.next().expect("give the medium's root folder"));
    let selected: Vec<String> = args.collect();
    let id = this_computer_id(&RealFs)
        .expect("read the computer ID")
        .expect("this computer has an ID");
    let hostname = platform::hostname(&RealFs).expect("read the hostname");
    let username = std::env::var("USER").expect("USER is set");
    let format = match std::fs::read_to_string("/proc/self/mounts")
        .unwrap()
        .lines()
        .find(|line| line.split(' ').nth(1) == root.to_str())
        .and_then(|line| line.split(' ').nth(2))
    {
        Some("exfat") => Format::ExFat,
        Some("ext4") => Format::Ext4,
        other => panic!(
            "{} is not an exFAT or Ext4 mount: {other:?}",
            root.display()
        ),
    };
    let Ok(Loaded::Ready(mut config)) = load(&RealFs, &root) else {
        panic!("{} has no usable configuration", root.display());
    };
    if config.computer(&id).is_none() {
        let on_medium = folders_on_medium(&RealFs, &root).expect("list the folders");
        let folder = config.suggest_folder(&hostname, &on_medium);
        config
            .add_computer(&id, &folder, &hostname, &on_medium)
            .expect("add the computer");
    }
    if config.account(&id, &username).is_none() {
        config.add_account(&id, &username).expect("add the account");
    }
    let computer = config
        .computers
        .iter_mut()
        .find(|computer| computer.id == id)
        .unwrap();
    let account = computer
        .accounts
        .iter_mut()
        .find(|account| account.username == username)
        .unwrap();
    account.selected = selected;
    save(&RealFs, &root, format, &config).expect("save the configuration");
    println!("{}", config.to_text(std::time::SystemTime::now()));
}
