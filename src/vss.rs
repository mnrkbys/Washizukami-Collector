use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[cfg(windows)]
use serde::Deserialize;

/// Metadata for a discovered Volume Shadow Copy snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShadowCopy {
    pub id: String,
    pub original_volume: String,
    pub device_object: String,
    pub install_date: Option<String>,
}

pub fn is_snapshot_path(path: &Path) -> bool {
    path.to_string_lossy()
        .to_ascii_uppercase()
        .contains("GLOBALROOT\\DEVICE\\HARDDISKVOLUMESHADOWCOPY")
}

/// Return the drive-root prefix (`C:`) for an absolute Windows path.
pub fn path_drive(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    let bytes = path_str.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        Some(path_str[..2].to_ascii_uppercase())
    } else {
        None
    }
}

/// Convert a live absolute path into the equivalent path inside `snapshot`.
///
/// `C:\Windows\System32\config\SAM` with device object
/// `\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy3`
/// becomes
/// `\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy3\Windows\System32\config\SAM`.
pub fn map_live_path_to_snapshot(snapshot: &ShadowCopy, live_path: &Path) -> Option<PathBuf> {
    let live_path_str = live_path.to_string_lossy();
    let bytes = live_path_str.as_bytes();
    if bytes.len() < 3 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' {
        return None;
    }

    let suffix = &live_path_str[2..];
    let suffix = suffix.trim_start_matches(['\\', '/']);
    let mut mapped = snapshot.device_object.trim_end_matches(['\\', '/']).to_owned();
    if !suffix.is_empty() {
        mapped.push('\\');
        mapped.push_str(&suffix.replace('/', "\\"));
    }

    Some(PathBuf::from(mapped))
}

pub fn expand_shadow_patterns(target_path: &str, vss_enabled: bool) -> Result<Vec<String>> {
    if !vss_enabled {
        return Ok(vec![target_path.to_owned()]);
    }

    if is_snapshot_path(Path::new(target_path)) {
        return Ok(vec![target_path.to_owned()]);
    }

    let drive = match path_drive(Path::new(target_path)) {
        Some(drive) => drive,
        None => return Ok(vec![target_path.to_owned()]),
    };

    let mut snapshots = snapshots_for_volume(&drive)?;
    if snapshots.is_empty() {
        return Ok(vec![target_path.to_owned()]);
    }

    snapshots.sort_by(|left, right| {
        right
            .install_date
            .cmp(&left.install_date)
            .then_with(|| left.device_object.cmp(&right.device_object))
    });

    let mut patterns = Vec::with_capacity(snapshots.len());
    for snapshot in snapshots {
        if let Some(mapped) = map_live_path_to_snapshot(&snapshot, Path::new(target_path)) {
            patterns.push(mapped.to_string_lossy().into_owned());
        }
    }

    if patterns.is_empty() {
        return Ok(vec![target_path.to_owned()]);
    }

    // VSS collection always includes the live path as a baseline source.
    patterns.push(target_path.to_owned());

    Ok(patterns)
}

#[cfg(windows)]
#[derive(Debug, Deserialize)]
#[serde(rename = "Win32_ShadowCopy")]
#[serde(rename_all = "PascalCase")]
struct ShadowCopyRow {
    id: String,
    device_object: String,
    volume_name: String,
    install_date: Option<String>,
    state: Option<u32>,
}

#[cfg(windows)]
pub fn snapshots_for_volume(volume: &str) -> Result<Vec<ShadowCopy>> {
    use wmi::WMIConnection;

    let normalized_volume = normalize_volume_name(volume_guid_path(volume)?);
    let connection = WMIConnection::new().context("failed to connect to WMI for VSS enumeration")?;
    let rows: Vec<ShadowCopyRow> = connection
        .raw_query(
            "SELECT ID, DeviceObject, VolumeName, InstallDate, State FROM Win32_ShadowCopy",
        )
        .context("failed to query Win32_ShadowCopy")?;

    let mut snapshots = Vec::new();
    for row in rows {
        if !is_collectable_shadow_state(row.state) {
            continue;
        }

        if normalize_volume_name(&row.volume_name) != normalized_volume {
            continue;
        }

        snapshots.push(ShadowCopy {
            id: row.id,
            original_volume: volume.to_ascii_uppercase(),
            device_object: row.device_object,
            install_date: row.install_date,
        });
    }

    Ok(snapshots)
}

#[cfg(not(windows))]
pub fn snapshots_for_volume(volume: &str) -> Result<Vec<ShadowCopy>> {
    let _ = volume;
    Ok(vec![])
}

#[cfg(windows)]
fn volume_guid_path(volume: &str) -> Result<String> {
    use windows::Win32::Storage::FileSystem::GetVolumeNameForVolumeMountPointW;
    use windows::core::PCWSTR;

    let mount_point = ensure_mount_point(volume);
    let mount_point_wide: Vec<u16> = mount_point.encode_utf16().chain(std::iter::once(0)).collect();
    let mut buffer = vec![0u16; 128];

    unsafe {
        GetVolumeNameForVolumeMountPointW(
            PCWSTR(mount_point_wide.as_ptr()),
            &mut buffer,
        )
        .with_context(|| format!("failed to resolve volume GUID path for {mount_point}"))?;
    }

    let len = buffer.iter().position(|&ch| ch == 0).unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..len]))
}

#[cfg(not(windows))]
fn volume_guid_path(volume: &str) -> Result<String> {
    Ok(volume.to_owned())
}

fn ensure_mount_point(volume: &str) -> String {
    let mut mount_point = volume.trim().replace('/', "\\");
    if !mount_point.ends_with('\\') {
        mount_point.push('\\');
    }
    mount_point
}

fn normalize_volume_name(volume: impl AsRef<str>) -> String {
    volume
        .as_ref()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_uppercase()
}

fn is_collectable_shadow_state(state: Option<u32>) -> bool {
    // VSS_SNAPSHOT_STATE defines CREATED as 12.
    matches!(state, Some(12))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> ShadowCopy {
        ShadowCopy {
            id: "shadow-1".to_owned(),
            original_volume: "C:".to_owned(),
            device_object: r"\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy3".to_owned(),
            install_date: Some("20260525010101.000000+000".to_owned()),
        }
    }

    #[test]
    fn path_drive_extracts_drive_letter() {
        assert_eq!(path_drive(Path::new(r"C:\Windows\System32")), Some("C:".to_owned()));
        assert_eq!(path_drive(Path::new(r"d:\Temp")), Some("D:".to_owned()));
        assert_eq!(path_drive(Path::new(r"relative\path")), None);
    }

    #[test]
    fn map_live_path_to_snapshot_rewrites_root() {
        let mapped = map_live_path_to_snapshot(
            &sample_snapshot(),
            Path::new(r"C:\Windows\System32\config\SAM"),
        )
        .unwrap();

        assert_eq!(
            mapped,
            PathBuf::from(r"\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy3\Windows\System32\config\SAM")
        );
    }

    #[test]
    fn map_live_path_to_snapshot_rejects_non_drive_paths() {
        assert!(map_live_path_to_snapshot(&sample_snapshot(), Path::new(r"\\server\share\file"))
            .is_none());
        assert!(map_live_path_to_snapshot(&sample_snapshot(), Path::new(r"relative\path")).is_none());
    }

    #[test]
    fn snapshot_path_detection_matches_globalroot_prefix() {
        assert!(is_snapshot_path(Path::new(
            r"\\?\GLOBALROOT\Device\HarddiskVolumeShadowCopy7\Windows\System32"
        )));
        assert!(!is_snapshot_path(Path::new(r"C:\Windows\System32")));
    }

    #[test]
    fn normalize_volume_name_is_case_insensitive() {
        assert_eq!(
            normalize_volume_name(r"\\?\Volume{ABC}\"),
            normalize_volume_name(r"\\?\volume{abc}")
        );
    }

    #[test]
    fn collectable_shadow_state_accepts_created_state_only() {
        assert!(is_collectable_shadow_state(Some(12)));
        assert!(!is_collectable_shadow_state(Some(9)));
    }

    #[test]
    fn collectable_shadow_state_rejects_non_created_values() {
        assert!(!is_collectable_shadow_state(None));
        assert!(!is_collectable_shadow_state(Some(13)));
        assert!(!is_collectable_shadow_state(Some(14)));
    }
}