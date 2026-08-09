<div align="center">
 <p>
    <img alt="Washizukami Logo" src="Logo.png" width="60%">
 </p>
 [ <b>English</b> ] | [<a href="README-Japanese.md">日本語</a>]
</div>

---

# Washizukami (鷲掴)

> **Windows Forensic Evidence Collection Tool**

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%2010%2F11%20x64-blue.svg)]()

---

## Overview

**Washizukami (鷲掴)** is a fast forensic evidence collection tool for Windows, implemented in Rust.

Even when the OS has files locked, it can acquire artifacts such as registry hives and event logs by directly parsing the NTFS Master File Table (MFT). Collected evidence is saved alongside an audit log containing SHA-256 hashes, making it ready to feed directly into various analysis tools.

This tool was inspired by [CDIR-C](https://github.com/CyberDefenseInstitute/CDIR) (Cyber Defense Institute). It aims to deliver the live-system artifact collection approach pioneered by CDIR-C as a portable single binary through a Rust implementation.

**Example analysis tools it pairs with:**

- [Hayabusa](https://github.com/Yamato-Security/hayabusa) — Threat hunting for Windows event logs
- [Velociraptor](https://github.com/Velocidex/velociraptor) / [KAPE](https://www.kroll.com/en/services/cyber-risk/incident-response-litigation-support/kroll-artifact-parser-extractor-kape) and other forensic frameworks
- Ingestion into SIEMs such as ELK Stack / Splunk

### Project Philosophy

> **Keep it simple. Keep it reliable.**

Washizukami is not aiming to become a general-purpose DFIR platform. It does one job — collecting evidence during the first hours of a Windows incident — and leaves analysis to the tools built for it. New default behaviour is added sparingly, and the command line stays small enough to use correctly under pressure.

What that restraint buys is the other half: collection that is dependable, reproducible, and maintainable. Every default artifact is one we intend to keep verifying, which is why the definition set is deliberately smaller than it could be.

---

## Features

| Feature                            | Description                                                                                               |
| ---------------------------------- | --------------------------------------------------------------------------------------------------------- |
| **NTFS Raw Read**                  | Directly parses the MFT to bypass OS file locks during collection                                         |
| **SHA-256 Integrity Verification** | Hashes all collected files to enable tamper detection                                                     |
| **Audit Log**                      | Structured log (`collection.log`) containing timestamps, collection method, and SHA-256 hashes            |
| **Single Binary**                  | Artifact definitions are embedded at compile time — no external files needed at runtime                   |
| **Flexible Filtering**             | Filter by category via `--category` (include or exclude with `!` prefix), or fine-tune via `config.yaml`  |
| **ZIP Output**                     | Compresses all collected artifacts into a single ZIP after collection for easy exfiltration               |
| **Memory Acquisition Integration** | `--mem` option integrates with [WinPmem](https://github.com/Velocidex/WinPmem) to capture memory dumps    |
| **Dry-Run Mode**                   | Verify collection target paths without touching the filesystem                                            |
| **YARA Scanning**                  | `scan` subcommand scans persistence mechanisms with YARA-X, collecting detected files into `infected.zip` |
| **Confirmation Prompt**            | Prompts `[y/N]` before starting collection or scanning to prevent accidental execution                    |

---

## Requirements

| Item                          | Requirement                                                                    |
| ----------------------------- | ------------------------------------------------------------------------------ |
| **OS**                        | Windows 10 / Windows 11 (x64)                                                  |
| **Privileges**                | Must be run with **Administrator** privileges                                  |
| **Runtime**                   | Not required (statically built — no VC++ Redistributable or MinGW DLLs needed) |
| **Disk Space**                | At least equal to the total size of artifacts to be collected                  |
| **Memory Acquisition Option** | When using `--mem`, place `winpmem*.exe` in the `tools\` folder                |

> **Note:** Because NTFS Raw Read is used, the target volume must be NTFS-formatted. Files on FAT32/exFAT volumes are collected using the standard File collector.

---

## Usage

### Quick Start — Double-Click Launch

The simplest way to use Washizukami is to **double-click `washi.exe`** in Explorer.

No configuration needed. Just right-click → **Run as administrator**, and collection starts immediately with default settings:

- Collects the entire **Core Set** (see [Collected Artifacts](#collected-artifacts))
- Output folder: **`HOSTNAME_ALL_YYYYMMDDHHMMSS\`** (created next to `washi.exe`, when NTFS uses `%AllNtfsDrives%`)
- Audit log: **`HOSTNAME_ALL_YYYYMMDDHHMMSS\collection.log`** (with SHA-256 hashes)

The console window stays open after collection completes so you can review the results. Press **Enter** to close it.

For more control — filtering by category, specifying an output directory, generating a ZIP, etc. — use the CLI options described below.

---

### Artifact Collection Mode (Default)

```
washi.exe [OPTIONS]

Options:
  -o, --output <DIR>               Output directory
                                   [Default: <executable folder>\<COMPUTERNAME>_<SOURCE>_<YYYYMMDDHHMMSS>]
                                   Default naming uses SOURCE=ALL in all-drive mode, or SOURCE=C in
                                   legacy single-drive mode.
  -c, --category <CATEGORY>        Filter by category (repeatable, case-insensitive).
                                   Without prefix: collect only these categories.
                                   With '!' prefix: exclude these categories.
                                   Available: EventLogs, Registry, NTFS, Filesystem, WMI, SRUM, Web
                                   Custom definitions loaded via config.yaml add their own categories.
      --dry-run                    Display path resolution results only (no files are collected)
      --zip                        Generate a ZIP archive after collection
      --mem                        Capture memory dump with tools\winpmem*.exe (runs before collection)
      --vss                        Collect from all discovered Volume Shadow Copy snapshots
                                   and always include the live volume.
      --volume <LETTER>            Override the source drive letter for all artifact collection.
                                   Useful when running washi.exe from a USB drive (e.g., D:) and
                                   targeting the system drive (C:) — default behavior collects from C:.
                                   Use --volume D to collect artifacts from D: instead.
                                   When NTFS artifacts use %AllNtfsDrives%, --volume takes precedence
                                   and forces collection from only the specified drive.
  -v, --verbose                    Show every collected file instead of one summary line per category
  -h, --help
  -V, --version
```

### YARA Scan Mode

```
washi.exe scan [OPTIONS] --rules <FILE> --output <DIR>

Options:
      --yara-path <PATH>           Path to YARA-X engine (yr.exe) [Default: ./tools/yr.exe]
      --rules <FILE>               Path to YARA rules file (required)
      --output <DIR>               Output directory for scan results (required)
  -h, --help
```

**Prerequisites:** Place [YARA-X](https://github.com/VirusTotal/yara-x) (`yr.exe`) in the `tools\` folder alongside `washi.exe`, or specify its path with `--yara-path`.

#### How it works

The scan mode targets executable paths registered in Windows **persistence mechanisms** — locations the OS uses to automatically launch programs. These are common hiding spots for malware that needs to survive reboots.

**Step 1 — Collect persistence targets**

Washi automatically enumerates the following sources and extracts the executable paths:

| Source                                               | What it covers                                                        |
| ---------------------------------------------------- | --------------------------------------------------------------------- |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Programs launched at login for **all users** (system-wide)            |
| `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Programs launched at login for the **current user** only              |
| `C:\Windows\System32\Tasks` (Task Scheduler XML)     | Scheduled tasks — programs triggered by time, events, or system state |

For Run key entries, argument strings are stripped (e.g., `"C:\App\tool.exe" --silent` → `C:\App\tool.exe`) and environment variables are expanded (`%SystemRoot%` → `C:\Windows`). Entries without an absolute path (bare filenames like `sc.exe`) are skipped with a warning since they cannot be reliably located on disk.

**Step 2 — Scan with YARA-X**

Collected paths are written to a temporary list file and passed to `yr.exe` via `--scan-list`. yr.exe scans each file's content against your YARA rules.

**Step 3 — Archive matches**

Files that triggered a YARA rule are copied into `infected.zip` in the output directory. The audit log (`collection.log`) records every match with the rule name and file path.

#### Sample output

```
·  Collecting persistence targets…
⚠  Not an absolute path: sc.exe              ← skipped (no absolute path)
·  48 target(s) to scan
·  Running YARA scan…
⚠  C:\Users\Public\malware.exe  —  Detect_Mimikatz
────────────────────────────────────────────────────
⚠  Scan complete  ·  1 of 48 target(s) matched
   Archive  C:\scan_out\infected.zip
```

### Confirmation Prompt

Before starting collection or a YARA scan, washi.exe displays a confirmation prompt:

```
[?] Start collection? [y/N]:
```

Type `y` or `yes` (case-insensitive) to proceed. Any other input, pressing Enter alone, or Ctrl+C will abort. `--dry-run` skips the prompt since no files are written.

### Examples

```powershell
# Collect all artifacts (with audit log)
washi.exe

# Generate a ZIP archive after collection
washi.exe --zip

# Capture memory dump → Collect all artifacts → Generate ZIP
washi.exe --mem --zip

# Collect Registry and EventLogs only
washi.exe --category Registry --category EventLogs

# Collect everything except EventLogs and WMI
washi.exe --category '!EventLogs' --category '!WMI'

# Show every collected file (verbose output)
washi.exe --verbose

# Collect from all discovered Volume Shadow Copy snapshots + live volume
washi.exe --vss

# Verify collection targets (no files are written)
washi.exe --dry-run

# Specify output directory
washi.exe --output D:\evidence\case001 --zip

# Collect from a different drive (e.g., forensic target mounted as D:)
washi.exe --volume D --output E:\evidence\case001

# Dry-run with VSS path expansion
washi.exe --vss --dry-run

# YARA scan (scan persistence paths and collect detected files into infected.zip)
washi.exe scan --rules C:\rules\malware.yar --output C:\scan_out
```

---

## Collected Artifacts

Artifact definitions come in exactly two kinds. See [`artifacts/README.md`](artifacts/README.md) for the full classification rules.

| | Core | Custom |
| --- | --- | --- |
| Location | `artifacts/core/` | `artifacts/custom/` |
| Shipped | Embedded in `washi.exe` | Sample files in this repository |
| Collected | By default, with no configuration | Only after being loaded via `config.yaml` |
| Maintenance | Continuously verified and maintained | Provided as-is |

### Core Set

Collected by default. This is what you get by double-clicking `washi.exe`.

| Category       | Artifact                                                                                             | Collection Method |
| -------------- | ---------------------------------------------------------------------------------------------------- | ----------------- |
| **EventLogs**  | Security / System / Application Event Log                                                            | NTFS              |
| **Registry**   | SAM / SECURITY / SOFTWARE / SYSTEM hives (incl. transaction logs: .LOG1 / .LOG2)                    | NTFS              |
| **Registry**   | Amcache.hve (incl. .LOG1 / .LOG2)                                                                   | NTFS              |
| **Registry**   | NTUSER.DAT / UsrClass.dat (all users, incl. .LOG1 / .LOG2 and TxR files)                            | NTFS              |
| **NTFS**       | `$MFT` (Master File Table)                                                                           | NTFS              |
| **NTFS**       | `$SECURE:$SDS` (Security Descriptor Stream)                                                          | NTFS + ADS        |
| **NTFS**       | `$UsnJrnl:$J` (USN Journal) — only allocated extents are collected; sparse leading region is skipped | NTFS + ADS        |
| **Filesystem** | Prefetch files (`Prefetch\*.pf`)                                                                     | File              |
| **Filesystem** | Recent files (`Recent\*.lnk`)                                                                        | File              |
| **WMI**        | WMI Repository (OBJECTS.DATA / INDEX.BTR / MAPPING\*.MAP)                                            | NTFS              |
| **SRUM**       | SRUM Database (SRUDB.dat)                                                                            | NTFS              |
| **Web**        | Chrome History                                                                                       | File              |
| **Web**        | Firefox History & Cookies (places.sqlite / cookies.sqlite)                                           | File              |
| **Web**        | IE / Edge WebCache (WebCacheV01.dat)                                                                 | File              |
| **Web**        | Edge History                                                                                         | File              |

> **NTFS + ADS:** Alternate Data Streams are acquired via direct MFT reads. This enables access to streams that cannot be read through normal APIs.

> **All NTFS drives:** Core NTFS metadata artifacts use `%AllNtfsDrives%` and are expanded to every OS-recognized NTFS drive. Non-NTFS drives are skipped automatically.

### Custom Definitions

Not collected by default. Each file under `artifacts/custom/` is already shaped like a `config.yaml`, so you can either copy one next to `washi.exe` and rename it to `config.yaml`, or merge its `artifacts:` entries into an existing `config.yaml`.

| File | Category | Artifact | Why it is not Core |
| --- | --- | --- | --- |
| `custom/paging.yaml` | `Paging` | `pagefile.sys` / `swapfile.sys` / `hiberfil.sys` | Sized in proportion to installed RAM — commonly 20-30 GB together. Valuable, but incompatible with a fast triage default. |
| `custom/browsers-extra.yaml` | `Web` | Brave / Vivaldi / Opera / Yandex history | Chromium derivatives installed only deliberately. |
| `custom/ai-tools.yaml` | `AI Tools` | Claude Code / Codex CLI local data | On-disk layout changes on the tools' own release cycle, and prompt history is unusually privacy-sensitive. |
| `custom/outlook.yaml` | `Mail` | Classic Outlook `.pst` files | The path varies by Outlook edition, display language and OneDrive configuration. |

```powershell
# After copying artifacts\custom\paging.yaml next to washi.exe as config.yaml
washi.exe --dry-run --category Paging   # confirm sizes first
washi.exe --category Paging
```

> Until a Custom definition is loaded this way, its category is unknown to the CLI and `--category` will reject it.

---

## Output Structure

```
<executable folder>\
├── HOSTNAME_ALL_YYYYMMDDHHMMSS\  ← Output folder (created next to washi.exe)
│   ├── collection.log          ← Audit log (timestamps, SHA-256, collection method)
│   ├── memory.dmp              ← Memory dump (only when --mem is specified)
│   ├── EventLogs\
│   │   ├── C\
│   │   │   ├── Windows\System32\winevt\Logs\Security.evtx
│   │   │   └── ...
│   ├── Registry\
│   │   ├── C\
│   │   │   ├── Windows\System32\config\SAM
│   │   │   └── ...
│   ├── NTFS\
│   │   ├── C\
│   │   │   ├── $MFT
│   │   │   ├── $Secure_SDS     ← $SECURE:$SDS stream
│   │   │   └── $UsnJrnl_J      ← $UsnJrnl:$J stream
│   │   └── D\
│   │       └── ...             ← additional NTFS drives when present
│   ├── Filesystem\
│   │   ├── C\
│   │   │   └── ...
│   │   └── D\
│   │       └── ...
│   ├── WMI\
│   │   └── C\
│   │       └── ...
│   ├── SRUM\
│   │   └── C\
│   │       └── Windows\System32\sru\SRUDB.dat
│   └── Web\
│        └── C\
│            └── ...
└── HOSTNAME_ALL_YYYYMMDDHHMMSS.zip ← ZIP archive (only when --zip is specified)
```

### Audit Log Format

```
[2026-03-21T10:30:00+0900] [OK   ] [NTFS        ] C:\Windows\System32\config\SAM -> HOSTNAME_ALL_20260418120000\Registry\C\Windows\System32\config\SAM (262144 bytes, SHA256: abcd1234...)
[2026-03-21T10:30:01+0900] [SKIP ] [-           ] C:\path\missing — file not found
[2026-03-21T10:30:02+0900] [FAIL ] [-           ] C:\path\locked — <error>
[2026-03-21T10:30:03+0900] [TOOL ] [winpmem_x64 ] Starting: tools\winpmem_x64.exe -> HOSTNAME_ALL_20260418120000\memory.dmp
[2026-03-21T10:30:10+0900] [INFO ] [-           ] Complete — OK: 141  Skipped: 1  Failed: 0

# When running washi.exe scan
[2026-03-23T11:00:00+0900] [SCAN ] [yr          ] Starting scan — engine: ./tools/yr.exe  rules: malware.yar  targets: 59
[2026-03-23T11:00:02+0900] [MATCH] [yara        ] C:\Windows\System32\notepad.exe — test_notepad
[2026-03-23T11:00:02+0900] [SCAN ] [-           ] Complete — matched: 1  archive: scan_out\infected.zip
```

---

## Customizing Artifact Definitions

The Core Set covers Windows event logs, registry hives, and common filesystem artifacts. By placing a `config.yaml` in the same folder as `washi.exe`, you can narrow collection targets or add Custom artifacts.

**Priority:** CLI flags > `config.yaml` > Core Set (embedded)

### Filters

#### `enabled_artifacts`

Whitelist of artifact names to collect. If empty or omitted, all artifacts are collected. Matching is case-insensitive.

<details>
<summary>Core Set artifact names</summary>

| Category   | Name                           |
| ---------- | ------------------------------ |
| EventLogs  | `Security Event Log`           |
| EventLogs  | `System Event Log`             |
| EventLogs  | `Application Event Log`        |
| Registry   | `SAM Registry Hive`            |
| Registry   | `SECURITY Registry Hive`       |
| Registry   | `SOFTWARE Registry Hive`       |
| Registry   | `SYSTEM Registry Hive`         |
| Registry   | `Amcache.hve`                  |
| Registry   | `User NTUSER.DAT`              |
| Registry   | `User UsrClass.dat`            |
| NTFS       | `$MFT`                         |
| NTFS       | `$SECURE:$SDS`                 |
| NTFS       | `$UsnJrnl:$J`                  |
| Filesystem | `Prefetch Files`               |
| Filesystem | `Recent LNK Files`             |
| WMI        | `WMI Repository OBJECTS.DATA`  |
| WMI        | `WMI Repository INDEX.BTR`     |
| WMI        | `WMI Repository MAPPING Files` |
| SRUM       | `SRUM Database`                |
| Web        | `Chrome History`               |
| Web        | `Firefox places.sqlite`        |
| Web        | `Firefox cookies.sqlite`       |
| Web        | `IE/Edge WebCacheV01.dat`      |
| Web        | `Edge History`                 |

</details>

#### `disabled_categories`

Excludes entire categories from collection. Valid values: `EventLogs` / `Registry` / `NTFS` / `Filesystem` / `WMI` / `SRUM` / `Web` (case-insensitive), plus any category introduced by Custom definitions in the same `config.yaml`.

> **Note:** `disabled_categories` is evaluated **after** `enabled_artifacts`. An artifact explicitly listed in `enabled_artifacts` will still be excluded if its category appears in `disabled_categories`.

### Custom Artifact Definitions

Use the `artifacts` key to add artifacts that are not part of the Core Set. If a custom entry shares the same `name` as a Core artifact, the custom definition takes priority.

Required fields:

| Field         | Description                                                                               |
| ------------- | ----------------------------------------------------------------------------------------- |
| `name`        | Unique display name. Referenced by `enabled_artifacts` in `config.yaml`.                  |
| `category`    | Group name. Also used as the output subfolder name.                                       |
| `target_path` | Path to collect. Supports `%VAR%` environment variables and glob wildcards (`*` and `?`). |
| `method`      | `File` — standard OS copy. `NTFS` — direct MFT read, bypasses file locks.                 |

### Example `config.yaml`

```yaml
# ── Filters ──────────────────────────────────────────────────────────────────
# Collect only these artifacts (comment out to collect all)
enabled_artifacts:
  - "SAM Registry Hive"
  - "Security Event Log"
  - "System Event Log"

# Exclude entire categories
disabled_categories:
  - Filesystem
  - Web

# ── Custom artifact definitions ───────────────────────────────────────────────
artifacts:
  - name: "My Application Log"
    category: "Custom"
    target_path: "C:\\MyApp\\logs\\app.log"
    method: File

  - name: "My Locked DB"
    category: "Custom"
    target_path: "%SystemDrive%\\MyApp\\data\\app.db"
    method: NTFS

  - name: "All XML Configs"
    category: "Custom"
    target_path: "C:\\MyApp\\config\\*.xml"
    method: File
```

### Loading a Custom Definition

Each file under [`artifacts/custom/`](artifacts/custom/) is already shaped like a `config.yaml`. To use one, copy it next to `washi.exe` and rename it to `config.yaml`, or merge its `artifacts:` entries into an existing `config.yaml`. Then confirm the result with `--dry-run` before collecting for real.

#### `custom/paging.yaml` — Paging / hibernation files

`pagefile.sys` and `hiberfil.sys` are sized in proportion to installed RAM, so a 16 GB host commonly yields 20-30 GB in total. They can hold fragments of process memory (credentials, decrypted payloads, network buffers).

> **Size warning:** always run `--dry-run --category Paging` first. On a host with a large amount of RAM these three files alone can exceed the free space on the collection target.
>
> `hiberfil.sys` is absent when hibernation is disabled (`powercfg /h off`), and `swapfile.sys` is absent when no Store apps have been run.

#### `custom/browsers-extra.yaml` — Chromium-derivative browsers

Brave, Vivaldi, Opera and Yandex. These entries use `category: Web`, so they are collected together with the Core browser artifacts and land in the same `Web\` output subtree. Each `History` file is a SQLite database readable with the same tooling used for Chrome.

#### `custom/ai-tools.yaml` — Claude Code / Codex CLI

[Claude Code](https://claude.ai/code) stores its local data under `%USERPROFILE%\.claude\`, and [Codex CLI](https://github.com/openai/codex) under `%USERPROFILE%\.codex\`.

| Path | Contents |
| ---- | -------- |
| `.claude\history.jsonl` | Full record of all prompt inputs entered by the user |
| `.claude\paste-cache\*` | Large text pastes stored externally to keep `history.jsonl` compact |
| `.claude\image-cache\*` | Large image pastes (same reason as paste-cache) |
| `.claude\file-history\*` | Pre-edit snapshots of files modified by Claude Code |
| `.codex\history.jsonl` | Prompt history: prompt text, session ID, timestamp |
| `.codex\sessions\**\rollout-*.jsonl` | Per-session event log: session metadata, user/agent messages, reasoning, tool calls and results, task state, token usage, context compaction, errors |
| `.codex\attachments\**\*` | Files attached to a Codex CLI session |

All entries use `method: NTFS` because these tools may be running and holding file locks during collection — Codex CLI in particular appends to its rollout logs continuously throughout a session.

> **Why `**`?** Codex writes rollout logs into a dated tree (`sessions\YYYY\MM\DD\`). A fixed number of single-level `*` wildcards would silently miss files, so `**` recursive descent is required. Note that a *trailing* bare `**` matches directories only — write `**\*` to sweep every file in a subtree.
>
> **Not collected:** credentials (`auth.json`), configuration (`config.toml`), environment files, shell snapshots, SQLite databases, and internal caches are deliberately excluded from these definitions.
>
> **Note on `image-cache`:** if you have never pasted an image into Claude Code, the directory does not exist and will show `⚠ NO MATCH` in dry-run output — this is expected.

#### `custom/outlook.yaml` — Classic Outlook `.pst` files

Classic Outlook (not the New Outlook app) stores `.pst` files in a location that varies by Outlook edition, Windows display language and OneDrive configuration. The shipped definition covers **Japanese Windows with OneDrive enabled**:

```
C:\Users\<username>\OneDrive\ドキュメント\Outlook ファイル\*.pst
```

Adjust `target_path` for other layouts, and verify with `--dry-run` before relying on it.

> **Why `method: NTFS`?** Classic Outlook holds an exclusive lock on `.pst` files while running. Using NTFS raw read bypasses the lock and allows collection without closing Outlook.
>
> **Size warning:** `.pst` files can be several GB. Run `--dry-run` first to check sizes before collecting.

---

## Memory Acquisition (WinPmem Integration)

Using the `--mem` option, you can capture a memory dump with [WinPmem](https://github.com/Velocidex/WinPmem) before artifact collection begins.

1. Download `winpmem_x64.exe` from the [WinPmem Releases page](https://github.com/Velocidex/WinPmem/releases)
2. Place it in the `tools\` folder alongside `washi.exe`
3. Run with the `--mem` flag

```
(Directory layout)
washi.exe
tools\
└── winpmem_x64.exe
```

> If `tools\winpmem*.exe` is not found, a warning is logged and artifact collection proceeds without memory acquisition.

---

## Building from Source

**Prerequisites:**

- Rust stable toolchain (`x86_64-pc-windows-gnu`)
- MSYS2 + MinGW-w64 (GNU linker)

```powershell
git clone https://github.com/tadmaddad/Washizukami-Collector.git
cd Washizukami-Collector
cargo build --release
```

---

## Roadmap

The following feature enhancements are currently planned or under consideration. Implementation order is undecided.

### YARA Scan Enhancements

The `scan` subcommand was implemented in v0.4.0. The following enhancements are being considered:

- `--target` option for scanning arbitrary directories
- Password protection for `infected.zip` (AES-256) — currently unimplemented due to build environment constraints
- Expanded scan targets (Startup folders, service registration paths, etc.)

### Email Client Artifacts

#### Microsoft Outlook `.pst` — available now as a Custom definition

Classic Outlook `.pst` collection is already supported via [`artifacts/custom/outlook.yaml`](artifacts/custom/outlook.yaml). See [`custom/outlook.yaml` — Classic Outlook `.pst` files](#customoutlookyaml--classic-outlook-pst-files) for details.

#### Planned Custom definitions

The following are planned as future Custom definitions:

| Client                  | Target Files                                                      |
| ----------------------- | ----------------------------------------------------------------- |
| **Microsoft Outlook**   | `.ost` files, attachment cache                                    |
| **Mozilla Thunderbird** | Mailboxes (`*.msf` / `INBOX`), address books, configuration files |

Since email data tends to be large, optimizations such as date-range filtering and differential collection are also being considered.

---

## Bug Fixes

### v0.6.1 — `$UsnJrnl:$J` sparse file over-collection

**Symptom:** The collected `$UsnJrnl:$J` was as large as the journal's full logical size (10 GB+) rather than the actual records size.

**Root cause:** `$UsnJrnl:$J` is a sparse file. Windows manages the USN Journal as a circular buffer: older records are deallocated, leaving a large sparse region (virtual zeros with no on-disk storage) at the beginning, and only the tail contains actual USN records. The previous implementation read the entire logical stream including sparse regions, which the ntfs crate fills with zeros, producing a file equal to the journal's maximum logical size.

**Fix:** `NtfsNonResidentAttributeValue::data_runs()` is now iterated directly. Sparse data runs (where `data_position()` returns `None`) are skipped without writing zeros. Only the allocated extents are written to the output file.

---

## Origin of the Name: Why "Washizukami (鷲掴)"?

This tool is named out of deep respect for **[Hayabusa](https://github.com/Yamato-Security/hayabusa)**, the de facto standard for Windows log analysis and a favorite among security engineers.

If the Hayabusa (Peregrine Falcon) — the king of the skies — spots its prey with razor-sharp eyes, then this tool physically "eagle-grabs" (鷲掴み) that prey (artifacts), overpowering even OS restrictions (file locks) to bring them back. The name embodies that commitment to powerful evidence collection.

...That said, the above is the official (serious) explanation.

We occasionally receive insinuations that "the author's personal preferences may be reflected in the naming," but this is categorically untrue. All we want is to hold NTFS MFT entries and registry hives — firmly, yet gently — through legally proper procedures.

---

## AI-Assisted Development

Design decisions, review, and final judgement on this project rest with its maintainer. AI assistants are used as development tools, and this section is here for transparency about that.

- **Claude Code** — Repository-level implementation: Rust code changes, refactoring, tests, and Git / GitHub workflow execution.
- **ChatGPT** — Roadmap planning, architecture and design discussion, critical review, task decomposition, release planning, and working out design principles and project direction.

Every change is reviewed and accepted by the maintainer before it lands.

---

## License

Copyright (C) 2026 tadmaddad - Jawfish Lab

This software is released as open source under the GNU Affero General Public License v3.0 (AGPL-3.0).

---

## Libraries & Tools Used

- [ntfs](https://github.com/ColinFinck/ntfs) by Colin Finck — Pure Rust NTFS parser enabling direct MFT access
- [WinPmem](https://github.com/Velocidex/WinPmem) by Velocidex — Windows memory acquisition tool
