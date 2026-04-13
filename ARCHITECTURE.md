# Hyprion Architecture

> Last updated: April 2026 (rev 2)
> This is a living document. Update it as decisions change.

---

## What is Hyprion?

Hyprion is a modular, Rust-native desktop environment layer built specifically for Hyprland. It exists to solve one problem: Hyprland is incredible but requires gluing together dozens of independent tools that don't know about each other, don't share a design language, and feel half-baked as a whole.

Hyprion makes Hyprland feel like a real DE — cohesive, polished, stable — without replacing what makes Hyprland great. Every component is an independent binary. Install all of them or just one. They all feel like they were made by the same hand because they were.

---

## Priorities

In order of importance:

1. **Performance** — low resource usage, no wasted CPU, smooth at all times
2. **Usefulness** — stable, comfortable, complete daily driver quality
3. **Modularity** — nothing depends on everything, install only what you want
4. **Eye candy** — visually impressive, makes people want to switch to Linux

---

## Components

| Binary                | Role                                                     | Status         |
| --------------------- | -------------------------------------------------------- | -------------- |
| `hyprion-core`        | Central daemon, state cache, event bus, IPC router       | 🚧 In progress |
| `hyprion-theme`       | Shared design token library (crate, not binary)          | 🚧 In progress |
| `hyprionctl`          | CLI wrapper — wraps hyprctl + adds Hyprion commands      | 📋 Planned     |
| `hyprion-bar`         | Status bar                                               | 📋 Planned     |
| `hyprion-notif`       | Notification server (D-Bus)                              | 📋 Planned     |
| `hyprion-osd`         | On-screen display (volume, brightness popups)            | 📋 Planned     |
| `hyprion-launcher`    | App launcher                                             | 📋 Planned     |
| `hyprion-files`       | File browser                                             | 📋 Planned     |
| `hyprion-settings`    | Settings GUI                                             | 📋 Planned     |
| `hyprion-session`     | Lock screen, user switching, power menu                  | 📋 Planned     |
| `hyprion-wallpaper`   | Wallpaper engine (static + video)                        | 📋 Planned     |
| `hyprion-screenshot`  | Screenshot tool with GUI (snip tool style)               | 📋 Planned     |
| `hyprion-greeter`     | Post-boot user picker (Wayland, full GPU)                | 📋 Planned     |
| `hyprion-crypthelper` | Privileged setuid helper for LUKS keyslot management     | 📋 Planned     |
| `hyprion-initui`      | Pre-boot framebuffer UI for LUKS unlock + user selection | 📋 Planned     |

---

## Core Architecture

### hyprion-core is a dumb router and state cache

Core does NOT know how to get volume, manage notifications, or handle any domain-specific logic. That belongs to the module that owns that domain.

What core actually does:

- **Caches** the last known state reported by each module
- **Routes** commands to the correct module
- **Broadcasts** events to all subscribers
- **Tracks** connected clients and their subscriptions
- **Bridges** Hyprland's IPC into Hyprion events

Modules own their domain completely. `hyprion-volume` knows how to talk to PipeWire — core just stores the last value it reported and tells everyone when it changes. If a module crashes and restarts, it reconnects, reports its current state, and everything continues. Nothing is lost.

### Module ownership model

Every domain is owned by exactly one module. Only the owning module can SET state for its domain. Anyone can GET cached state from core.

Example:

- `hyprion-volume` owns the `volume` domain
- It connects to PipeWire, monitors changes, and reports them to core
- When volume changes, it tells core → core broadcasts to all subscribers
- Other modules (bar, osd) can read volume from core's cache
- Other modules can send a `volume.set` command to core → core forwards it to `hyprion-volume` → `hyprion-volume` actually changes PipeWire volume → reports new value back to core

Nobody bypasses the owning module. Core never talks to PipeWire directly.

### Public commands

Modules expose commands through core that anyone can trigger. These are declared when the module connects:

```json
{
  "kind": "register",
  "domain": "volume",
  "commands": ["set", "increase", "decrease", "mute"]
}
```

Core learns what commands exist and who handles them. When a client sends a command, core validates the domain/command exists and forwards it to the right module.

### Everything talks to one socket

```
/run/user/{uid}/hyprion/
    core.sock
```

One socket. Every component — bar, notif, launcher, third party apps — connects here. Core is the single entry point. There is no module-to-module communication, everything goes through core.

### Why not module-to-module?

If bar talked directly to volume's socket and theme's socket and notif's socket, you'd have a web of connections where every module needs to know about every other module. Adding a new module means updating every existing one. Core as a hub keeps it clean — modules only need to know about core.

### Why not D-Bus for everything?

D-Bus is battle-tested and every Linux app speaks it. We DO use D-Bus where the system expects it (notification server spec, media controls). But for internal Hyprion IPC, a custom Unix socket gives us more control, lower overhead, and simpler debugging.

---

## IPC Protocol

JSON over Unix socket, newline delimited.

### Message kinds

**Register — module identifies itself and declares its domain**

```json
{
  "kind": "register",
  "domain": "volume",
  "commands": ["set", "increase", "decrease", "mute"]
}
```

**State report — module pushes its current state to core's cache**

```json
{
  "kind": "state",
  "domain": "volume",
  "payload": { "level": 75, "muted": false }
}
```

**Command — send a command to a module via core**

```json
{
  "kind": "command",
  "domain": "volume",
  "action": "set",
  "payload": { "level": 75 }
}
```

**Query — get cached state for a domain**

```json
{ "kind": "query", "domain": "volume" }
```

**Subscribe — listen for events**

```json
{ "kind": "subscribe", "events": ["volume.*", "workspace.changed"] }
```

Supports wildcard `*` per domain, or `"*"` for all events.

**Event — broadcast from core to all matching subscribers**

```json
{
  "kind": "event",
  "name": "volume.changed",
  "payload": { "level": 75, "muted": false }
}
```

### Event system

Core's event system is completely generic — it does not know or care what events mean. Events are just:

- A name (`domain.action`, e.g. `volume.changed`)
- A raw JSON payload (core forwards it blind)

Modules define their own event names. Core just pattern-matches subscriptions against event names and forwards. This means new modules can introduce new events without touching core.

### Two tiers of events

**Simple events** — state changes. Emitted automatically by core when a module reports new state:

- `volume.changed`
- `theme.changed`
- `workspace.changed`

**Complex events** — module-specific, emitted explicitly by the module:

- `notif.dismissed`
- `wallpaper.changed`
- `screenshot.taken`

Core forwards both identically. The distinction is only conceptual.

### Why JSON?

- Human readable and debuggable (you can literally `nc` into the socket and read messages)
- Zero extra tooling needed
- Performance difference vs binary formats is completely irrelevant at DE scale
- Easy for third party apps to integrate

### Why event-driven instead of polling?

Polling wastes CPU and adds latency. Event-driven means the bar updates the instant volume changes — zero delay, zero wasted cycles.

---

## hyprionctl

A CLI binary that serves as the command-line interface to Hyprion. It:

- Wraps `hyprctl` for all existing Hyprland commands (so you only need one tool)
- Adds Hyprion-specific commands on top

Examples:

```bash
hyprionctl volume set 75
hyprionctl volume increase 5
hyprionctl theme set catppuccin-mocha
hyprionctl notify "Build finished"
hyprionctl workspace 2
hyprionctl dispatch movewindow l   # wraps hyprctl dispatch
```

Under the hood it just connects to `core.sock` and sends the appropriate JSON message. Simple, scriptable, composable.

---

## Third Party API

Third party apps connect to `core.sock` exactly like any Hyprion component. No special treatment, no separate API surface. They can:

- Query any cached state
- Subscribe to any events
- Send commands to any module
- Emit their own events

The only constraint is SET operations — only the owning module can report state for its domain. Third parties can send commands which the owning module chooses to honor or reject.

---

## Design System (hyprion-theme)

All components share a single `hyprion-theme` crate that defines:

- **Color tokens** — semantic names (`background`, `surface`, `accent`, `text`, `text_muted`, `border`) not raw hex values. Components never hardcode colors.
- **Spacing scale** — base 4px unit, named sizes (`xs`, `sm`, `md`, `lg`, `xl`)
- **Typography** — font family, named sizes (`label`, `body`, `title`, `heading`)
- **Corner radius** — consistent rounding (`sm`, `md`, `lg`)

Theme is loaded from a TOML config file and deserialized into Rust structs. All components react to theme changes via core events — live theme switching with no restarts needed.

Default theme is dark. Light theme supported but not the focus.

---

## Boot Process

### Goals

- No double password entry ever
- Boot as fast as possible
- No black screens
- Fancy user picker
- Full disk encryption per user

### The problem

A fancy user picker requires Wayland and GPU acceleration. LUKS decryption happens in initramfs before the system boots — no Wayland available. These two requirements are in conflict.

### The solution

Split into two phases:

**Phase 1 — initramfs (hyprion-initui)**

A custom Rust framebuffer application built into the initramfs. Runs before anything boots. Uses direct framebuffer rendering (no Wayland, no GPU acceleration, but can still look clean and intentional with careful design).

Reads user metadata from `/boot/hyprion/` (unencrypted, accessible before LUKS unlock):

```
/boot/hyprion/
    users.toml          ← display names, usernames
    avatars/
        sergio.png      ← profile pictures (optimized for framebuffer)
```

Flow:

1. Plymouth splash animation plays during early boot
2. hyprion-initui appears — shows user list with avatars
3. User selects their account, enters password
4. Password is tried against their LUKS keyslot
5. Success → system boots fully, user identity passed to session
6. Failure → back to password prompt

**Phase 2 — post-boot (hyprion-greeter + hyprion-session)**

Full Wayland, full GPU, full Hyprion design language. Used for:

- Lock screen (most common interaction)
- User switching
- Session switching

Since the user already authenticated at LUKS time, the greeter on first boot after LUKS unlock can auto-login. Lock screen requires password re-entry as normal.

### User metadata sync

When a user changes their avatar or display name in `hyprion-settings`, it automatically syncs to `/boot/hyprion/`. Since `/boot` is unencrypted this is accessible at initramfs time.

Security note: usernames and avatars in `/boot/hyprion/` are visible to anyone with physical access to the machine. This is an inherent tradeoff of the pre-decryption user picker. Document this clearly for users.

---

## LUKS Multi-User Encryption

### How it works

LUKS2 supports up to 32 keyslots. Each keyslot can decrypt the volume independently. Hyprion assigns one keyslot per user:

```
LUKS volume
├── keyslot 0 → encrypted with user1's password
├── keyslot 1 → encrypted with user2's password
└── keyslot 2 → encrypted with user3's password
```

Each user's password decrypts their keyslot which contains the master volume key. From the user's perspective they just have one password. LUKS handles the rest.

### hyprion-crypthelper

Changing LUKS keyslots requires root. Rather than using sudo or PolicyKit, Hyprion ships a tiny setuid binary — `hyprion-crypthelper` — that does exactly one thing: re-encrypt a LUKS keyslot given an old and new password.

It verifies the old password actually decrypts the correct keyslot before doing anything, preventing users from touching other users' keyslots. Under the hood it calls `cryptsetup luksChangeKey`.

### Password change flow

When a user changes their password in `hyprion-settings`:

```
user enters old password + new password
    ↓
hyprion-settings sends both to hyprion-crypthelper
    ↓
crypthelper verifies old password decrypts their keyslot
    ↓
crypthelper re-encrypts keyslot with new password (cryptsetup luksChangeKey)
    ↓
crypthelper updates system login password (passwd)
    ↓
done — one action, both updated atomically
```

No sudo prompt. No PolicyKit popup. Just a normal password change dialog.

### Adding/removing users

When a new user is created in `hyprion-settings`:

1. System user account created
2. New LUKS keyslot added for their password
3. Their metadata added to `/boot/hyprion/users.toml`
4. Their avatar added to `/boot/hyprion/avatars/`

When a user is deleted, the reverse happens — keyslot removed, metadata cleaned up.

---

## Existing Tools We Rely On

We don't reinvent things that already work well:

| Tool           | Why                                                         |
| -------------- | ----------------------------------------------------------- |
| Hyprland       | The compositor, we build on top of it                       |
| PipeWire       | Audio routing, we talk to it for volume control             |
| NetworkManager | Network management, we talk to it for network status        |
| wl-clipboard   | Clipboard protocol, we enhance it rather than replace it    |
| Plymouth       | Boot splash animation, covers early boot black screen       |
| D-Bus          | Notification spec, media controls (where system expects it) |
| cryptsetup     | LUKS operations via hyprion-crypthelper                     |
| loginctl       | Session management in hyprion-session                       |

---

## Tech Stack

| Purpose               | Crate                                       |
| --------------------- | ------------------------------------------- |
| Async runtime         | `tokio`                                     |
| Serialization         | `serde` + `serde_json`                      |
| Config parsing        | `toml`                                      |
| Wayland (layer shell) | `smithay-client-toolkit` + `wayland-client` |
| D-Bus                 | `zbus`                                      |
| Rendering             | TBD — `tiny-skia` + `cosmic-text` or `iced` |
| Fonts                 | `cosmic-text`                               |
| Framebuffer (initui)  | Direct framebuffer + `fontdue`              |

---

## Workspace Structure

```
hyprion/
├── Cargo.toml              ← workspace root
├── ARCHITECTURE.md         ← this file
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
└── crates/
    ├── hyprion-core/       ← central daemon
    ├── hyprion-theme/      ← shared design token library
    ├── hyprionctl/         ← CLI tool
    ├── hyprion-bar/
    ├── hyprion-notif/
    ├── hyprion-osd/
    ├── hyprion-launcher/
    ├── hyprion-files/
    ├── hyprion-settings/
    ├── hyprion-session/
    ├── hyprion-wallpaper/
    ├── hyprion-screenshot/
    ├── hyprion-greeter/
    ├── hyprion-crypthelper/
    └── hyprion-initui/
```

---

## Open Questions

- Rendering approach for bar and other components — `iced` for speed of development vs `tiny-skia` for pixel-perfect control?
- Wallpaper engine video decoding — ffmpeg bindings vs gstreamer?
- Screenshot tool — build on top of existing Wayland screencopy protocol or deeper integration?
- Clipboard — exactly how do we enhance wl-clipboard without replacing it?
