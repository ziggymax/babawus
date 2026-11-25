#!/usr/bin/env python3
"""Drives Babawus's test VMs from the host, the same way for every guest (T-126, TestSetup.md).

Two channels, both below or beside the guest, so that they work on GNOME on Wayland too:
- libvirt, in the user's own session: defining a guest, starting and stopping it, snapshots,
  screenshots, clicks and keys (QEMU's input-send-event and send-key), and plugging in sticks;
- SSH, to the port forwarded to each guest: commands.

Only the standard library and the distribution's python3-libvirt are used (issue 170). A guest is
named by its file in vms/, such as ubuntu-26.04 for vms/ubuntu-26.04.xml; its libvirt name has
"babawus-" in front. Run with --help for the commands.
"""

import argparse
import json
import os
import struct
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET
from pathlib import Path

import libvirt
import libvirt_qemu

URI = "qemu:///session"
HERE = Path(__file__).resolve().parent
# Where libvirt's user session keeps disks (issue 170): the guests' disks, their installer ISOs
# and the stick images.
IMAGES = Path.home() / ".local/share/libvirt/images"
KNOWN_HOSTS = Path.home() / ".local/share/libvirt/babawus-known_hosts"
METADATA_NS = "https://github.com/ziggymax/babawus/vms"
DISK_SIZE = "25G"
# QEMU's absolute pointer coordinates run from 0 to this, whatever the screen size.
ABS_MAX = 0x7FFF

# Each character typed, as the QEMU key codes pressed together for it, on a US keyboard layout,
# which every guest is installed with.
_SHIFTED = {
    "!": "1", "@": "2", "#": "3", "$": "4", "%": "5", "^": "6", "&": "7", "*": "8",
    "(": "9", ")": "0", "_": "minus", "+": "equal", "{": "bracket_left",
    "}": "bracket_right", "|": "backslash", ":": "semicolon", '"': "apostrophe",
    "~": "grave_accent", "<": "comma", ">": "dot", "?": "slash",
}
_PLAIN = {
    " ": "spc", "\n": "ret", "\t": "tab", "-": "minus", "=": "equal", "[": "bracket_left",
    "]": "bracket_right", "\\": "backslash", ";": "semicolon", "'": "apostrophe",
    "`": "grave_accent", ",": "comma", ".": "dot", "/": "slash",
}
# Other names accepted by the key command, besides QEMU's own key codes.
_KEY_NAMES = {"enter": "ret", "return": "ret", "space": "spc", "escape": "esc",
              "super": "meta_l", "win": "meta_l", "del": "delete", "pageup": "pgup",
              "pagedown": "pgdn"}


def fail(message):
    print(message, file=sys.stderr)
    sys.exit(1)


def connect():
    try:
        return libvirt.open(URI)
    except libvirt.libvirtError as error:
        fail(f"Can't connect to libvirt's user session: {error}")


def domain(conn, guest):
    try:
        return conn.lookupByName(f"babawus-{guest}")
    except libvirt.libvirtError:
        fail(f"No VM babawus-{guest}; define it first with: vms/driver.py define {guest}")


def ssh_settings(dom):
    """The port on the host that is forwarded to the guest's SSH, and the user to log in as."""
    text = dom.metadata(libvirt.VIR_DOMAIN_METADATA_ELEMENT, METADATA_NS)
    ssh = ET.fromstring(text).find("{*}ssh")
    return ssh.get("port"), ssh.get("user")


def qmp(dom, command, **arguments):
    """Sends a command to the guest's QEMU, and returns what it answers."""
    message = {"execute": command}
    if arguments:
        message["arguments"] = arguments
    try:
        reply = libvirt_qemu.qemuMonitorCommand(dom, json.dumps(message), 0)
    except libvirt.libvirtError as error:
        fail(f"QEMU refused {command}: {error}")
    return json.loads(reply).get("return")


def require_running(dom):
    if not dom.isActive():
        fail(f"{dom.name()} isn't running; start it with: vms/driver.py start")


# Defining, starting and stopping

def cmd_define(conn, args):
    template = HERE / f"{args.guest}.xml"
    if not template.exists():
        fail(f"No {template}")
    xml = template.read_text().replace("@IMAGES@", str(IMAGES))
    disk = IMAGES / f"babawus-{args.guest}.qcow2"
    if not disk.exists():
        IMAGES.mkdir(parents=True, exist_ok=True)
        subprocess.run(["qemu-img", "create", "-q", "-f", "qcow2", str(disk), DISK_SIZE],
                       check=True)
        print(f"Created {disk} ({DISK_SIZE})")
    # Defining it again, after the template changed, keeps the guest, its disk and its snapshots.
    try:
        existing = conn.lookupByName(f"babawus-{args.guest}")
    except libvirt.libvirtError:
        existing = None
    if existing is not None:
        xml = xml.replace("<domain type=\"kvm\">",
                          f"<domain type=\"kvm\">\n  <uuid>{existing.UUIDString()}</uuid>", 1)
    conn.defineXML(xml)
    print(f"Defined babawus-{args.guest}")


def hmp(dom, command):
    """Sends a command to the guest's QEMU in its human monitor language, for the few commands
    that have no QMP form."""
    return libvirt_qemu.qemuMonitorCommand(
        dom, command, libvirt_qemu.VIR_DOMAIN_QEMU_MONITOR_COMMAND_HMP)


def forward_ssh(dom):
    """Forwards 127.0.0.1:<ssh port> on the host to the guest's SSH, unless it is already.
    libvirt forwards ports only for passt, which can't be used in the user session on Ubuntu
    26.04: its AppArmor profile lets nobody stop it, so it outlives its guest and keeps the port
    (TestSetup.md, "The VMs"). QEMU's own network forgets the forward when QEMU stops, so it is
    added again each time the guest starts."""
    port, _user = ssh_settings(dom)
    # QEMU lists a forward as columns, such as "TCP[HOST_FORWARD]  30  127.0.0.1  2601  10.0.2.15  22".
    for line in hmp(dom, "info usernet").splitlines():
        fields = line.split()
        if fields and fields[0] == "TCP[HOST_FORWARD]" and fields[2:4] == ["127.0.0.1", port]:
            return
    reply = hmp(dom, f"hostfwd_add tcp:127.0.0.1:{port}-:22").strip()
    if reply:
        fail(f"Couldn't forward port {port} to the guest's SSH: {reply}")


def cmd_start(conn, args):
    dom = domain(conn, args.guest)
    if not dom.isActive():
        dom.create()
    forward_ssh(dom)
    print(f"VNC: {vnc_address(dom)}")


def cmd_shutdown(conn, args):
    dom = domain(conn, args.guest)
    if dom.isActive():
        dom.shutdown()
        for _ in range(120):
            if not dom.isActive():
                return
            time.sleep(1)
        fail(f"{dom.name()} didn't shut down within 2 minutes; use destroy to pull its plug")


def cmd_destroy(conn, args):
    dom = domain(conn, args.guest)
    if dom.isActive():
        dom.destroy()


def cmd_eject(conn, args):
    """Takes the installer's ISO out of the guest's drive, once it is installed."""
    dom = domain(conn, args.guest)
    root = ET.fromstring(dom.XMLDesc(libvirt.VIR_DOMAIN_XML_INACTIVE))
    for disk in root.iter("disk"):
        if disk.get("device") == "cdrom":
            source = disk.find("source")
            if source is not None:
                disk.remove(source)
            flags = libvirt.VIR_DOMAIN_AFFECT_CONFIG
            if dom.isActive():
                flags |= libvirt.VIR_DOMAIN_AFFECT_LIVE
            dom.updateDeviceFlags(ET.tostring(disk, encoding="unicode"), flags)
    print("Ejected")


def vnc_address(dom):
    root = ET.fromstring(dom.XMLDesc(0))
    graphics = root.find("devices/graphics[@type='vnc']")
    return f"{graphics.get('listen')}:{graphics.get('port')}"


def cmd_list(conn, args):
    for dom in conn.listAllDomains():
        if dom.name().startswith("babawus-"):
            state = "running" if dom.isActive() else "shut off"
            port, user = ssh_settings(dom)
            snapshots = ", ".join(dom.snapshotListNames()) or "none"
            print(f"{dom.name()[len('babawus-'):]}: {state}; ssh {user}@127.0.0.1 -p {port}; "
                  f"snapshots: {snapshots}")


# Snapshots

def cmd_snapshot(conn, args):
    """Saves the guest as it is, running or not, under the name; a running guest's memory too,
    so that a revert returns to the same desktop."""
    dom = domain(conn, args.guest)
    try:
        dom.snapshotLookupByName(args.name).delete()
    except libvirt.libvirtError:
        pass
    dom.snapshotCreateXML(f"<domainsnapshot><name>{args.name}</name></domainsnapshot>")
    print(f"Saved snapshot {args.name}")


def cmd_revert(conn, args):
    dom = domain(conn, args.guest)
    try:
        snapshot = dom.snapshotLookupByName(args.name)
    except libvirt.libvirtError:
        fail(f"{dom.name()} has no snapshot {args.name}")
    dom.revertToSnapshot(snapshot, libvirt.VIR_DOMAIN_SNAPSHOT_REVERT_RUNNING)
    forward_ssh(dom)
    print(f"Reverted to {args.name}")


# Screen, pointer and keyboard

def screendump(dom, path):
    """Writes the guest's screen to path as a PNG, and returns its width and height."""
    require_running(dom)
    qmp(dom, "screendump", filename=str(path), format="png")
    with open(path, "rb") as png:
        header = png.read(24)
    return struct.unpack(">II", header[16:24])


def cmd_screenshot(conn, args):
    dom = domain(conn, args.guest)
    path = Path(args.file).resolve()
    width, height = screendump(dom, path)
    print(f"{path} ({width}x{height})")


def screen_size(dom):
    with tempfile.TemporaryDirectory() as folder:
        return screendump(dom, Path(folder) / "screen.png")


def move(dom, x, y):
    width, height = screen_size(dom)
    if not (0 <= x < width and 0 <= y < height):
        fail(f"{x},{y} is outside the screen, which is {width}x{height}")
    qmp(dom, "input-send-event", events=[
        {"type": "abs", "data": {"axis": "x", "value": round(x * ABS_MAX / (width - 1))}},
        {"type": "abs", "data": {"axis": "y", "value": round(y * ABS_MAX / (height - 1))}},
    ])


def button(dom, name, down):
    qmp(dom, "input-send-event",
        events=[{"type": "btn", "data": {"down": down, "button": name}}])


def cmd_move(conn, args):
    move(domain(conn, args.guest), args.x, args.y)


def cmd_click(conn, args):
    dom = domain(conn, args.guest)
    move(dom, args.x, args.y)
    for _ in range(2 if args.double else 1):
        button(dom, args.button, True)
        time.sleep(0.05)
        button(dom, args.button, False)
        time.sleep(0.05)


def cmd_drag(conn, args):
    dom = domain(conn, args.guest)
    move(dom, args.x1, args.y1)
    button(dom, "left", True)
    time.sleep(0.2)
    move(dom, args.x2, args.y2)
    time.sleep(0.2)
    button(dom, "left", False)


HOLD_MS = 30


def press(dom, codes):
    """Presses the keys together, then lets them go, and returns once they are let go: QEMU
    answers send-key at once, so without the wait, keys would pile up and a screenshot right
    after typing would show only part of the text."""
    qmp(dom, "send-key", keys=[{"type": "qcode", "data": code} for code in codes],
        **{"hold-time": HOLD_MS})
    time.sleep(HOLD_MS / 1000 + 0.03)


def key_codes(char):
    if char.isascii() and (char.isalnum()):
        return ["shift", char.lower()] if char.isupper() else [char]
    if char in _PLAIN:
        return [_PLAIN[char]]
    if char in _SHIFTED:
        return ["shift", _SHIFTED[char]]
    fail(f"Can't type {char!r}: only the characters of a US keyboard")


def cmd_type(conn, args):
    dom = domain(conn, args.guest)
    require_running(dom)
    codes = [key_codes(char) for char in args.text]
    for code in codes:
        press(dom, code)


def cmd_key(conn, args):
    """Presses each combination in turn, such as ctrl+alt+t, then enter."""
    dom = domain(conn, args.guest)
    require_running(dom)
    for combination in args.keys:
        press(dom, [_KEY_NAMES.get(part.lower(), part.lower())
                    for part in combination.split("+")])
        time.sleep(0.1)


# Commands over SSH

def ssh_command(dom):
    port, user = ssh_settings(dom)
    KNOWN_HOSTS.parent.mkdir(parents=True, exist_ok=True)
    return ["ssh", "-p", port, "-o", "BatchMode=yes", "-o", "ConnectTimeout=5",
            "-o", "StrictHostKeyChecking=accept-new",
            "-o", f"UserKnownHostsFile={KNOWN_HOSTS}", f"{user}@127.0.0.1"]


def cmd_run(conn, args):
    dom = domain(conn, args.guest)
    require_running(dom)
    result = subprocess.run(ssh_command(dom) + ["--", *args.command])
    sys.exit(result.returncode)


def cmd_launch(conn, args):
    """Starts a program in the desktop session of the user who is logged in, where its window
    appears. A command run over SSH is not part of that session, but the user's systemd is, and
    the desktop gives it WAYLAND_DISPLAY or DISPLAY. The program runs as a unit of its own, named
    after the program, which `run <guest> systemctl --user stop <name>` stops."""
    dom = domain(conn, args.guest)
    require_running(dom)
    unit = f"driver-{Path(args.command[0]).name}"
    result = subprocess.run(ssh_command(dom) + [
        "--", "systemd-run", "--user", "--collect", "--quiet", f"--unit={unit}",
        *args.command])
    if result.returncode == 0:
        print(f"Started as {unit}")
    sys.exit(result.returncode)


def cmd_copy(conn, args):
    """Copies a file from the host into the guest's home folder, or the folder given."""
    dom = domain(conn, args.guest)
    require_running(dom)
    ssh = ssh_command(dom)
    port, target = ssh[2], ssh[-1]
    options = ssh[3:-1]
    result = subprocess.run(["scp", "-q", "-P", port, *options, args.file,
                             f"{target}:{args.to}"])
    sys.exit(result.returncode)


def cmd_wait_ssh(conn, args):
    """Waits until the guest answers on SSH, such as after it is started."""
    dom = domain(conn, args.guest)
    require_running(dom)
    deadline = time.monotonic() + args.timeout
    while time.monotonic() < deadline:
        result = subprocess.run(ssh_command(dom) + ["--", "true"],
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if result.returncode == 0:
            return
        time.sleep(2)
    fail(f"{dom.name()} didn't answer on SSH within {args.timeout} s")


# Sticks

def stick_path(name):
    path = Path(name)
    return path if path.is_absolute() or path.parent != Path(".") else IMAGES / name


def cmd_stick(conn, args):
    """Makes an empty stick image, which Babawus can then format, or a file system can be put on
    with mkfs."""
    path = stick_path(args.file)
    if path.exists():
        fail(f"{path} exists already")
    subprocess.run(["qemu-img", "create", "-q", "-f", "raw", str(path), args.size], check=True)
    print(f"Created {path} ({args.size})")


def stick_alias(slot):
    # libvirt keeps aliases that begin with "ua-" as given, so that the stick can be found again.
    return f"ua-stick-{slot}"


def cmd_plug(conn, args):
    """Plugs a stick image into the guest, which sees a USB stick arrive."""
    dom = domain(conn, args.guest)
    require_running(dom)
    path = stick_path(args.file)
    if not path.exists():
        fail(f"No {path}; make one with: vms/driver.py stick {args.file} 2G")
    device = f"""
        <disk type="file" device="disk">
          <driver name="qemu" type="raw"/>
          <source file="{path}"/>
          <target dev="sd{'xyz'['abc'.index(args.slot)]}" bus="usb" removable="on"/>
          <alias name="{stick_alias(args.slot)}"/>
        </disk>"""
    dom.attachDeviceFlags(device, libvirt.VIR_DOMAIN_AFFECT_LIVE)
    print(f"Plugged {path.name} into slot {args.slot}")


def cmd_unplug(conn, args):
    """Pulls the stick out, as a user would, without ejecting it first."""
    dom = domain(conn, args.guest)
    require_running(dom)
    try:
        dom.detachDeviceAlias(stick_alias(args.slot), libvirt.VIR_DOMAIN_AFFECT_LIVE)
    except libvirt.libvirtError as error:
        fail(f"No stick in slot {args.slot}: {error}")
    print(f"Unplugged slot {args.slot}")


def host_sticks():
    """The USB storage devices plugged into the host, as (serial, bus, device, name)."""
    sticks = []
    for device in sorted(Path("/sys/bus/usb/devices").iterdir()):
        interfaces = list(device.glob("*:*/bInterfaceClass"))
        if not any(interface.read_text().strip() == "08" for interface in interfaces):
            continue

        def read(name):
            path = device / name
            return path.read_text().strip() if path.exists() else ""

        name = " ".join(part for part in (read("manufacturer"), read("product")) if part)
        sticks.append((read("serial"), int(read("busnum")), int(read("devnum")), name))
    return sticks


def cmd_sticks(conn, args):
    for serial, bus, device, name in host_sticks():
        print(f"{serial or '(no serial)'}: {name}, bus {bus} device {device}")


REAL_ALIAS = "ua-real-stick"


def cmd_pass(conn, args):
    """Passes a real stick from the host through to the guest, which sees it arrive, while the
    host no longer has it. It needs vms/70-babawus-usb-storage.rules on the host."""
    dom = domain(conn, args.guest)
    require_running(dom)
    found = [stick for stick in host_sticks() if stick[0] == args.serial]
    if not found:
        fail(f"No USB stick with serial {args.serial} on the host; see: vms/driver.py sticks")
    _serial, bus, device, name = found[0]
    node = Path(f"/dev/bus/usb/{bus:03}/{device:03}")
    if not os.access(node, os.R_OK | os.W_OK):
        fail(f"No access to {node}: install vms/70-babawus-usb-storage.rules (DevEnvSetup.md)")
    hostdev = f"""
        <hostdev mode="subsystem" type="usb">
          <source><address bus="{bus}" device="{device}"/></source>
          <alias name="{REAL_ALIAS}"/>
        </hostdev>"""
    dom.attachDeviceFlags(hostdev, libvirt.VIR_DOMAIN_AFFECT_LIVE)
    print(f"Passed {name} ({args.serial}) through to {dom.name()}")


def cmd_unpass(conn, args):
    """Gives the real stick back to the host, as if pulled out of the guest."""
    dom = domain(conn, args.guest)
    require_running(dom)
    try:
        dom.detachDeviceAlias(REAL_ALIAS, libvirt.VIR_DOMAIN_AFFECT_LIVE)
    except libvirt.libvirtError as error:
        fail(f"No real stick passed through: {error}")
    print("Gave the stick back to the host")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    commands = parser.add_subparsers(dest="command", required=True, metavar="command")

    def command(name, function, help_text, *arguments):
        sub = commands.add_parser(name, help=help_text, description=help_text)
        sub.add_argument("guest", help="such as ubuntu-26.04, for vms/ubuntu-26.04.xml")
        for names, options in arguments:
            sub.add_argument(*names, **options)
        sub.set_defaults(function=function)

    number = {"type": int}
    command("define", cmd_define, "Define the guest in libvirt, with an empty disk if it has none")
    command("start", cmd_start, "Start the guest, and print where its VNC is")
    command("shutdown", cmd_shutdown, "Shut the guest down, and wait until it is off")
    command("destroy", cmd_destroy, "Turn the guest off at once")
    command("eject", cmd_eject, "Take the installer ISO out of the guest's drive")
    command("snapshot", cmd_snapshot, "Save the guest as it is, replacing a snapshot of that name",
            (["name"], {"nargs": "?", "default": "clean"}))
    command("revert", cmd_revert, "Return the guest to a snapshot, running",
            (["name"], {"nargs": "?", "default": "clean"}))
    command("screenshot", cmd_screenshot, "Save the guest's screen as a PNG",
            (["file"], {}))
    command("move", cmd_move, "Move the pointer to x,y on the guest's screen",
            (["x"], number), (["y"], number))
    command("click", cmd_click, "Click at x,y on the guest's screen",
            (["x"], number), (["y"], number),
            (["--button"], {"choices": ["left", "right", "middle"], "default": "left"}),
            (["--double"], {"action": "store_true"}))
    command("drag", cmd_drag, "Drag with the left button from x1,y1 to x2,y2",
            (["x1"], number), (["y1"], number), (["x2"], number), (["y2"], number))
    command("type", cmd_type, "Type the text, on a US keyboard layout", (["text"], {}))
    command("key", cmd_key, "Press keys, such as ctrl+alt+t or enter, one combination after "
            "another", (["keys"], {"nargs": "+"}))
    command("run", cmd_run, "Run a command in the guest over SSH, with its exit code",
            (["command"], {"nargs": argparse.REMAINDER}))
    command("launch", cmd_launch, "Start a program in the guest's desktop, where its window "
            "appears", (["command"], {"nargs": argparse.REMAINDER}))
    command("copy", cmd_copy,"Copy a file into the guest over SSH",
            (["file"], {}), (["to"], {"nargs": "?", "default": "."}))
    command("wait-ssh", cmd_wait_ssh, "Wait until the guest answers on SSH",
            (["--timeout"], {"type": int, "default": 300}))
    command("plug", cmd_plug, "Plug a stick image into the guest",
            (["file"], {"help": "a name in the images folder, or a path"}),
            (["--slot"], {"choices": ["a", "b", "c"], "default": "a"}))
    command("unplug", cmd_unplug, "Pull a stick out of the guest",
            (["--slot"], {"choices": ["a", "b", "c"], "default": "a"}))

    command("pass", cmd_pass, "Pass a real stick from the host through to the guest",
            (["serial"], {"help": "the stick's serial number, as the sticks command lists it"}))
    command("unpass", cmd_unpass, "Give the real stick back to the host")

    sticks = commands.add_parser("sticks", help="List the real USB sticks plugged into the host")
    sticks.set_defaults(function=cmd_sticks)
    stick = commands.add_parser("stick", help="Make an empty stick image")
    stick.add_argument("file", help="a name in the images folder, or a path")
    stick.add_argument("size", help="such as 2G")
    stick.set_defaults(function=cmd_stick)
    listing = commands.add_parser("list", help="List the test VMs")
    listing.set_defaults(function=cmd_list)

    args = parser.parse_args()
    conn = connect()
    args.function(conn, args)


if __name__ == "__main__":
    libvirt.registerErrorHandler(lambda _context, _error: None, None)
    main()
