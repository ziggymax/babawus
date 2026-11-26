//! Test and debug output, turned on with `--debug=<topics>`, such as `babawus --debug=sticks`.
//! Each topic that is on makes Babawus print what it sees to standard error, where a test can
//! read it without looking at the screen. It is for testing, so the Help and the README don't
//! mention it; TestSetup.md does (issue 170, T-128).

use crate::media::Snapshot;
use crate::start_screen::{self, Unlisted};

/// The command line option, followed by `=` and a comma-separated list of topics.
pub const OPTION: &str = "--debug";

/// What can be turned on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topic {
    /// The sticks the start screen lists, each time the list is read again.
    Sticks,
}

impl Topic {
    const ALL: [Topic; 1] = [Topic::Sticks];

    /// The topic's name in the option.
    pub fn name(self) -> &'static str {
        match self {
            Topic::Sticks => "sticks",
        }
    }

    fn named(name: &str) -> Option<Topic> {
        Topic::ALL.into_iter().find(|topic| topic.name() == name)
    }
}

/// The topics that are on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Debug {
    topics: Vec<Topic>,
}

impl Debug {
    pub fn on(&self, topic: Topic) -> bool {
        self.topics.contains(&topic)
    }
}

/// Takes every `--debug=<topics>` out of `args`, so that GTK doesn't see them, and returns the
/// topics that are on, and a problem for each topic that isn't known, which is then ignored.
pub fn take(args: &mut Vec<String>) -> (Debug, Vec<String>) {
    let mut debug = Debug::default();
    let mut problems = Vec::new();
    let prefix = format!("{OPTION}=");
    args.retain(|arg| {
        let list = if arg == OPTION {
            ""
        } else if let Some(list) = arg.strip_prefix(&prefix) {
            list
        } else {
            return true;
        };
        if list.is_empty() {
            problems.push(format!("{OPTION} needs topics, such as {prefix}sticks"));
        }
        for name in list.split(',').filter(|name| !name.is_empty()) {
            match Topic::named(name) {
                Some(topic) if !debug.topics.contains(&topic) => debug.topics.push(topic),
                Some(_) => {}
                None => problems.push(format!(
                    "Unknown {OPTION} topic {name:?}; the topics are: {}",
                    known_topics()
                )),
            }
        }
        false
    });
    (debug, problems)
}

fn known_topics() -> String {
    Topic::ALL.map(Topic::name).join(", ")
}

/// The line printed for [`Topic::Sticks`]: the labels of the sticks the start screen lists, and
/// what kind of stick is plugged in but not listed, if any. For example
/// `Babawus debug sticks: listed "BACKUP-A", "SMOKE"`.
pub fn sticks_line(snapshot: &Snapshot) -> String {
    let labels: Vec<String> = snapshot
        .media()
        .iter()
        .map(|medium| format!("{:?}", medium.volume.label))
        .collect();
    let mut line = format!("Babawus debug {}: listed ", Topic::Sticks.name());
    if labels.is_empty() {
        line.push_str("none");
    } else {
        line.push_str(&labels.join(", "));
    }
    match start_screen::unlisted(snapshot) {
        Some(Unlisted::NotOpened) => line.push_str("; also plugged in: one that isn't opened"),
        Some(Unlisted::CantBeUsed) => line.push_str("; also plugged in: one that can't be used"),
        None => {}
    }
    line
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::media::{Drive, Volume};

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn without_the_option_nothing_is_on_and_the_arguments_stay() {
        let mut given = args(&["babawus", "--checklastbackup"]);

        let (debug, problems) = take(&mut given);

        assert!(!debug.on(Topic::Sticks));
        assert!(problems.is_empty());
        assert_eq!(given, args(&["babawus", "--checklastbackup"]));
    }

    #[test]
    fn a_known_topic_is_turned_on_and_the_option_taken_out() {
        let mut given = args(&["babawus", "--debug=sticks", "--checklastbackup"]);

        let (debug, problems) = take(&mut given);

        assert!(debug.on(Topic::Sticks));
        assert!(problems.is_empty());
        assert_eq!(given, args(&["babawus", "--checklastbackup"]));
    }

    #[test]
    fn an_unknown_topic_is_reported_and_the_known_ones_still_count() {
        let mut given = args(&["babawus", "--debug=locks,sticks,,sticks"]);

        let (debug, problems) = take(&mut given);

        assert!(debug.on(Topic::Sticks));
        assert_eq!(
            problems,
            ["Unknown --debug topic \"locks\"; the topics are: sticks"]
        );
        assert_eq!(given, args(&["babawus"]));
    }

    #[test]
    fn the_option_without_topics_is_reported() {
        for option in ["--debug", "--debug="] {
            let mut given = args(&["babawus", option]);

            let (debug, problems) = take(&mut given);

            assert_eq!(debug, Debug::default());
            assert_eq!(problems, ["--debug needs topics, such as --debug=sticks"]);
            assert_eq!(given, args(&["babawus"]));
        }
    }

    fn stick(name: &str, label: &str, mounted: bool) -> (Drive, Volume) {
        let drive = Drive {
            object_path: format!("/drives/{name}"),
            id: name.to_string(),
            removable: true,
            connection_bus: "usb".to_string(),
            ..Drive::default()
        };
        let volume = Volume {
            object_path: format!("/blocks/{name}1"),
            drive: drive.object_path.clone(),
            label: label.to_string(),
            file_system: "exfat".to_string(),
            mount_point: mounted.then(|| PathBuf::from(format!("/media/tester/{label}"))),
            ..Volume::default()
        };
        (drive, volume)
    }

    fn snapshot(sticks: &[(Drive, Volume)]) -> Snapshot {
        Snapshot {
            drives: sticks.iter().map(|(drive, _)| drive.clone()).collect(),
            volumes: sticks.iter().map(|(_, volume)| volume.clone()).collect(),
            blocks: Vec::new(),
        }
    }

    #[test]
    fn the_sticks_line_names_each_listed_stick() {
        let line = sticks_line(&snapshot(&[
            stick("b", "SMOKE", true),
            stick("a", "BACKUP-A", true),
        ]));

        assert_eq!(line, "Babawus debug sticks: listed \"BACKUP-A\", \"SMOKE\"");
    }

    #[test]
    fn the_sticks_line_says_when_none_is_listed() {
        assert_eq!(
            sticks_line(&Snapshot::default()),
            "Babawus debug sticks: listed none"
        );
    }

    #[test]
    fn the_sticks_line_mentions_a_stick_that_isnt_opened() {
        let line = sticks_line(&snapshot(&[stick("a", "SMOKE", false)]));

        assert_eq!(
            line,
            "Babawus debug sticks: listed none; also plugged in: one that isn't opened"
        );
    }
}
