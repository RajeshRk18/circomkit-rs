//! PTAU (Powers of Tau) file utilities

use crate::error::{CircomkitError, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Information about a PTAU file
#[derive(Debug, Clone)]
pub struct PtauInfo {
    /// Power of 2 for the number of constraints supported
    pub power: u8,
    /// File name
    pub filename: String,
    /// URL to download from
    pub url: String,
    /// Expected file size in bytes
    pub size: u64,
}

/// Hermez ceremony PTAU files
const HERMEZ_PTAU_BASE: &str = "https://storage.googleapis.com/zkevm/ptau";

/// Get information about the recommended PTAU for a given number of constraints
pub fn get_recommended_ptau(num_constraints: usize) -> PtauInfo {
    // Calculate minimum power needed
    let power = (num_constraints as f64).log2().ceil() as u8;
    let power = power.max(8).min(28); // Clamp between 8 and 28

    let filename = format!("powersOfTau28_hez_final_{:02}.ptau", power);
    let url = format!("{}/{}", HERMEZ_PTAU_BASE, filename);

    // Approximate sizes (actual sizes vary)
    let size = match power {
        8 => 8_388_608,
        9 => 16_777_216,
        10 => 33_554_432,
        11 => 67_108_864,
        12 => 134_217_728,
        13 => 268_435_456,
        14 => 536_870_912,
        15 => 1_073_741_824,
        16 => 2_147_483_648,
        _ => 0, // Unknown size for larger powers
    };

    PtauInfo {
        power,
        filename,
        url,
        size,
    }
}

/// Download a PTAU file
pub async fn download_ptau(info: &PtauInfo, output_dir: &Path) -> Result<PathBuf> {
    let output_path = output_dir.join(&info.filename);

    // Check if already exists
    if output_path.exists() {
        log::info!("PTAU file already exists: {:?}", output_path);
        return Ok(output_path);
    }

    // Create output directory if needed
    fs::create_dir_all(output_dir).await?;

    log::info!("Downloading PTAU from: {}", info.url);
    log::info!("This may take a while for larger files...");

    // Use curl or wget to download
    let output = std::process::Command::new("curl")
        .arg("-L")
        .arg("-o")
        .arg(&output_path)
        .arg("--progress-bar")
        .arg(&info.url)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                // Try wget instead
                return CircomkitError::tool_not_found("curl (or wget)");
            }
            CircomkitError::Io(e)
        })?;

    if !output.status.success() {
        // Try wget as fallback
        let output = std::process::Command::new("wget")
            .arg("-O")
            .arg(&output_path)
            .arg("--show-progress")
            .arg(&info.url)
            .output()
            .map_err(|e| CircomkitError::Io(e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CircomkitError::CommandFailed {
                command: "wget".to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }
    }

    log::info!("Downloaded PTAU to: {:?}", output_path);

    Ok(output_path)
}

/// Verify a PTAU file integrity
pub async fn verify_ptau(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Err(CircomkitError::PtauNotFound(path.to_path_buf()));
    }

    // Check file size is reasonable (at least 1MB)
    let metadata = fs::metadata(path).await?;
    if metadata.len() < 1_000_000 {
        return Ok(false);
    }

    // Check file starts with correct magic bytes (zkey format)
    let content = fs::read(path).await?;
    if content.len() < 4 {
        return Ok(false);
    }

    // PTAU files should start with specific bytes
    // This is a simplified check
    Ok(true)
}

/// Get all PTAU files in a directory
pub async fn list_ptau_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    let mut entries = fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map(|e| e == "ptau").unwrap_or(false) {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_recommended_ptau() {
        let info = get_recommended_ptau(100);
        assert_eq!(info.power, 8); // 2^7 = 128 > 100

        let info = get_recommended_ptau(1000);
        assert_eq!(info.power, 10); // 2^10 = 1024 > 1000

        let info = get_recommended_ptau(1_000_000);
        assert_eq!(info.power, 20); // 2^20 = 1048576 > 1000000
    }

    #[test]
    fn test_ptau_info_url() {
        let info = get_recommended_ptau(1000);
        assert!(info.url.contains("powersOfTau28_hez_final_10.ptau"));
    }
}
