# Hyprion

> A modular, Rust-native desktop environment layer for Hyprland.

Hyprion is what you reach for when you want Hyprland to feel like a real DE — cohesive, polished, and complete — without sacrificing the freedom that made you choose Hyprland in the first place.

Every component is an independent binary. Install all of them, or just the one you need. They all speak the same language.

## Philosophy

- **Modular** — each component works standalone, all of them work better together
- **Rust-native** — no scripts gluing things together, no shell wrappers
- **Cohesive** — shared design system, shared IPC, feels like one thing made it all
- **Hyprland-first** — not a generic Wayland DE, built specifically around Hyprland's ecosystem

## Components

| Component          | Description                                      | Status     |
| ------------------ | ------------------------------------------------ | ---------- |
| `hyprion-core`     | Central daemon, IPC bus, API for app integration | 🚧 Planned |
| `hyprion-bar`      | Status bar                                       | 🚧 Planned |
| `hyprion-notif`    | Notification server (D-Bus)                      | 🚧 Planned |
| `hyprion-osd`      | On-screen display (volume, brightness...)        | 🚧 Planned |
| `hyprion-launcher` | App launcher                                     | 🚧 Planned |
| `hyprion-files`    | File browser                                     | 🚧 Planned |
| `hyprion-settings` | Settings GUI                                     | 🚧 Planned |
| `hyprion-session`  | Power/logout menu                                | 🚧 Planned |

## Status

**Early development.** Nothing is usable yet. The architecture is being designed.

If you're interested in contributing or following progress, watch the repo.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
