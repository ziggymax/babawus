//! Checks that the AppStream metainfo stays in step with the packages (issue 175, T-141): the
//! same summary and description, the IDs the desktop entry and the icon have, and, for a release
//! version, a `<release>` entry for it at the top.

const METAINFO: &str = include_str!("../assets/io.github.ziggymax.Babawus.metainfo.xml");
const CARGO: &str = include_str!("../Cargo.toml");
const WORKSPACE: &str = include_str!("../../../Cargo.toml");
const APP_ID: &str = "io.github.ziggymax.Babawus";

/// The text between the first `<tag>` (or `<tag ...>`) and its `</tag>`, after `from`.
fn element<'a>(text: &'a str, tag: &str, from: usize) -> Option<(&'a str, usize)> {
    let mut at = from;
    loop {
        let start = at + text[at..].find(&format!("<{tag}"))?;
        let after_name = start + 1 + tag.len();
        if text[after_name..].starts_with(['>', ' ', '\n']) {
            let open_end = after_name + text[after_name..].find('>')? + 1;
            let close = open_end + text[open_end..].find(&format!("</{tag}>"))?;
            return Some((&text[open_end..close], close));
        }
        at = after_name;
    }
}

fn first(text: &str, tag: &str) -> String {
    element(text, tag, 0)
        .unwrap_or_else(|| panic!("no <{tag}> in the metainfo"))
        .0
        .trim()
        .to_owned()
}

fn package() -> toml::Table {
    CARGO.parse().unwrap()
}

#[test]
fn the_ids_are_those_of_the_desktop_entry_and_the_icon() {
    assert_eq!(first(METAINFO, "id"), APP_ID);
    assert_eq!(first(METAINFO, "launchable"), format!("{APP_ID}.desktop"));
    assert_eq!(first(METAINFO, "icon"), APP_ID);
    let assets = package()["package"]["metadata"]["deb"]["assets"].to_string();
    for file in [".desktop", ".svg", ".metainfo.xml"] {
        assert!(assets.contains(&format!("assets/{APP_ID}{file}")), "{file}");
    }
}

#[test]
fn the_summary_and_description_are_those_of_the_packages() {
    let package = package();
    assert_eq!(
        first(METAINFO, "summary"),
        package["package"]["description"].as_str().unwrap()
    );

    let deb = package["package"]["metadata"]["deb"]["extended-description"]
        .as_str()
        .unwrap();
    let wanted: Vec<&str> = deb.split("\n\n").map(str::trim).collect();
    let description = first(METAINFO, "description");
    let mut paragraphs = Vec::new();
    let mut at = 0;
    while let Some((paragraph, end)) = element(&description, "p", at) {
        paragraphs.push(paragraph.trim().to_owned());
        at = end;
    }
    assert_eq!(paragraphs, wanted);
}

#[test]
fn a_release_version_is_the_newest_release_listed() {
    let workspace: toml::Table = WORKSPACE.parse().unwrap();
    let version = workspace["workspace"]["package"]["version"]
        .as_str()
        .unwrap();
    let releases = first(METAINFO, "releases");
    let newest = releases
        .split("version=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next());
    if version.contains('-') {
        // An interim version, such as 1.2.0-alpha.2, has no entry of its own.
        assert!(newest.is_none_or(|newest| !newest.contains('-')));
    } else {
        assert_eq!(
            newest,
            Some(version),
            "add <release version=\"{version}\" date=\"YYYY-MM-DD\"/> at the top of <releases> in the metainfo"
        );
    }
}
