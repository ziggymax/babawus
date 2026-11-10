//! Every text the user sees, in every language. See "Translations" under "Technology" in
//! Requirements.md: the language changes at once, without a restart, so the texts are looked up
//! each time they are shown, rather than through gettext.
//!
//! A text may hold placeholders such as `{label}`, which [`Text::fill`] replaces. Every
//! translation of a text has the same placeholders as the English one.

use crate::settings::Language;

macro_rules! texts {
    ($($name:ident: $english:literal, $german:literal;)*) => {
        /// A text the user sees.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Text {
            $($name,)*
        }

        impl Text {
            /// Every text, for the tests.
            pub const ALL: &[Self] = &[$(Self::$name,)*];

            /// The text in each language, in the order of [`Language::ALL`]. An empty text has
            /// not been translated yet.
            fn translations(self) -> [&'static str; Language::ALL.len()] {
                match self {
                    $(Self::$name => [$english, $german],)*
                }
            }
        }
    };
}

// German follows Issues.md items 90-92: "du", and "Betreuer" for the poweruser.
texts! {
    WindowTitle: "Backup", "Sicherung";
    Menu: "Menu", "Menü";
    MenuLanguage: "Language", "Sprache";
    MenuAdvanced: "Advanced", "Erweitert";
    MenuAbout: "About Babawus", "Über Babawus";
    AdvancedMarker: "Advanced", "Erweitert";
    AdvancedMarkerTooltip: "The poweruser's actions are shown. Choose Advanced in the menu again to hide them.", "Die Funktionen für deinen Betreuer werden angezeigt. Wähle „Erweitert“ im Menü noch einmal, um sie auszublenden.";
    AboutComments: "Basic backup with USB storage", "Einfache Sicherung auf USB-Sticks";
    Logo: "Babawus", "Babawus";
    PlugIn: "Plug in your backup stick", "Steck deinen Sicherungs-Stick ein";
    ChooseMedium: "Choose a stick in the list", "Wähle einen Stick in der Liste";
    AlreadyOpen: "Babawus is already open for {user} on this computer", "Babawus ist auf diesem Computer schon für {user} geöffnet";
    NoUdisks: "Babawus can't see the USB sticks, because the udisks2 service isn't running. Contact your poweruser.", "Babawus kann die USB-Sticks nicht sehen, weil der Dienst udisks2 nicht läuft. Wende dich an deinen Betreuer.";

    Checking: "Checking…", "Wird geprüft …";
    Ready: "Ready", "Bereit";
    NotSetUp: "Not set up for you", "Nicht für dich eingerichtet";
    InProgress: "Backup in progress (by {user})", "Sicherung läuft (für {user})";
    NeedsAttention: "Needs attention", "Braucht Aufmerksamkeit";
    WriteProtected: "Write-protected", "Schreibgeschützt";
    NotBackupMedium: "Not a backup medium", "Kein Sicherungs-Stick";
    OtherAccount: "Open for another account", "Für ein anderes Konto geöffnet";

    ExplainNotSetUp: "This stick hasn't been set up for you on this computer. Contact your poweruser.", "Dieser Stick ist auf diesem Computer nicht für dich eingerichtet. Wende dich an deinen Betreuer.";
    ExplainInProgress: "A backup to this stick is running for {user}. Wait until it has finished.", "Auf diesen Stick läuft gerade eine Sicherung für {user}. Warte, bis sie fertig ist.";
    ExplainNeedsAttention: "The setup on this stick is damaged, or needs a newer Babawus. Contact your poweruser.", "Die Einrichtung auf diesem Stick ist beschädigt oder braucht ein neueres Babawus. Wende dich an deinen Betreuer.";
    ExplainWriteProtected: "This stick can only be read, so no backup can be written to it. If it has a lock switch, slide it to unlocked. Otherwise contact your poweruser.", "Dieser Stick kann nur gelesen werden, deshalb kann keine Sicherung darauf geschrieben werden. Wenn er einen Schiebeschalter hat, schieb ihn auf „offen“. Sonst wende dich an deinen Betreuer.";
    ExplainOtherAccount: "This stick is still open for another account on this computer, so Babawus can't look at it. If nobody else is using it, pull it out and plug it back in.", "Dieser Stick ist noch für ein anderes Konto auf diesem Computer geöffnet, deshalb kann Babawus ihn nicht ansehen. Wenn ihn sonst niemand benutzt, zieh ihn ab und steck ihn wieder ein.";
    ExplainNotBackupMedium: "This stick isn't set up for backups with Babawus. Your poweruser can prepare it.", "Dieser Stick ist nicht für Sicherungen mit Babawus eingerichtet. Dein Betreuer kann ihn vorbereiten.";

    LastBackupToday: "Last backup: today at {time}", "Letzte Sicherung: heute um {time}";
    LastBackupYesterday: "Last backup: yesterday at {time}", "Letzte Sicherung: gestern um {time}";
    LastBackupDaysAgo: "Last backup: {days} days ago ({date} at {time})", "Letzte Sicherung: vor {days} Tagen ({date} um {time})";
    NoBackupYet: "No backup yet", "Noch keine Sicherung";
    SkippedMark: "Some files were skipped in the last backup", "Bei der letzten Sicherung wurden einige Dateien übersprungen";

    Free: "{size} free", "{size} frei";

    StartBackup: "Start backup", "Sicherung starten";
    StartBackupTooltip: "Back up your files to this stick", "Deine Dateien auf diesen Stick sichern";
    ShowHistory: "Show history", "Verlauf anzeigen";
    ShowHistoryTooltip: "See when backups to this stick ran, and how they went", "Sehen, wann auf diesen Stick gesichert wurde und wie es lief";
    WhatsBackedUp: "What's backed up", "Was wird gesichert";
    WhatsBackedUpTooltip: "See which of your files and folders are backed up to this stick", "Sehen, welche deiner Dateien und Ordner auf diesen Stick gesichert werden";
    Eject: "Eject", "Auswerfen";
    EjectTooltip: "Make this stick safe to pull out", "Den Stick so vorbereiten, dass du ihn gefahrlos abziehen kannst";
    Prepare: "Prepare…", "Vorbereiten …";
    PrepareTooltip: "Erase this stick and set it up for backups with Babawus", "Diesen Stick löschen und für Sicherungen mit Babawus einrichten";
    ChangeSetup: "Change setup…", "Einrichtung ändern …";
    ChangeSetupTooltip: "Change what is backed up to this stick, and for whom", "Ändern, was auf diesen Stick gesichert wird, und für wen";
    CantStart: "No backup can start: {reason}", "Keine Sicherung möglich: {reason}";

    GettingReady: "Finding the files to back up…", "Die zu sichernden Dateien werden gesucht …";
    BackingUp: "Backing up…", "Wird gesichert …";
    Stopping: "Stopping at a safe point…", "Wird an einer sicheren Stelle angehalten …";
    Finishing: "Finishing… Don't pull out the stick yet.", "Wird abgeschlossen … Zieh den Stick noch nicht ab.";
    Overview: "To back up: {files} files ({size})", "Zu sichern: {files} Dateien ({size})";
    OverviewOneFile: "To back up: 1 file ({size})", "Zu sichern: 1 Datei ({size})";
    FilesLeft: "Files left: {files}", "Verbleibende Dateien: {files}";
    BytesLeft: "Left to copy: {size}", "Noch zu kopieren: {size}";
    Cancel: "Cancel", "Abbrechen";
    CancelTooltip: "Stop the backup", "Die Sicherung anhalten";
    StopQuestion: "Stop the backup?", "Sicherung anhalten?";
    StopDetail: "The files copied so far stay on the stick, but the backup won't be finished.", "Die bisher kopierten Dateien bleiben auf dem Stick, aber die Sicherung wird nicht fertig.";
    KeepGoing: "Keep going", "Weitermachen";
    StopBackup: "Stop backup", "Sicherung anhalten";
    Back: "Back", "Zurück";
    BackTooltip: "Go back to the list of sticks", "Zurück zur Liste der Sticks";
    VerdictBusy: "Another backup to this stick is just starting", "Gerade beginnt eine andere Sicherung auf diesen Stick";
    NextWait: "Wait until it has finished, then try again.", "Warte, bis sie fertig ist, und versuch es dann noch einmal.";
    NextRunAgain: "Run the backup again. If it happens again, contact your poweruser.", "Starte die Sicherung noch einmal. Wenn es wieder passiert, wende dich an deinen Betreuer.";
    NextContact: "Contact your poweruser.", "Wende dich an deinen Betreuer.";
    NothingToCopy: "Everything was already up to date.", "Alles war schon auf dem neuesten Stand.";
    Copied: "Copied: {files} ({size})", "Kopiert: {files} ({size})";
    OneFile: "1 file", "1 Datei";
    Files: "{files} files", "{files} Dateien";
    SkippedGroup: "{reason}: {files}", "{reason}: {files}";

    ReasonFilesSkipped: "Some files could not be backed up.", "Einige Dateien konnten nicht gesichert werden.";
    ReasonCancelled: "You stopped the backup.", "Du hast die Sicherung angehalten.";
    ReasonMediumRemoved: "The stick was removed.", "Der Stick wurde abgezogen.";
    ReasonInterrupted: "The backup was interrupted.", "Die Sicherung wurde unterbrochen.";
    ReasonCrashed: "Babawus stopped unexpectedly.", "Babawus wurde unerwartet beendet.";
    ReasonFlushFailed: "The last changes could not be saved on the stick.", "Die letzten Änderungen konnten nicht auf dem Stick gespeichert werden.";
    ReasonMediumFull: "The stick is full.", "Der Stick ist voll.";
    ReasonNotAdded: "This stick isn't set up for you on this computer.", "Dieser Stick ist auf diesem Computer nicht für dich eingerichtet.";
    ReasonConfigDamaged: "The setup on this stick is damaged, or needs a newer Babawus.", "Die Einrichtung auf diesem Stick ist beschädigt oder braucht ein neueres Babawus.";
    ReasonCannotWrite: "The stick can't be written to.", "Auf den Stick kann nicht geschrieben werden.";

    SkipUnreadable: "Couldn't be read", "Konnte nicht gelesen werden";
    SkipDisappeared: "Disappeared during the backup", "Ist während der Sicherung verschwunden";
    SkipNotWritable: "Couldn't be written to the stick", "Konnte nicht auf den Stick geschrieben werden";
    SkipTypeConflict: "A folder of the same name is on the stick", "Auf dem Stick gibt es einen Ordner mit demselben Namen";
    SkipChangedDuringCopy: "Kept changing while being copied", "Hat sich beim Kopieren ständig geändert";
    SkipVersionNameTooLong: "Name too long to keep older versions", "Name zu lang, um ältere Versionen aufzubewahren";
    SkipNameInvalidChars: "Name has characters this stick can't store", "Der Name enthält Zeichen, die dieser Stick nicht speichern kann";
    SkipNameTrailingDot: "Name ends with a dot, which this stick can't store", "Der Name endet mit einem Punkt, den dieser Stick nicht speichern kann";
    SkipNameNotUtf8: "Name isn't valid text", "Der Name ist kein gültiger Text";
    SkipPathTooLong: "Name or folder path too long", "Name oder Ordnerpfad zu lang";
    SkipVersionFolderForm: "Name looks like a folder of older versions", "Der Name sieht aus wie ein Ordner mit älteren Versionen";
    SkipCaseClash: "Name differs only in upper and lower case from another, which this stick can't tell apart", "Der Name unterscheidet sich von einem anderen nur durch Groß- und Kleinschreibung, die dieser Stick nicht unterscheiden kann";
    SkipSelectionMissing: "Chosen for backup, but doesn't exist any more", "Zum Sichern ausgewählt, aber nicht mehr vorhanden";
    SkipOutsideHome: "Chosen for backup, but outside your home folder", "Zum Sichern ausgewählt, aber außerhalb deines persönlichen Ordners";

    DetailsTitle: "Details for your poweruser", "Einzelheiten für deinen Betreuer";
    DetailsTooltip: "What happened, in a form your poweruser can use", "Was passiert ist, in einer Form, mit der dein Betreuer etwas anfangen kann";
    CopyDetails: "Copy", "Kopieren";
    CopyDetailsTooltip: "Copy the details, to paste them into an email or a chat", "Die Einzelheiten kopieren, um sie in eine E-Mail oder einen Chat einzufügen";
    DetailsCopied: "Copied. You can paste the details into an email or a chat.", "Kopiert. Du kannst die Einzelheiten jetzt in eine E-Mail oder einen Chat einfügen.";
    DetailsVerdict: "Verdict: {verdict} ({code})", "Ergebnis: {verdict} ({code})";
    DetailsReason: "Reason: {reason} ({code})", "Grund: {reason} ({code})";
    DetailsTime: "Time: {time}", "Zeit: {time}";
    DetailsAccount: "Account: {account}", "Konto: {account}";
    DetailsMedium: "Medium: {label}", "Stick: {label}";
    DetailsCounts: "Files found: {scanned}, copied: {copied} ({size}), skipped: {skipped}, errors: {errors}, older versions removed: {removed}", "Gefundene Dateien: {scanned}, kopiert: {copied} ({size}), übersprungen: {skipped}, Fehler: {errors}, ältere Versionen entfernt: {removed}";
    DetailsFree: "Free on the medium: {size}", "Frei auf dem Stick: {size}";
    DetailsProblems: "First problems:", "Erste Probleme:";
    DetailsMore: "  … and {more} more, listed in the log on the medium", "  … und {more} weitere, aufgeführt im Protokoll auf dem Stick";
    EjectNow: "Eject {label}", "{label} auswerfen";
    EjectNowTooltip: "Make the stick safe to pull out now", "Den Stick jetzt so vorbereiten, dass du ihn gefahrlos abziehen kannst";
    VerdictAllGood: "All good", "Alles in Ordnung";
    VerdictSkipped: "Some files were skipped", "Einige Dateien wurden übersprungen";
    VerdictNotFinished: "Backup not finished", "Sicherung nicht fertig";
    VerdictFailed: "Backup failed", "Sicherung fehlgeschlagen";

    Ejecting: "Ejecting {label}…", "{label} wird ausgeworfen …";
    Ejected: "You can pull out {label} now.", "Du kannst {label} jetzt abziehen.";
    EjectedUnmountedOnly: "You can pull out {label} safely now, even if its light stays on.", "Du kannst {label} jetzt gefahrlos abziehen, auch wenn sein Lämpchen noch leuchtet.";
    EjectBusy: "{label} is still in use by a program, for example an open file or a file manager window. Close it and try again.", "{label} wird noch von einem Programm benutzt, zum Beispiel von einer geöffneten Datei oder einem Dateimanager-Fenster. Schließ es und versuch es noch einmal.";
    EjectGone: "{label} has already been removed.", "{label} wurde schon abgezogen.";
    EjectFailed: "{label} could not be ejected. Contact your poweruser.", "{label} konnte nicht ausgeworfen werden. Wende dich an deinen Betreuer.";

    HistoryHeading: "Backups to this stick", "Sicherungen auf diesen Stick";
    HistoryEmpty: "No backups to this stick yet.", "Noch keine Sicherungen auf diesen Stick.";
    HistoryUnreadable: "The history on this stick can't be read. Go back and try again.", "Der Verlauf auf diesem Stick kann nicht gelesen werden. Geh zurück und versuch es noch einmal.";
    HistoryRunning: "Backup in progress", "Sicherung läuft";
    HistoryToday: "Today, {time}", "Heute, {time}";
    HistoryYesterday: "Yesterday, {time}", "Gestern, {time}";
    HistoryDate: "{date}, {time}", "{date}, {time}";
    HistoryWho: "By {user} on {computer}", "Von {user} auf {computer}";
    HistorySkipped: "Skipped: {files}", "Übersprungen: {files}";

    CoverageHeading: "What's backed up to this stick", "Was auf diesen Stick gesichert wird";
    CoverageIntro: "These are backed up from this computer for {user}:", "Folgendes wird von diesem Computer für {user} gesichert:";
    CoverageNothing: "Nothing has been chosen for backup yet. Contact your poweruser.", "Es wurde noch nichts zum Sichern ausgewählt. Wende dich an deinen Betreuer.";
    CoverageUnreadable: "The setup on this stick can't be read. Go back and try again.", "Die Einrichtung auf diesem Stick kann nicht gelesen werden. Geh zurück und versuch es noch einmal.";
    CoverageHome: "Your home folder", "Dein persönlicher Ordner";
    CoverageMissing: "Doesn't exist any more, so nothing is backed up from it", "Existiert nicht mehr, deshalb wird daraus nichts gesichert";
    CoverageExcluded: "Not backed up, because it is left out below", "Wird nicht gesichert, weil es unten ausgelassen wird";
    CoverageOutsideHome: "Not backed up, because it is outside your home folder", "Wird nicht gesichert, weil es außerhalb deines persönlichen Ordners liegt";
    CoverageLink: "Not backed up, because it is a link, which Babawus doesn't follow", "Wird nicht gesichert, weil es eine Verknüpfung ist, der Babawus nicht folgt";
    CoverageUnreadableItem: "Can't be read, so it can't be backed up", "Kann nicht gelesen und deshalb nicht gesichert werden";
    CoverageLeftOut: "Left out:", "Ausgelassen:";
    CoveragePatterns: "Anything named {names}, in any folder", "Alles mit dem Namen {names}, in jedem Ordner";
    Either: "{first} or {last}", "{first} oder {last}";
    Loading: "Reading the stick…", "Der Stick wird gelesen …";

    FormatHeading: "Prepare a stick", "Einen Stick vorbereiten";
    FormatIntro: "Formatting erases a stick and sets it up for backups with Babawus. Choose the stick:", "Beim Formatieren wird ein Stick gelöscht und für Sicherungen mit Babawus eingerichtet. Wähle den Stick:";
    FormatNoDrives: "No stick that Babawus may format is plugged in.", "Es ist kein Stick eingesteckt, den Babawus formatieren darf.";
    FormatNoFileSystem: "No file system", "Kein Dateisystem";
    FormatNoLabel: "No name", "Kein Name";
    FormatWithBackups: "{volume}, with Babawus backups", "{volume}, mit Babawus-Sicherungen";
    FormatNotMounted: "{volume}, not mounted", "{volume}, nicht eingehängt";
    FormatLabel: "Name for the stick:", "Name für den Stick:";
    FormatLabelRule: "Use 1 to 11 letters, digits and dashes, with no spaces.", "Verwende 1 bis 11 Buchstaben, Ziffern und Bindestriche, ohne Leerzeichen.";
    FormatFileSystem: "File system:", "Dateisystem:";
    FormatExFat: "exFAT", "exFAT";
    FormatExFatDetail: "Also works on Windows and Mac", "Funktioniert auch unter Windows und auf dem Mac";
    FormatExt4: "Ext4", "Ext4";
    FormatExt4Detail: "Works on Linux only, but allows longer names and copes better with being pulled out", "Funktioniert nur unter Linux, erlaubt aber längere Namen und verträgt das Abziehen besser";
    FormatExt4AsksPassword: "Ext4 asks for your password while formatting.", "Bei Ext4 wird beim Formatieren nach deinem Passwort gefragt.";
    FormatExt4NeedsAdministrator: "Only an administrator can choose Ext4, because formatting it needs an administrator's password. Choose exFAT, or ask an administrator to prepare the stick.", "Ext4 kann nur ein Administrator wählen, weil das Formatieren damit das Passwort eines Administrators braucht. Wähle exFAT, oder bitte einen Administrator, den Stick vorzubereiten.";
    FormatWarning: "Everything on {drive} will be erased: {contents}. This includes every partition on it. Formatting can't be stopped once it has started.", "Alles auf {drive} wird gelöscht: {contents}. Das gilt für jede Partition darauf. Das Formatieren kann nicht angehalten werden, wenn es einmal begonnen hat.";
    FormatUnderstood: "I understand that everything on this stick will be erased", "Mir ist klar, dass alles auf diesem Stick gelöscht wird";
    FormatErase: "Erase and format", "Löschen und formatieren";
    FormatEraseTooltip: "Erase the stick and set it up for backups with Babawus", "Den Stick löschen und für Sicherungen mit Babawus einrichten";
    FormatWorking: "Formatting {label}…", "{label} wird formatiert …";
    FormatWorkingDetail: "This can take a few minutes. Don't pull out the stick.", "Das kann ein paar Minuten dauern. Zieh den Stick nicht ab.";
    FormatDone: "{label} is ready", "{label} ist bereit";
    FormatDoneDetail: "The stick is empty and set up for Babawus. Before anyone can back up to it, it must be set up for each computer and account that will use it.", "Der Stick ist leer und für Babawus eingerichtet. Bevor jemand darauf sichern kann, muss er für jeden Computer und jedes Konto eingerichtet werden, die ihn benutzen.";
    FormatFailed: "Formatting failed", "Formatieren fehlgeschlagen";
    FormatGone: "The stick was removed.", "Der Stick wurde abgezogen.";
    FormatNotAllowed: "Babawus may no longer format this stick, for example because it is now in use.", "Babawus darf diesen Stick nicht mehr formatieren, zum Beispiel weil er jetzt benutzt wird.";
    FormatNotPrepared: "The stick was formatted, but Babawus couldn't set it up for backups.", "Der Stick wurde formatiert, aber Babawus konnte ihn nicht für Sicherungen einrichten.";
    FormatFailedReason: "Formatting didn't work.", "Das Formatieren hat nicht geklappt.";
    FormatNothingErased: "Nothing on the stick was changed.", "Auf dem Stick wurde nichts verändert.";
    FormatPartlyErased: "The stick has been erased, but isn't ready for backups. Format it again.", "Der Stick wurde gelöscht, ist aber nicht für Sicherungen bereit. Formatiere ihn noch einmal.";
    FormatMessage: "The system said: {message}", "Meldung des Systems: {message}";
    AddComputer: "Add this computer and account…", "Diesen Computer und dein Konto hinzufügen …";
    AddComputerTooltip: "Set up this stick for you on this computer", "Diesen Stick auf diesem Computer für dich einrichten";
    TryAgain: "Try again", "Noch einmal versuchen";
    TryAgainTooltip: "Go back to choosing the stick and how to format it", "Zurück zur Auswahl des Sticks und der Art der Formatierung";

    SetupHeading: "Setup of this stick", "Einrichtung dieses Sticks";
    SetupLoading: "Reading the setup…", "Die Einrichtung wird gelesen …";
    SetupDamaged: "The setup on this stick was changed outside Babawus, or is damaged. Check it, then save it.", "Die Einrichtung auf diesem Stick wurde außerhalb von Babawus geändert oder ist beschädigt. Prüfe sie und speichere sie dann.";
    SetupUnreadable: "The setup on this stick can't be read at all. You can start a new setup. The backups on the stick are kept, and their folders stay in use.", "Die Einrichtung auf diesem Stick kann gar nicht gelesen werden. Du kannst eine neue Einrichtung beginnen. Die Sicherungen auf dem Stick bleiben erhalten, und ihre Ordner bleiben belegt.";
    SetupStartNew: "Start a new setup…", "Neue Einrichtung beginnen …";
    SetupStartNewQuestion: "Start a new setup?", "Neue Einrichtung beginnen?";
    SetupStartNewDetail: "The old setup is replaced when you save. The backups on the stick are kept.", "Die alte Einrichtung wird beim Speichern ersetzt. Die Sicherungen auf dem Stick bleiben erhalten.";
    SetupNewerFormat: "The setup on this stick needs a newer Babawus. It can't be changed here.", "Die Einrichtung auf diesem Stick braucht ein neueres Babawus. Sie kann hier nicht geändert werden.";
    SetupNotBabawus: "This stick has no Babawus setup any more.", "Dieser Stick hat keine Babawus-Einrichtung mehr.";
    SetupOlderVersions: "Older versions to keep of each file:", "Ältere Versionen pro Datei:";
    SetupOlderVersionsDetail: "When a file changes, its earlier copies are kept, up to this many. Fewer leaves more space on the stick.", "Wenn sich eine Datei ändert, werden ihre früheren Fassungen aufbewahrt, bis zu dieser Anzahl. Weniger lässt mehr Platz auf dem Stick.";
    SetupHere: "You on this computer", "Du auf diesem Computer";
    SetupNoComputerId: "This computer has no ID, so it can't be set up. Contact whoever looks after this computer.", "Dieser Computer hat keine Kennung und kann deshalb nicht eingerichtet werden. Wende dich an die Person, die diesen Computer betreut.";
    SetupComputerMissing: "This computer ({hostname}) isn't set up on this stick yet.", "Dieser Computer ({hostname}) ist auf diesem Stick noch nicht eingerichtet.";
    SetupComputerMissingLink: "This computer ({hostname}) isn't set up on this stick yet. Add it, or, if it was set up before under another entry, for example before a reinstall, link it to that entry.", "Dieser Computer ({hostname}) ist auf diesem Stick noch nicht eingerichtet. Füge ihn hinzu, oder verknüpfe ihn mit seinem früheren Eintrag, wenn er schon einmal unter einem anderen eingerichtet war, zum Beispiel vor einer Neuinstallation.";
    SetupAccountMissing: "Your account, {user}, isn't set up on this computer ({folder}) yet.", "Dein Konto, {user}, ist auf diesem Computer ({folder}) noch nicht eingerichtet.";
    SetupAccountMissingLink: "Your account, {user}, isn't set up on this computer ({folder}) yet. Add it, or, if your user name has changed, link it to your earlier name.", "Dein Konto, {user}, ist auf diesem Computer ({folder}) noch nicht eingerichtet. Füge es hinzu, oder verknüpfe es mit deinem früheren Namen, wenn sich dein Benutzername geändert hat.";
    SetupHereReady: "Your account, {user}, is set up on this computer ({folder}).", "Dein Konto, {user}, ist auf diesem Computer ({folder}) eingerichtet.";
    SetupAddComputer: "Add this computer as {folder}", "Diesen Computer als {folder} hinzufügen";
    SetupLinkComputer: "Link to {folder}…", "Mit {folder} verknüpfen …";
    SetupAddAccount: "Add {user}", "{user} hinzufügen";
    SetupLinkAccount: "Link to {old}…", "Mit {old} verknüpfen …";
    SetupChoose: "Choose what to back up…", "Auswählen, was gesichert wird …";
    SetupChooseTooltip: "Choose which of your files and folders are backed up to this stick", "Auswählen, welche deiner Dateien und Ordner auf diesen Stick gesichert werden";
    SetupEntries: "Computers and accounts on this stick", "Computer und Konten auf diesem Stick";
    SetupEmpty: "No computers or accounts are set up on this stick yet.", "Auf diesem Stick sind noch keine Computer oder Konten eingerichtet.";
    SetupThisComputer: "{folder} (this computer)", "{folder} (dieser Computer)";
    SetupOtherComputer: "{folder} · last seen as {hostname}", "{folder} · zuletzt gesehen als {hostname}";
    SetupYou: "{user} (you)", "{user} (du)";
    SetupNothingChosen: "nothing chosen yet", "noch nichts ausgewählt";
    SetupOneChosen: "1 file or folder chosen", "1 Datei oder Ordner ausgewählt";
    SetupChosen: "{count} files or folders chosen", "{count} Dateien oder Ordner ausgewählt";
    SetupRemove: "Remove…", "Entfernen …";
    SetupRemoveComputerTooltip: "Take this computer and its accounts out of the setup", "Diesen Computer und seine Konten aus der Einrichtung nehmen";
    SetupRemoveAccountTooltip: "Take this account out of the setup", "Dieses Konto aus der Einrichtung nehmen";
    SetupRemoveComputerQuestion: "Remove {folder} and its accounts from the setup?", "{folder} und seine Konten aus der Einrichtung entfernen?";
    SetupRemoveAccountQuestion: "Remove {user} on {folder} from the setup?", "{user} auf {folder} aus der Einrichtung entfernen?";
    SetupRemoveDetail: "Its backups stay on the stick, and you can delete them by hand if you want the space back. Nothing changes until you save.", "Die Sicherungen bleiben auf dem Stick, und du kannst sie von Hand löschen, wenn du den Platz brauchst. Nichts ändert sich, bevor du speicherst.";
    SetupRemoveConfirm: "Remove", "Entfernen";
    SetupLinkComputerQuestion: "Link this computer to {folder}?", "Diesen Computer mit {folder} verknüpfen?";
    SetupLinkComputerDetail: "Do this only if {folder} was this computer before, for example before a reinstall. Its backups then go on in the same folder. Nothing changes until you save.", "Tu das nur, wenn {folder} früher dieser Computer war, zum Beispiel vor einer Neuinstallation. Seine Sicherungen gehen dann im selben Ordner weiter. Nichts ändert sich, bevor du speicherst.";
    SetupLinkAccountQuestion: "Link your account to {old}?", "Dein Konto mit {old} verknüpfen?";
    SetupLinkAccountDetail: "Do this only if {old} was your user name before. Its backups and history are renamed to {user}, and the setup is saved now.", "Tu das nur, wenn {old} früher dein Benutzername war. Seine Sicherungen und sein Verlauf werden in {user} umbenannt, und die Einrichtung wird sofort gespeichert.";
    SetupLinkConfirm: "Link", "Verknüpfen";
    SetupSave: "Save", "Speichern";
    SetupSaveTooltip: "Write the changes to the stick", "Die Änderungen auf den Stick schreiben";
    SetupSaved: "Saved.", "Gespeichert.";
    SetupSaving: "Saving…", "Wird gespeichert …";
    SetupDiscardQuestion: "Leave without saving?", "Ohne Speichern verlassen?";
    SetupDiscardDetail: "Your changes to the setup of this stick will be lost.", "Deine Änderungen an der Einrichtung dieses Sticks gehen verloren.";
    SetupDiscard: "Leave without saving", "Ohne Speichern verlassen";
    SetupKeepEditing: "Keep editing", "Weiter bearbeiten";
    SetupBusy: "The stick is in use by a backup for {user}. Wait until it has finished, then save again.", "Der Stick wird gerade von einer Sicherung für {user} benutzt. Warte, bis sie fertig ist, und speichere dann noch einmal.";
    SetupInvalid: "That change can't be made, because it clashes with another entry.", "Diese Änderung ist nicht möglich, weil sie mit einem anderen Eintrag kollidiert.";
    SetupBothFolders: "Both your earlier and your new name have a folder on the stick, so Babawus can't link them. Your poweruser has to sort this out by hand.", "Sowohl dein früherer als auch dein neuer Name haben einen Ordner auf dem Stick, deshalb kann Babawus sie nicht verknüpfen. Dein Betreuer muss das von Hand klären.";
    SetupSaveFailed: "The setup couldn't be saved: {error}", "Die Einrichtung konnte nicht gespeichert werden: {error}";

    ChooseHeading: "What's backed up for {user}", "Was für {user} gesichert wird";
    ChooseIntro: "Tick the files and folders to back up. A ticked folder is backed up with everything in it. To leave out a folder inside it, untick that folder.", "Setz einen Haken bei den Dateien und Ordnern, die gesichert werden sollen. Ein Ordner mit Haken wird mit allem darin gesichert. Um einen Ordner darin auszulassen, entferne seinen Haken.";
    ChooseShowHidden: "Show hidden files", "Versteckte Dateien anzeigen";
    ChooseNoteLeftOut: "Left out", "Ausgelassen";
    ChooseNoteLeftOutAbove: "Left out with a folder above it", "Mit einem Ordner darüber ausgelassen";
    ChooseNotePattern: "Left out by the name pattern {pattern}", "Durch das Namensmuster {pattern} ausgelassen";
    ChooseNoteOtherDrive: "On another drive, so it is only backed up if ticked on its own", "Auf einem anderen Laufwerk, wird deshalb nur gesichert, wenn es selbst einen Haken hat";
    ChooseNoteLink: "A link, which Babawus doesn't follow", "Eine Verknüpfung, der Babawus nicht folgt";
    ChooseNoteNotText: "Its name isn't valid text, so it can't be chosen on its own", "Sein Name ist kein gültiger Text, deshalb kann es nicht einzeln ausgewählt werden";
    ChooseUnreadable: "This folder can't be read.", "Dieser Ordner kann nicht gelesen werden.";
    ChoosePatterns: "Leave out anything named:", "Alles mit diesem Namen auslassen:";
    ChoosePatternsDetail: "In any folder. * stands for any characters, and ? for one character.", "In jedem Ordner. * steht für beliebige Zeichen und ? für ein einzelnes Zeichen.";
    ChoosePatternAdd: "Add", "Hinzufügen";
    ChoosePatternRemove: "Remove", "Entfernen";
    ChoosePatternEmpty: "Type a name or a pattern first.", "Gib zuerst einen Namen oder ein Muster ein.";
    ChoosePatternSlash: "A pattern can't contain /. To leave out one folder, untick it in the tree.", "Ein Muster darf kein / enthalten. Um einen einzelnen Ordner auszulassen, entferne seinen Haken im Baum.";
    ChoosePatternDuplicate: "That pattern is in the list already.", "Dieses Muster steht schon in der Liste.";
    ChooseCounting: "Counting…", "Wird gezählt …";
    ChooseNothing: "Nothing is chosen yet.", "Noch nichts ausgewählt.";
    ChooseTotals: "Chosen: {files} ({size}) · {free} free on the stick", "Ausgewählt: {files} ({size}) · {free} frei auf dem Stick";
    ChooseTotalsNoFree: "Chosen: {files} ({size})", "Ausgewählt: {files} ({size})";
    ChooseTooBig: "That is more than the free space on the stick, unless much of it is backed up there already.", "Das ist mehr als der freie Platz auf dem Stick, es sei denn, ein großer Teil davon ist dort schon gesichert.";
    ChooseDone: "Done", "Fertig";
    ChooseDoneTooltip: "Go back to the setup. The changes are written when you save there.", "Zurück zur Einrichtung. Die Änderungen werden geschrieben, wenn du dort speicherst.";
    ChooseCancelTooltip: "Go back to the setup without these changes", "Ohne diese Änderungen zurück zur Einrichtung";

    Bytes: "{size} bytes", "{size} Bytes";
    Kilobytes: "{size} kB", "{size} kB";
    Megabytes: "{size} MB", "{size} MB";
    Gigabytes: "{size} GB", "{size} GB";
    Terabytes: "{size} TB", "{size} TB";
}

/// The languages whose texts are complete. The tests check them.
pub const COMPLETE: &[Language] = &[Language::English, Language::German];

impl Text {
    /// The text in `language`. A text that has not been translated yet is shown in English.
    pub fn get(self, language: Language) -> &'static str {
        let translations = self.translations();
        let text = translations[index(language)];
        if text.is_empty() {
            translations[index(Language::English)]
        } else {
            text
        }
    }

    /// The text in `language`, with each placeholder `{name}` in `values` replaced.
    pub fn fill(self, language: Language, values: &[(&str, &str)]) -> String {
        values
            .iter()
            .fold(self.get(language).to_owned(), |text, (name, value)| {
                text.replace(&format!("{{{name}}}"), value)
            })
    }
}

fn index(language: Language) -> usize {
    Language::ALL
        .iter()
        .position(|&each| each == language)
        .expect("every language is in Language::ALL")
}

/// A size in bytes, as on the packaging of a USB stick: in powers of 1000, with one decimal, for
/// example "15.5 GB".
pub fn size(bytes: u64, language: Language) -> String {
    const UNITS: [Text; 4] = [
        Text::Kilobytes,
        Text::Megabytes,
        Text::Gigabytes,
        Text::Terabytes,
    ];
    if bytes < 1000 {
        return Text::Bytes.fill(language, &[("size", &bytes.to_string())]);
    }
    let mut value = bytes as f64 / 1000.0;
    let mut unit = 0;
    // Rounding to one decimal can reach 1000, which then reads better in the next unit.
    while value >= 999.95 && unit + 1 < UNITS.len() {
        value /= 1000.0;
        unit += 1;
    }
    let number = format!("{value:.1}");
    let number = match language {
        Language::English => number,
        Language::German => number.replace('.', ","),
    };
    UNITS[unit].fill(language, &[("size", &number)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placeholders(text: &str) -> Vec<&str> {
        let mut found: Vec<&str> = text
            .split('{')
            .skip(1)
            .filter_map(|rest| rest.split_once('}').map(|(name, _)| name))
            .collect();
        found.sort_unstable();
        found
    }

    #[test]
    fn every_text_exists_in_every_complete_language() {
        for &language in COMPLETE {
            for &text in Text::ALL {
                let translated = text.translations()[index(language)];
                assert!(!translated.is_empty(), "{text:?} has no {language:?}");
            }
        }
    }

    #[test]
    fn every_translation_has_the_placeholders_of_the_english_text() {
        for &text in Text::ALL {
            let english = placeholders(text.get(Language::English));
            for language in Language::ALL {
                assert_eq!(
                    placeholders(text.get(language)),
                    english,
                    "{text:?} in {language:?}"
                );
            }
        }
    }

    #[test]
    fn a_missing_translation_falls_back_to_english() {
        let text = Text::ALL
            .iter()
            .find(|text| text.translations()[index(Language::German)].is_empty());
        if let Some(text) = text {
            assert_eq!(text.get(Language::German), text.get(Language::English));
        }
    }

    #[test]
    fn placeholders_are_filled() {
        let text = Text::InProgress.fill(Language::English, &[("user", "anna")]);
        assert_eq!(text, "Backup in progress (by anna)");
    }

    #[test]
    fn sizes_are_in_powers_of_1000_with_one_decimal() {
        let en = Language::English;
        assert_eq!(size(0, en), "0 bytes");
        assert_eq!(size(999, en), "999 bytes");
        assert_eq!(size(1000, en), "1.0 kB");
        assert_eq!(size(15_496_000_000, en), "15.5 GB");
        assert_eq!(size(999_960_000, en), "1.0 GB");
        assert_eq!(size(64_000_000_000, en), "64.0 GB");
        assert_eq!(size(2_000_000_000_000_000, en), "2000.0 TB");
        assert_eq!(size(15_496_000_000, Language::German), "15,5 GB");
    }
}
