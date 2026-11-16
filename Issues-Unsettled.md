# Babawus – Unsettled issues

The questions about Babawus that are still open. Once decided, an issue moves word for word to [Issues-Resolved.md](Issues-Resolved.md), with the decision and its date.

## How this list works

- **Numbers never change or get reused.** A new issue takes the next free number here, and keeps it when it moves. The next free number is 155.
- **References** elsewhere say just "issue 132", without a file name, so they never go stale. GitHub issues and pull requests are "#63".
- **Each issue** states the question, the options with what speaks for each, and which tasks wait for it. A task in [Tasks-Open.md](Tasks-Open.md) points at every open issue that needs settling.
- **A decision** is written as "**Decided (date):**" under the issue as soon as it is given, and the issue then moves to Issues-Resolved.md.

## Open issues

158. **The Help copies texts, which can drift (T-087).** The user found that a Help heading such as "#1 USB-stikket er ikke åbnet." repeats a text, here `StickNotOpened`, and nothing checks that the two stay the same. Today the checks on a language file are: valid TOML, `code` the same as the file name, `name` and `decimal` not empty, every English text present and not empty, the same placeholders as English, a warning for unknown texts, and a warning for each number the Help has no entry for. Texts are never compared with each other. Of the 56 Help headings in English, 29 are exact copies of the numbered text, 3 of another text, 9 the numbered text's first sentence, and 15 have their own wording, shortened or without a placeholder; Danish #24 has already drifted from `SkipVersionNameTooLong`. The body of the Help also copies button names, such as "Change setup…".
    - **Proposed (2026-11-16):** the Help refers to texts, for example `#1 {StickNotOpened}.`, and Babawus fills them in when it loads the file, rejecting a file with a reference to an unknown text or to one with a placeholder. To do this before 1.0.0, as the language file format is then something others build on.
    - **To decide at the start of T-087:** the exact form of a reference; whether the body's button names become references too; and, for each heading with its own wording or only a first sentence, whether it stays free text, becomes the full message, or gets a short heading text of its own.
