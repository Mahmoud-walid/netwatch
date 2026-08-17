# NetWatch — Full Project Handoff & Implementation Specification

## 1. Role and Mission

You are taking over an existing software project called **NetWatch**.

Your job is to continue the implementation professionally from the current repository state, preserve valid existing work, correct architectural issues when necessary, and evolve the project into a production-quality Linux network monitoring and traffic accounting system.

The repository snapshot attached to this prompt contains the complete current project state. **Treat that file as the authoritative representation of the current implementation.**

Do not assume files, modules, dependencies, APIs, or functionality that are not present in the repository snapshot.

Do not rewrite the project from scratch.

First inspect the current repository carefully, understand what is already implemented, identify what is incomplete or architecturally weak, and then continue incrementally.

---

# 2. Product Vision

NetWatch is intended to become a **Linux network traffic monitoring and accounting platform**, with a strong focus on:

- Per-device traffic monitoring.
- Per-user traffic accounting.
- Download and upload statistics.
- Daily usage.
- Weekly usage.
- Monthly usage.
- Historical usage.
- Live bandwidth monitoring.
- Device discovery and identification.
- Device-to-user assignment.
- User-level aggregation across multiple devices.
- Persistent historical data.
- A local API.
- A native KDE Plasma 6 widget.
- Advanced dashboard functionality.
- Arch Linux and Arch-based distribution support.
- Safe and configurable storage locations.
- Strong storage safety guarantees.
- No silent fallback that can accidentally write data to the wrong filesystem.

The project must be designed as a real Linux application/daemon with a GUI integration, not merely as a KDE widget.

The KDE widget is a major part of the product, but it is only one frontend for the NetWatch core.

---

# 3. Critical Product Requirement: Accurate Accounting

The most important principle of NetWatch is:

> NetWatch must never present fabricated, guessed, or misleading traffic statistics.

Every reported number must have a real source and a clear accounting model.

For example, if the system claims:

- Mahmoud used 182 GB this month.
- A specific phone consumed 6.2 GB today.
- A device currently uses 12.4 Mbps.

Those numbers must come from real traffic counters or supported router/device data.

Do not infer per-device traffic from traffic visible only on the local machine.

---

# 4. Network Reality and Monitoring Modes

The current development machine has one physical Ethernet interface and is currently connected directly to the router.

The current topology is effectively:

Internet → MW306R router → multiple devices

The Linux development machine is only another client on that network.

Therefore:

**A normal client machine cannot reliably observe the traffic of other clients simply because it is connected to the same LAN.**

Do not build a fake solution that assumes otherwise.

NetWatch must support different collector modes.

## 4.1 Local Mode

The machine monitors its own traffic.

This is useful when the machine is just a normal network client.

The data source represents traffic belonging to that machine itself.

## 4.2 Gateway Mode

The Linux machine becomes a network gateway/router for the monitored devices.

Architecture:

Internet → Linux gateway → access point/router → monitored devices

In this mode, all relevant traffic passes through the Linux machine and can be accounted for accurately.

This is the primary approach for accurate per-device traffic accounting when the router itself does not provide usable traffic counters.

## 4.3 Router Integration Mode

Future-compatible architecture for routers that provide:

- API
- SNMP
- traffic counters
- supported management interfaces

The NetWatch core must support adapter/collector abstractions so router-specific implementations can be added later without redesigning the accounting engine.

## 4.4 Passive / Future Collection Modes

The architecture should remain extensible for future methods such as traffic mirrored from another location, bridge-based monitoring, or other passive collection mechanisms.

Do not implement unnecessary passive features in v1.

---

# 5. Router Context

The current router is:

- MERCUSYS MW306R
- Hardware version: MW306R 1.0
- Current observed firmware: 1.4.4 Build 201110 Rel.42649n(4555)

The router exposes connected-client information but does not provide a documented per-device traffic accounting interface that can currently be relied upon.

Therefore NetWatch must not depend on the MW306R for per-device traffic accounting.

The architecture may later support router adapters, but the core product must not require them.

---

# 6. Core Architecture

NetWatch should be structured as independent layers.

Conceptually:

Core / Daemon
→ Traffic Engine
→ Device Management
→ User Management
→ Accounting Engine
→ Storage
→ Local API
→ KDE Plasma frontend

The KDE Plasma widget must remain a frontend.

It must not:

- access SQLite directly,
- execute shell commands to collect traffic,
- require root,
- implement traffic accounting itself.

The widget should consume data from the NetWatch local API.

---

# 7. Core Technology Direction

The current architectural direction is:

- Rust for the core and daemon.
- SQLite for persistence.
- nftables and Linux networking facilities for traffic accounting in Gateway Mode.
- Netlink or appropriate Linux-native mechanisms where appropriate.
- Local REST API.
- WebSocket for live updates.
- Native KDE Plasma 6 / QML Plasmoid for the GUI.
- Arch-native packaging.

The core must not be tied specifically to CachyOS.

---

# 8. Linux / Distribution Portability

The project should target:

- Arch Linux.
- CachyOS.
- EndeavourOS.
- Manjaro.
- Garuda.
- Other Arch-based distributions where the required Linux facilities are available.

Do not use CachyOS-specific APIs.

Do not make NetworkManager a hard dependency.

Do not assume a specific Linux network configuration manager.

Do not hardcode paths belonging to the developer's machine.

Do not hardcode `/mnt/stor`.

The developer currently uses an HDD mounted at `/mnt/stor`, but that is only the developer's own storage configuration.

Other users must be able to choose completely different paths.

---

# 9. Storage Architecture

Storage location configurability is a first-class product requirement.

The user must be able to choose where persistent NetWatch data is stored.

Example:

`/mnt/stor/netwatch-data`

is valid for the current developer environment, but must never be hardcoded into application logic.

Another user might choose:

`/home/user/.local/share/netwatch`

or:

`/mnt/data/netwatch`

or any other valid storage path.

The application must support custom storage paths from the beginning.

---

# 10. Configuration vs Data Separation

Separate application configuration from persistent application data.

Conceptually:

Configuration:

user configuration directory
→ NetWatch configuration
→ config file

Persistent data:

user-selected storage path
→ database
→ backups
→ exports
→ persistent logs if appropriate
→ future persistent application data

The configuration file may contain the selected storage path.

Example:

`storage.path = "/mnt/stor/netwatch-data"`

But the value must come from configuration, not hardcoded source code.

---

# 11. Storage Safety Is Critical

This is one of the most important requirements.

The user intentionally wants to store NetWatch data on a separate HDD.

If the HDD disappears, is unmounted, or becomes unavailable, NetWatch must **not silently start writing to the underlying directory on another filesystem**.

Example dangerous scenario:

A disk is mounted at:

`/mnt/stor`

The configured storage path is:

`/mnt/stor/netwatch-data`

The disk becomes unavailable.

The `/mnt/stor` directory may still exist on the root filesystem.

NetWatch must not blindly create:

`/mnt/stor/netwatch-data`

on the root filesystem.

That would silently move persistent NetWatch data from the intended HDD to the SSD/root filesystem.

This must be explicitly prevented.

---

# 12. Storage Safety Model

The storage layer should distinguish concepts such as:

- Configured.
- Existing.
- Mounted.
- Writable.
- Available.
- Unsafe.
- Unavailable.

A storage path may exist while its intended filesystem is not mounted.

The system must determine which filesystem/mount actually backs the configured path.

Do not rely only on `Path::exists()`.

The current project already has a mount abstraction and Linux mount implementation using `findmnt`. Use the existing abstraction, but integrate it properly with the storage manager.

The `NotMounted` state currently exists conceptually but is not yet fully integrated into storage initialization.

Complete this safely.

---

# 13. No Silent Storage Fallback

If the user configured storage to a specific path and that storage becomes unavailable:

NetWatch must fail safely.

It must not automatically switch to a different default storage path.

It must report an explicit storage error/state.

Any fallback behavior must be a deliberate user-visible configuration decision, never an accidental side effect.

---

# 14. Storage Manager Responsibilities

The storage manager should be responsible for:

- validating storage paths,
- detecting invalid path types,
- creating directories when safe,
- checking mount availability,
- checking filesystem availability,
- checking writability,
- exposing current storage state,
- preventing unsafe writes,
- providing a clean interface for database initialization.

It should not contain SQLite-specific business logic.

---

# 15. Error Architecture

The current repository has more than one `ConfigError` definition.

This must be cleaned up.

Use a centralized error hierarchy in the appropriate core error module.

The architecture should allow errors for areas such as:

- configuration,
- storage,
- database,
- collector,
- networking,
- API,
- system integration.

Errors must be strongly typed.

Use idiomatic Rust error handling.

Do not use vague string-based error handling when a typed error is appropriate.

Do not use `unwrap()` or `expect()` in production paths unless there is a very strong invariant and justification.

Tests may use explicit expectations where appropriate.

---

# 16. Configuration Architecture

The configuration system should eventually support:

- storage path,
- monitoring mode,
- sampling interval,
- retention settings,
- timezone,
- API settings,
- future collector configuration,
- future limits/alerts.

Configuration should be versionable and migration-friendly.

Configuration loading must distinguish between:

- configuration file not found,
- invalid configuration,
- unreadable configuration,
- valid configuration.

Do not swallow configuration errors silently.

A corrupt config file must result in an explicit actionable error.

---

# 17. CLI Architecture

The current repository has:

- `netwatchd`
- `netwatch-cli`

The current argument parsing implementation is only a temporary/simple parser.

Refactor the CLI architecture before it grows further.

Use a proper CLI parser suitable for a real application.

The eventual CLI should support useful commands and options such as:

- status,
- start/diagnostics where applicable,
- configuration inspection,
- storage configuration,
- device listing,
- user listing,
- usage inspection,
- database diagnostics,
- version,
- help.

The CLI must not duplicate business logic.

The core must own business behavior.

The CLI is a frontend/client of that logic.

---

# 18. Daemon Architecture

`netwatchd` should be a thin orchestration layer.

Do not allow `main.rs` to become a large application implementation.

The daemon should eventually orchestrate components such as:

- configuration service,
- storage manager,
- database,
- device manager,
- collector manager,
- accounting engine,
- API server,
- WebSocket/live update service,
- lifecycle management.

The daemon should have clear startup/shutdown behavior.

---

# 19. Database

Use SQLite as the persistent storage engine.

The database must reside inside the configured NetWatch storage location.

Do not store the main database in the root filesystem if the user selected another path.

The database design must support historical accounting.

Do not store every packet individually.

Use counters, samples, and aggregates.

---

# 20. Suggested Domain Entities

The data model should include concepts equivalent to:

## Users

A logical person/account/entity responsible for one or more devices.

Examples:

Mahmoud  
Ahmed  
Guest  
Unassigned

## Devices

Physical/logical network clients.

Important attributes include:

- stable internal ID,
- MAC address,
- IP address,
- hostname,
- display name,
- assigned user,
- first seen,
- last seen,
- online/offline state,
- device metadata when available.

---

# 21. User vs Device

This distinction is essential.

A user can own multiple devices.

Example:

Mahmoud:

- desktop,
- phone,
- laptop.

The user's usage is the aggregate of the traffic of all devices assigned to that user.

Therefore:

User usage
=
sum of usage for the user's assigned devices.

Do not model user usage as a direct replacement for device usage.

Both levels must exist.

---

# 22. Device Identity

MAC address should be treated as an important identity signal, but the design must account for:

- DHCP IP changes,
- hostname changes,
- randomized Wi-Fi MAC addresses,
- MAC rotation,
- IPv4/IPv6,
- reconnects,
- device disappearance/reappearance.

Do not rely exclusively on IP addresses.

---

# 23. Device Management

Users should be able to:

- see discovered devices,
- see currently online devices,
- see offline devices,
- rename devices,
- assign devices to users,
- move devices between users,
- mark devices as ignored when appropriate,
- inspect device details,
- see historical usage.

Historical data must remain intact if a device goes offline.

---

# 24. Traffic Accounting

The accounting engine is a core component.

It should work from monotonically increasing counters where possible and calculate deltas between observations.

It must handle:

- daemon restarts,
- system reboots,
- counter resets,
- interface resets,
- router restarts where applicable,
- missing samples,
- duplicate observations,
- discontinuities.

Never double-count traffic after a restart.

If a counter decreases unexpectedly, treat that as a counter reset/discontinuity, not as a huge negative/positive transfer.

---

# 25. Download / Upload

Track separately:

- download bytes,
- upload bytes,
- total bytes.

At every level where meaningful:

- device,
- user,
- daily,
- weekly,
- monthly,
- historical.

---

# 26. Daily Usage

Daily accounting must represent:

00:00 → current time

according to the configured/local timezone.

Do not assume UTC boundaries.

Support timezone-aware calculations.

The current developer is in Egypt, but the application must not hardcode Egypt.

The timezone must be user/system configurable or derived from the operating environment appropriately.

---

# 27. Weekly Usage

Support:

- today,
- yesterday,
- current week,
- previous week.

The week boundary must be defined clearly and preferably configurable according to locale/application settings.

---

# 28. Monthly Usage

Monthly accounting is especially important.

Support:

- current calendar month,
- previous calendar month,
- historical monthly usage.

Monthly usage must be calculated according to calendar months, not merely "last 30 days".

Example:

August 1 → August 31

The system should be able to show per-user monthly usage.

---

# 29. Historical Data

The product must preserve historical information.

Useful aggregation levels include:

- live samples,
- hourly aggregates,
- daily aggregates,
- monthly aggregates.

Retention should be configurable.

Example policies:

- short-lived live samples,
- limited hourly history,
- long-lived daily history,
- long-lived monthly history.

Do not allow the database to grow indefinitely with unnecessarily granular data.

---

# 30. Live Traffic

The system should expose current bandwidth information.

Examples:

Device:

Download: 12.4 Mbps  
Upload: 0.8 Mbps

Network total:

Download: 18.7 Mbps  
Upload: 2.4 Mbps

Live information should be provided through an efficient mechanism.

Do not make the Plasma widget execute repeated shell commands.

---

# 31. REST API

Expose a local REST API for application clients.

The API should provide resources for concepts such as:

- status,
- devices,
- users,
- device details,
- user details,
- usage,
- historical data,
- storage state,
- settings where appropriate.

Design versioned endpoints.

A versioned API structure such as `/api/v1/...` is preferred.

The API should remain local by default.

Do not expose it publicly by default.

---

# 32. WebSocket / Live Updates

Use WebSocket or an equivalent persistent live-update mechanism for:

- live traffic,
- online/offline state,
- discovered devices,
- important state changes,
- real-time dashboard updates.

REST should primarily be used for request/response and historical information.

The Plasma widget should not aggressively poll every second if a live stream can provide the same information efficiently.

---

# 33. Security

The local API must default to localhost/local-only exposure unless the user explicitly enables broader exposure.

The Plasma widget must not require root.

The GUI must not directly perform privileged networking operations.

Privileged operations should stay inside the daemon/system integration layer.

Do not store passwords unnecessarily.

Do not log sensitive information.

Do not inspect or persist packet payload contents.

The monitoring system should focus on metadata/counters and byte accounting.

---

# 34. Traffic Collection Technology

For Gateway Mode, use Linux networking mechanisms appropriate for byte accounting.

The preferred architecture is based around:

- nftables counters,
- Linux networking,
- appropriate kernel-level counters,
- Netlink where appropriate.

Do not packet-sniff entire traffic payloads merely to count bytes.

Traffic accounting should be efficient and privacy-conscious.

---

# 35. Collector Architecture

Create collector abstractions.

Conceptually:

Collector
→ Local Collector
→ Gateway Collector
→ Router Collector

The collector interface should return normalized traffic/device information to the rest of NetWatch.

The accounting engine should not care whether traffic came from:

- nftables,
- router API,
- SNMP,
- another supported source.

This separation is essential for future extensibility.

---

# 36. KDE Plasma 6 Widget

The KDE Plasma widget is a major product requirement.

It must be a native KDE Plasma 6 / QML Plasmoid.

Do not replace it with:

- Conky,
- Electron,
- an embedded browser,
- a generic desktop overlay.

The widget should be installable as a normal Plasma widget.

It should consume the NetWatch local API.

---

# 37. Widget — Compact Mode

The compact widget should provide a useful glanceable overview.

Conceptually:

- NetWatch status.
- Online device count.
- Current download.
- Current upload.
- Today's total usage.
- Current month's total usage.
- Visual activity indication.

It should remain useful at a small desktop widget size.

---

# 38. Widget — Expanded Mode

Expanded mode should show:

- live network traffic,
- devices,
- each device's current bandwidth,
- today's usage,
- weekly usage,
- monthly usage,
- top consumers,
- online/offline state.

It should allow navigating to more detailed views.

---

# 39. Full Dashboard

The product should eventually provide a richer dashboard accessible through the widget.

Suggested sections:

- Overview
- Devices
- Users
- History
- Settings

The dashboard should provide:

- live traffic graphs,
- daily usage charts,
- weekly comparisons,
- monthly charts,
- top users,
- top devices,
- storage status,
- monitoring mode,
- system health.

---

# 40. Device Details UI

A device detail view should show things such as:

- display name,
- user,
- IP,
- MAC,
- online/offline state,
- last seen,
- live download,
- live upload,
- today's usage,
- weekly usage,
- monthly usage,
- historical graph.

---

# 41. User Details UI

A user detail page should aggregate all assigned devices.

Example:

Mahmoud

Devices: 3  
Today: 8.42 GB  
This week: 41.7 GB  
This month: 182.4 GB

The user detail view should also show the contribution of each assigned device.

---

# 42. Charts

The dashboard/widget should eventually contain polished charts for:

## 24-hour live/historical traffic

Bandwidth over time.

## Daily usage

Usage per day over a selected range.

## Monthly usage

Usage per calendar month.

## User comparison

Compare usage across users.

## Device comparison

Compare usage across devices.

The UI should prioritize clarity and readability over visual excess.

---

# 43. Top Consumers

Provide views such as:

Top users today.

Top devices today.

Top users this week.

Top users this month.

Top devices this month.

This should be calculated from real stored usage data.

---

# 44. Alerts

The architecture should support advanced alerts.

Potential examples:

- user exceeds daily usage threshold,
- user reaches monthly usage threshold,
- device exceeds configured usage limit,
- device suddenly consumes unusually high bandwidth,
- storage becomes unavailable,
- collector stops functioning.

Possible threshold levels:

- warning,
- critical,
- reached.

Do not implement traffic blocking in v1.

Monitoring and alerting comes before enforcement.

---

# 45. No Packet Payload Inspection in v1

Do not add:

- DPI,
- packet content collection,
- website blocking,
- parental controls,
- DNS filtering,
- firewall management,
- traffic shaping,
- VPN management,
- intrusion detection.

Those may become future products/features.

Version 1 is primarily:

**Network usage accounting + monitoring + visualization + historical analysis.**

---

# 46. Storage Retention

Provide configurable retention policies.

Potential categories:

- live samples,
- hourly aggregates,
- daily aggregates,
- monthly aggregates.

The user should eventually be able to control how much historical data is retained.

---

# 47. Export / Backup

The architecture should leave room for:

- CSV export,
- JSON export,
- database backup,
- historical data export.

Do not necessarily implement every export feature immediately, but design storage in a way that makes them possible.

---

# 48. Packaging

The project should eventually have Arch-native packaging.

Prepare for:

- PKGBUILD,
- daemon installation,
- CLI installation,
- KDE widget installation,
- configuration paths,
- systemd integration where appropriate.

Do not make systemd a hard architectural dependency of the core.

The daemon should be capable of running standalone.

Systemd integration should be packaging/system-integration functionality.

---

# 49. Service Management Portability

The core application must not assume:

- systemd,
- NetworkManager,
- CachyOS,
- a specific desktop environment.

Systemd support can be provided as the primary Arch integration, but it must remain outside the domain/core layer.

Future service-manager adapters should remain possible.

---

# 50. Privilege Architecture

Do not run the KDE frontend as root.

Do not make the desktop UI directly manipulate nftables.

Preferred model:

GUI
→ Local API
→ NetWatch daemon
→ privileged networking operations where necessary

Keep privilege boundaries explicit.

---

# 51. Testing Requirements

Testing is mandatory.

The current project already uses tests heavily and should continue this practice.

Every important module should have tests.

Tests should cover:

- configuration parsing,
- configuration serialization,
- storage validation,
- mount handling,
- storage safety,
- counter delta calculations,
- counter resets,
- restart behavior,
- daily boundaries,
- weekly boundaries,
- monthly boundaries,
- user/device aggregation,
- retention,
- API behavior,
- collector normalization.

Tests must not depend on the developer's machine-specific HDD names.

Do not hardcode `/mnt/stor` into tests unless a specific integration test explicitly documents that it requires the developer's environment.

---

# 52. Test Isolation

Tests must be isolated.

Do not allow multiple tests to delete each other's directories.

Use unique temporary directories or robust test isolation.

The current repository has already encountered parallel test interference in temporary storage directories, and this should not reappear.

---

# 53. Quality Gates

At appropriate checkpoints, the following should remain clean:

- `cargo fmt`
- `cargo check`
- `cargo test`
- `cargo clippy`

The project currently treats Clippy warnings as errors with a `-D warnings` style workflow.

Preserve this quality standard.

Do not introduce warnings and silence them casually.

---

# 54. Type Safety

Use idiomatic strongly typed Rust.

Do not use vague catch-all types to avoid modeling a real error/state.

Use explicit enums and domain types whenever the domain benefits from them.

---

# 55. Current Repository State

The attached repository snapshot already contains:

- Rust workspace.
- `netwatch-core`.
- `netwatchd`.
- `netwatch-cli`.
- configuration types.
- TOML configuration loading/saving.
- storage manager.
- storage status abstraction.
- Linux mount provider abstraction.
- CLI storage-path parsing.
- typed error infrastructure.
- test coverage around these components.

The current workspace uses Rust edition 2024.

The current configuration layer already models a storage path and default configuration/data directories.

The current storage manager already validates paths and creates missing directories, but mount safety is not yet integrated.

The current Linux mount provider uses an abstraction and a Linux implementation based on `findmnt`.

The current daemon can initialize using a configured storage path and persist it to the TOML configuration.

---

# 56. Important Existing Issues to Correct

Before implementing large new functionality, address the following issues found during repository review:

## Issue A — Duplicate ConfigError

There is currently one `ConfigError` definition in `config.rs` and another in `error.rs`.

Unify them into one coherent error architecture.

Do not leave duplicate domain error types.

## Issue B — `StorageStatus::NotMounted` is not actually implemented

The enum contains the state, but `StorageManager::status()` currently only checks path existence and directory state.

Integrate the mount provider.

## Issue C — Mount provider is not connected to StorageManager

Complete the dependency flow:

StorageManager
→ MountProvider
→ LinuxMountProvider

while keeping it testable through the abstraction.

## Issue D — Dangerous storage behavior

Do not create/write the configured storage directory before validating that its intended filesystem is actually mounted.

Prevent silent fallback to the root filesystem.

## Issue E — Configuration errors are currently swallowed

Do not use error-swallowing patterns that turn configuration parsing failures into "no configuration".

Corrupt configuration must be reported as an explicit configuration error.

## Issue F — `netwatchd` has too much orchestration logic in `main.rs`

Refactor toward a cleaner application/daemon initialization architecture.

## Issue G — Manual CLI parsing is temporary

The simple argument loop is not sufficient for the eventual CLI.

Replace it with a proper CLI architecture before the command surface grows.

---

# 57. Development Method

Work incrementally.

Do not implement the entire product in one large change.

Use milestones.

For each milestone:

1. Inspect the current state.
2. Explain what will change.
3. Implement the smallest coherent slice.
4. Add/update tests.
5. Run formatting.
6. Run checks.
7. Run tests.
8. Run Clippy.
9. Report what changed and why.
10. Stop at a meaningful checkpoint.

Do not jump ahead to the KDE widget while the underlying accounting engine is unverified.

The system must be data-first.

---

# 58. Recommended Implementation Order

Follow this order unless a technically justified change is necessary:

## Phase 0 — Foundation cleanup

- unify errors,
- clean configuration architecture,
- finalize storage abstraction,
- integrate mount detection,
- establish storage safety contract,
- refactor daemon initialization,
- improve CLI structure.

## Phase 1 — Persistent storage

- SQLite initialization,
- schema foundation,
- migration/versioning strategy,
- storage integrity checks.

## Phase 2 — Device model

- device identity,
- device discovery,
- online/offline state,
- device persistence,
- device/user relationship.

## Phase 3 — Traffic engine

- Local Collector,
- Gateway Collector,
- normalized traffic counter model,
- byte counters,
- download/upload direction,
- counter delta calculation,
- reset/restart handling.

## Phase 4 — Accounting engine

- live traffic,
- daily aggregation,
- weekly aggregation,
- monthly aggregation,
- historical aggregation,
- retention.

## Phase 5 — Users

- user entities,
- device assignment,
- user aggregation,
- user history,
- top consumers.

## Phase 6 — API

- REST API,
- API versioning,
- storage status,
- device endpoints,
- user endpoints,
- usage endpoints.

## Phase 7 — Live communication

- WebSocket,
- live device changes,
- live traffic updates,
- daemon health/status updates.

## Phase 8 — KDE Plasma 6

- native QML Plasmoid,
- compact view,
- expanded view,
- dashboard,
- devices,
- users,
- charts,
- details,
- live updates.

## Phase 9 — Advanced features

- alerts,
- limits,
- exports,
- backups,
- retention controls,
- historical comparisons,
- advanced statistics.

## Phase 10 — Packaging

- Arch packaging,
- PKGBUILD,
- daemon integration,
- CLI installation,
- Plasma widget installation,
- documentation,
- upgrade/migration behavior.

---

# 59. Important Principle About the Widget

The KDE widget is not optional.

A completed NetWatch product is expected to include a polished KDE Plasma 6 widget.

However, the widget must sit on top of a strong backend.

Do not implement fake widget data just to demonstrate UI.

The UI should eventually consume real API data generated by the real accounting engine.

---

# 60. Expected Final User Experience

The final product should let a user install NetWatch, configure a storage path, select the appropriate monitoring mode, and then view their network usage.

A typical user should be able to see:

## Overview

- current download speed,
- current upload speed,
- total network usage today,
- total usage this week,
- total usage this month,
- online device count.

## Users

For each user:

- devices,
- today,
- this week,
- this month,
- historical usage.

## Devices

For each device:

- name,
- IP,
- MAC,
- user,
- online/offline,
- live traffic,
- daily usage,
- weekly usage,
- monthly usage.

## History

- 24-hour graphs,
- daily graphs,
- weekly comparison,
- monthly comparison,
- historical trends.

## Storage

- selected path,
- available/unavailable,
- mounted/unmounted,
- writable/not writable,
- database status.

---

# 61. What Not To Do

Do not:

- rewrite the repository from scratch,
- hardcode `/mnt/stor`,
- assume the router exposes per-device traffic counters,
- infer other-device traffic from local client traffic,
- store packet payloads,
- make the GUI a privileged process,
- make KDE a core dependency,
- make CachyOS a core dependency,
- make NetworkManager a core dependency,
- assume systemd in the domain layer,
- silently swallow configuration errors,
- silently fall back to another storage path,
- add unnecessary dependencies without justification,
- ignore Clippy warnings,
- skip tests,
- create fake traffic data for the GUI.

---

# 62. Deliverables

The final system should provide:

1. A robust Rust NetWatch daemon.
2. Strong local persistent storage.
3. Configurable data location.
4. Safe external-disk storage behavior.
5. Device monitoring.
6. Per-device accounting.
7. User-based accounting.
8. Daily usage.
9. Weekly usage.
10. Monthly usage.
11. Historical usage.
12. Live bandwidth.
13. REST API.
14. WebSocket/live updates.
15. Native KDE Plasma 6 widget.
16. Advanced dashboard.
17. Arch-native packaging.
18. Strong tests.
19. Clear documentation.
20. Maintainable modular architecture.

---

# 63. Final Development Rule

Do not optimize for quickly producing a visible UI.

Optimize for correctness of the underlying network data first.

The correct order is:

**real traffic source → normalized counters → accounting → persistence → API → live updates → KDE UI**

The final widget should be a polished presentation of trustworthy data.

Start from the current repository state in the attached project snapshot.

Do not ask the user to recreate files that already exist.

Do not invent repository contents.

Before modifying a file, inspect its current content and preserve valid existing behavior.

Proceed incrementally and report each meaningful milestone clearly.