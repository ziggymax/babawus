# Adding a language to Babawus

Babawus reads its texts from language files, one per language, each time it starts. To add a language, write one more file and copy it into place. No new build of Babawus is needed. The rules are in "Translations" in [Requirements.md](projectmgt/Requirements.md), and the reasons in issues 147 to 149.

## What a language file holds

A language file is a [TOML](https://toml.io) text file, named after the language's two-letter code, for example `fr.toml` for French. It starts with a `[language]` table:

```toml
[language]
code = "fr"            # the same as the file name
name = "Français"      # shown in the Language menu, in the language itself
decimal = ","          # in "15,5 GB"
thousands = " "        # in "1 234 fichiers"
quotes = ["« ", " »"]  # the marks that the Help's plain "" are shown as
```

Then comes a `[texts]` table, with every text Babawus shows, by name:

```toml
[texts]
WindowTitle = "Sauvegarde"
StartBackup = "Démarrer la sauvegarde"
InProgress = "Sauvegarde en cours (par {user})"
```

- **Every text must be there**, and none may be empty. The names are the same in every file, so the English file shows which texts there are.
- **Placeholders** in braces, such as `{user}` or `{files}`, are filled in by Babawus. Keep each one exactly as in English, in whatever place the sentence needs it. Don't translate what is inside the braces.
- **Numbered messages:** a text whose name ends with a number, such as `ReasonMediumFull_14`, is a message that Babawus shows with "#14" after it. Keep the number in the name.
- **The Help** is the text `Help`, at the end of the file, written over many lines between `"""` and `"""`. Keep its layout: a line starting with "# " is a heading, a line starting with "- " a bullet, a line starting with "> " a notice in italics, and a line with only "#" and a number, such as "#14", begins the explanation of that numbered message. Babawus writes the message itself after the number, from your translation of `ReasonMediumFull_14`, so translate only the explanation below it, and keep every number. If one is missing, Babawus still offers the language, and says which numbers are missing.
- **References in the Help:** a text's name in braces, such as `{ChangeSetup}`, is filled in with that text, so that the Help always names the buttons as they are. Keep them as they are. Only texts without placeholders can be referred to.
- **Quotes in the Help** are written as straight double quotes, `"{ChangeSetup}"`, and shown as the marks in `quotes`. A straight quote with a space on both sides, as in the list of characters exFAT can't store, stays as it is.
- **Lines starting with `#`** are comments, which Babawus ignores, except inside the Help, where they are headings and entries. The English file uses them to explain some texts.
- **Quotes in other texts:** a text is written between straight double quotes. A double quote inside a one-line text needs a backslash, `\"`, so it is easier to use the language's own quotation marks there, such as « » or „ “.

## Writing the file

1. Copy the English file, `/usr/share/babawus/languages/en.toml`, to your home folder, and rename it to the new language's code, for example `fr.toml`.
1. Change the `[language]` table for the new language.
1. Translate every text after the `=`, leaving the names before it as they are.

An AI or a machine translation can do step 3. Give it the whole file, and ask it to translate only the quoted texts, to keep the names, placeholders and comments unchanged, and to address the user in the familiar form, as the German file does with "du". Then have someone who speaks the language read the result. The texts are short and are seen by people who don't know much about computers, so plain words matter more than literal ones.

Babawus also has `de.toml`, in German, and `da.toml`, in Danish, to compare with.

## Installing and checking it

1. Close Babawus.
1. Copy the file into place. This needs an administrator's password:

    ```bash
    sudo cp fr.toml /usr/share/babawus/languages/
    ```

1. Start Babawus from a terminal, so that you can see what it says about the files:

    ```bash
    babawus
    ```

1. If the file is fine, the terminal shows nothing about it, and the language is in the Language menu. Choose it there to see the texts.

An update of Babawus keeps a language file added like this. Uninstalling Babawus leaves it, and its folder, behind: remove them with `sudo rm -r /usr/share/babawus`.

## When a language isn't offered

Babawus leaves out a file it can't use, and says why in the terminal, for example:

```text
Babawus: /usr/share/babawus/languages/fr.toml: not offered, because these texts are missing: StartBackup, CantStart
```

| The reason | What to do |
| --- | --- |
| `it isn't valid: …` | The file isn't valid TOML. The message gives the line and column; often a quote is missing at the end of a text. |
| `its code "…" isn't its file name, ….toml` | Make `code` the same as the file name without `.toml`. |
| `its name is empty` or `its decimal is empty` | Fill in `name` or `decimal` in the `[language]` table. |
| `its quotes aren't two marks`, or `it isn't valid` ending with "missing field `quotes`" | Add `quotes`, with the opening and the closing mark, to the `[language]` table, such as `quotes = ["« ", " »"]`. |
| `the Help refers to what isn't a text without placeholders: …` | Each named reference, such as `{ChangeSetUp}`, is misspelt, or is a text with a placeholder. Copy the name from `en.toml`. |
| `these texts are missing: …` | Add the named texts, copied from `en.toml` and translated, or fill in the empty ones. This also happens after an update of Babawus adds texts: compare the file with the new `en.toml`. |
| `these texts have other placeholders than in English: …` | Make the placeholders in the named texts the same as in English. |
| `it can't be read` or `it isn't UTF-8 text` | Save the file as UTF-8, and make sure everyone can read it: `sudo chmod 644 /usr/share/babawus/languages/fr.toml`. |

A warning that the Help "has text after #14", or that "a quotation mark in the Help has no partner", doesn't stop the file from being offered either: Babawus leaves out the text after the number, or shows that line's quotes as written.

A warning that a text "isn't one Babawus knows, and is left out" doesn't stop the file from being offered. It usually means a misspelt name, which then also shows up as a missing text, or a file made for a newer Babawus.

If Babawus says that it can't start, its own English file, `en.toml`, is missing or damaged. Installing Babawus again puts it back.

## Sharing a language

To have a language come with Babawus, open a pull request on [GitHub](https://github.com/ziggymax/babawus) that adds the file to the `languages/` folder. The tests then check that it is complete.
