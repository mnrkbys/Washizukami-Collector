# Artifact Definitions

> Keep it simple. Keep it reliable.

Washizukami classifies every artifact definition into exactly two kinds.

| | Core | Custom |
| --- | --- | --- |
| Location | `artifacts/core/` | `artifacts/custom/` |
| Shipped | Embedded in `washi.exe` at compile time | Sample files in this repository |
| Collected | By default, with no configuration | Only after being loaded via `config.yaml` |
| Maintenance | Continuously verified and maintained | Provided as-is |

## Core

The Core Set is what a responder gets by double-clicking `washi.exe`. A
definition belongs in Core when all of the following hold:

- it is used at high frequency during Windows incident response
- it is present across a wide range of environments
- its location and format are comparatively stable
- it is worth verifying continuously
- collecting it by default is not excessive

Note that the test is **not** "is it shipped with Windows". It is "should a fast
forensic collection take it as a matter of course".

| File | Category | Contents |
| --- | --- | --- |
| `core/windows_eventlogs.yaml` | `EventLogs` | Security / System / Application `.evtx` |
| `core/windows_registry.yaml` | `Registry` | SAM / SECURITY / SOFTWARE / SYSTEM, Amcache, per-user NTUSER.DAT and UsrClass.dat, including transaction logs |
| `core/windows_ntfs.yaml` | `NTFS` | `$MFT`, `$Secure:$SDS`, `$UsnJrnl:$J` across all NTFS volumes |
| `core/windows_filesystem.yaml` | `Filesystem` | Prefetch, Recent `.lnk` |
| `core/windows_wmi.yaml` | `WMI` | WMI repository (OBJECTS.DATA / INDEX.BTR / MAPPING\*.MAP) |
| `core/windows_srum.yaml` | `SRUM` | SRUDB.dat |
| `core/windows_web.yaml` | `Web` | Chrome, Edge, Firefox, IE/Edge WebCache |

These files are embedded via `include_str!` in `src/config.rs`. Adding, removing
or renaming a file here requires a matching change to `EMBEDDED_SOURCES`.

## Custom

| File | Category | Why it is not Core |
| --- | --- | --- |
| `custom/paging.yaml` | `Paging` | pagefile.sys / swapfile.sys / hiberfil.sys are sized in proportion to installed RAM — commonly 20-30 GB together. Valuable, but incompatible with a fast triage default. |
| `custom/browsers-extra.yaml` | `Web` | Brave / Vivaldi / Opera / Yandex are Chromium derivatives installed only deliberately. |
| `custom/ai-tools.yaml` | `AI Tools` | Claude Code and Codex CLI change their on-disk layout on their own release cycle, and prompt history is unusually privacy-sensitive. |
| `custom/outlook.yaml` | `Mail` | The `.pst` path varies by Outlook edition, display language and OneDrive configuration; no single path is correct across environments. |

### Using a Custom definition

Custom files are already shaped like a `config.yaml`. Either:

- copy one next to `washi.exe` and rename it to `config.yaml`, or
- merge its `artifacts:` entries into an existing `config.yaml`.

```powershell
washi.exe --dry-run --category Paging   # confirm what would be collected
washi.exe --category Paging
```

Until a Custom definition is loaded this way, its category is unknown to the
CLI and `--category` will reject it.

## File formats

The two directories use different YAML shapes, because each is read by a
different loader in `src/config.rs`:

```yaml
# artifacts/core/*.yaml — a bare list, parsed as Vec<ArtifactDefinition>
- name: Security Event Log
  category: EventLogs
  target_path: "%SystemRoot%\\System32\\winevt\\Logs\\Security.evtx"
  method: NTFS
```

```yaml
# artifacts/custom/*.yaml — an ExternalConfig, same shape as config.yaml
artifacts:
  - name: pagefile.sys
    category: Paging
    target_path: "%SystemDrive%\\pagefile.sys"
    method: NTFS
```

Fields are identical in both:

| Field | Description |
| --- | --- |
| `name` | Unique display name. Referenced by `enabled_artifacts` in `config.yaml`. |
| `category` | Group name. Also used as the output subfolder name. |
| `target_path` | Path to collect. Supports `%VAR%` environment variables and glob wildcards (`*`, `?`, `**`). |
| `method` | `File` — standard OS copy. `NTFS` — direct MFT read, bypasses file locks. |
| `stream` | Optional NTFS Alternate Data Stream name (e.g. `$SDS`, `$J`). `NTFS` method only. |

A definition loaded from `config.yaml` overrides an embedded one with the same
`name`, so a Custom file can also be used to retarget a Core artifact.
