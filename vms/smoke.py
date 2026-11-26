#!/usr/bin/env python3
"""The smoke test: tries Babawus's packages in each test VM, without Claude (T-128, TestSetup.md).

For each guest, one after another:
1. revert it to its clean snapshot, which has a desktop logged in;
2. install the package as a user would, with apt or dnf;
3. start Babawus with --debug=sticks, plug in an emulated stick, check that Babawus lists it,
   save a screenshot, pull the stick out and check that Babawus no longer lists it;
4. install the polkit rule and the tools that the tests on real file systems need, and run those
   tests, built in that distribution's container.

The tools and the rule come only after Babawus is tried, so that they can't hide a dependency
missing from a package, and the clean snapshot has neither (issue 170). Each guest is left
running as the test left it. The packages are taken from target/containers/packages/, where
containers/package.sh leaves them, or from the folder given. A log and a screenshot of each
guest go to target/smoke/.

    vms/smoke.py                      every guest
    vms/smoke.py debian-13 ...        only these
"""

import argparse
import shlex
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import driver  # noqa: E402
import libvirt  # noqa: E402

REPO = HERE.parent
OUT = REPO / "target/smoke"

# Each guest, with the container its tests are built in and the package it installs. Linux Mint
# 22 is built on Ubuntu 24.04, so it shares that container.
GUESTS = {
    "mint-22.3": ("ubuntu-24.04", "deb"),
    "ubuntu-24.04": ("ubuntu-24.04", "deb"),
    "ubuntu-26.04": ("ubuntu-26.04", "deb"),
    "debian-13": ("debian-13", "deb"),
    "fedora-45": ("fedora-45", "rpm"),
}

# How each kind of package is installed, and the tools the tests on real file systems use.
INSTALL = {
    "deb": "sudo apt-get update -qq && sudo DEBIAN_FRONTEND=noninteractive "
           "apt-get install -y -qq -o DPkg::Lock::Timeout=600 {}",
    "rpm": "sudo dnf install -y -q {}",
}
TEST_TOOLS = "exfatprogs e2fsprogs acl udisks2 util-linux"

# Lets tester use udisks2 from SSH, outside the desktop session, as CI lets its runner.
POLKIT_RULE = """polkit.addRule(function (action, subject) {
    if (action.id.indexOf("org.freedesktop.udisks2.") == 0 && subject.user == "tester") {
        return polkit.Result.YES;
    }
});
"""

STICK_LABEL = "SMOKE"
STICK_SIZE = 64 * 1024 * 1024
DEBUG_LOG = "/tmp/babawus-smoke.log"
UNIT = "smoke-babawus"


class Failed(Exception):
    pass


class Guest:
    """One guest under test, with its log."""

    def __init__(self, conn, name, log):
        self.conn = conn
        self.name = name
        self.dom = driver.domain(conn, name)
        self.log = log

    def note(self, text):
        self.log.write(text + "\n")
        self.log.flush()

    def run(self, command, check=True):
        """Runs a shell command in the guest over SSH, with its output in the log."""
        self.note(f"$ {command}")
        result = subprocess.run(driver.ssh_command(self.dom) + ["--", command],
                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        self.note(result.stdout.rstrip())
        if check and result.returncode != 0:
            raise Failed(f"{command!r} failed with exit code {result.returncode}; see the log")
        return result

    def copy(self, files, to):
        ssh = driver.ssh_command(self.dom)
        port, target, options = ssh[2], ssh[-1], ssh[3:-1]
        result = subprocess.run(["scp", "-q", "-P", port, *options, *map(str, files),
                                 f"{target}:{to}"], stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True)
        self.note(result.stdout.rstrip())
        if result.returncode != 0:
            raise Failed(f"Couldn't copy {len(files)} file(s) into the guest")

    def wait_for_ssh(self, timeout=300):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if subprocess.run(driver.ssh_command(self.dom) + ["--", "true"],
                              stdout=subprocess.DEVNULL,
                              stderr=subprocess.DEVNULL).returncode == 0:
                return
            time.sleep(2)
        raise Failed(f"It didn't answer on SSH within {timeout} s")

    def debug_lines(self):
        """The sticks lines Babawus has printed so far."""
        output = self.run(f"cat {DEBUG_LOG} 2>/dev/null || true").stdout
        return [line for line in output.splitlines()
                if line.startswith("Babawus debug sticks: ")]

    def wait_for_line(self, after, wanted, what, timeout=60):
        """Waits until Babawus prints a sticks line, after the first `after`, that `wanted`
        accepts, and returns how many lines there are then."""
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            lines = self.debug_lines()
            if any(wanted(line) for line in lines[after:]):
                return len(lines)
            time.sleep(2)
        raise Failed(f"Babawus didn't {what} within {timeout} s")


# The steps, each of which raises Failed when it fails

def revert(guest, _package, _tests):
    snapshot = guest.dom.snapshotLookupByName("clean")
    guest.dom.revertToSnapshot(snapshot, libvirt.VIR_DOMAIN_SNAPSHOT_REVERT_RUNNING)
    driver.forward_ssh(guest.dom)
    guest.wait_for_ssh()
    # The clock stood still in the snapshot, and a clock far behind makes apt refuse the package
    # lists as not valid yet.
    guest.run(f"sudo date -s @{int(time.time())}")


def install(guest, package, _tests):
    guest.copy([package], "/tmp/")
    guest.run(INSTALL[GUESTS[guest.name][1]].format(shlex.quote(f"/tmp/{package.name}")))
    guest.run("command -v babawus")


def stick(guest, _package, _tests):
    guest.run(f"pkill -x babawus; systemd-run --user --collect --quiet --unit={UNIT} "
              f"-p StandardError=truncate:{DEBUG_LOG} /usr/bin/babawus --debug=sticks",
              check=False)
    count = guest.wait_for_line(0, lambda line: True, "start and list the sticks")

    image = OUT / f"{guest.name}-stick.img"
    image.unlink(missing_ok=True)
    with open(image, "wb") as file:
        file.truncate(STICK_SIZE)
    subprocess.run(["/usr/sbin/mkfs.exfat", "-L", STICK_LABEL, str(image)], check=True,
                   stdout=subprocess.DEVNULL)
    listed = f'"{STICK_LABEL}"'
    plug(guest, image)
    try:
        count = guest.wait_for_line(count, lambda line: listed in line,
                                    f"list the stick {STICK_LABEL}")
        # The window draws a moment after the list changes.
        time.sleep(2)
        driver.screendump(guest.dom, OUT / f"{guest.name}.png")
    finally:
        guest.dom.detachDeviceAlias(driver.stick_alias("a"), libvirt.VIR_DOMAIN_AFFECT_LIVE)
    guest.wait_for_line(count, lambda line: listed not in line,
                        f"take {STICK_LABEL} off its list once it was pulled out")


def plug(guest, image):
    guest.dom.attachDeviceFlags(f"""
        <disk type="file" device="disk">
          <driver name="qemu" type="raw"/>
          <source file="{image}"/>
          <target dev="sdx" bus="usb" removable="on"/>
          <alias name="{driver.stick_alias('a')}"/>
        </disk>""", libvirt.VIR_DOMAIN_AFFECT_LIVE)


def test_tools(guest, _package, _tests):
    guest.run(INSTALL[GUESTS[guest.name][1]].format(TEST_TOOLS))
    guest.run("sudo tee /etc/polkit-1/rules.d/50-babawus-smoke.rules > /dev/null <<'EOF'\n"
              f"{POLKIT_RULE}EOF")


def real_file_systems(guest, _package, tests):
    # The tests use the folder they were built in, as CARGO_TARGET_TMPDIR.
    built_in = "/src/" + str(tests[0].relative_to(REPO)).split("/debug/")[0] + "/tmp"
    guest.run(f"rm -rf ~/smoke-tests && mkdir ~/smoke-tests && "
              f"sudo install -d -o tester {shlex.quote(built_in)}")
    guest.copy(tests, "smoke-tests/")
    failed = []
    for test in tests:
        result = guest.run(f"cd ~/smoke-tests && ./{test.name} --ignored", check=False)
        summary = [line for line in result.stdout.splitlines() if line.startswith("test result")]
        name = test.name.rsplit("-", 1)[0]
        print(f"    {name}: {summary[-1] if summary else 'no result'}", flush=True)
        if result.returncode != 0:
            failed.append(name)
    if failed:
        raise Failed(f"Failed: {', '.join(failed)}; see the log")


STEPS = [
    ("revert to the clean snapshot", revert),
    ("install the package", install),
    ("Babawus lists a stick", stick),
    ("install the test tools and polkit rule", test_tools),
    ("tests on real file systems", real_file_systems),
]


def build_tests(container):
    """Builds the tests in the container, and returns the test programs."""
    print(f"==> Building the tests in the {container} container", flush=True)
    result = subprocess.run([str(REPO / "containers/build-tests.sh"), container],
                            stdout=subprocess.PIPE, text=True)
    if result.returncode != 0:
        driver.fail(f"Couldn't build the tests in the {container} container")
    return [Path(line) for line in result.stdout.splitlines()]


def find_package(folder, kind):
    found = sorted(folder.glob(f"*.{kind}"))
    if len(found) != 1:
        driver.fail(f"Expected one .{kind} in {folder}, found {len(found)}; "
                    "build them with: containers/package.sh")
    return found[0]


def test_guest(conn, name, package, tests):
    """Runs every step on the guest, and returns the step that failed, if any."""
    with open(OUT / f"{name}.log", "w") as log:
        guest = Guest(conn, name, log)
        for title, step in STEPS:
            print(f"==> {name}: {title}", flush=True)
            guest.note(f"\n=== {title}")
            try:
                step(guest, package, tests)
            except (Failed, libvirt.libvirtError, subprocess.CalledProcessError) as error:
                print(f"==> {name}: FAILED: {error}", flush=True)
                guest.note(f"FAILED: {error}")
                return title
        print(f"==> {name}: passed", flush=True)
        return None


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("guests", nargs="*", metavar="guest",
                        help=f"the guests to test, of: {', '.join(GUESTS)}; all by default")
    parser.add_argument("--packages", type=Path, default=REPO / "target/containers/packages",
                        help="the folder with the .deb and the .rpm")
    args = parser.parse_args()
    names = args.guests or list(GUESTS)
    for name in names:
        if name not in GUESTS:
            driver.fail(f"Unknown guest {name}; the guests are: {', '.join(GUESTS)}")

    packages = {kind: find_package(args.packages.resolve(), kind)
                for kind in {GUESTS[name][1] for name in names}}
    tests = {}
    for name in names:
        container = GUESTS[name][0]
        if container not in tests:
            tests[container] = build_tests(container)

    OUT.mkdir(parents=True, exist_ok=True)
    conn = driver.connect()
    results = {}
    for name in names:
        container, kind = GUESTS[name]
        results[name] = test_guest(conn, name, packages[kind], tests[container])

    print("\nSmoke test:")
    for name, failed in results.items():
        print(f"  {name}: {'passed' if failed is None else 'FAILED at: ' + failed}")
    print(f"Logs and screenshots are in {OUT}")
    sys.exit(0 if all(failed is None for failed in results.values()) else 1)


if __name__ == "__main__":
    libvirt.registerErrorHandler(lambda _context, _error: None, None)
    main()
