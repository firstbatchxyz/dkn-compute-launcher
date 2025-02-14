//! Utilities for releases & version management.

use eyre::{eyre, Context, OptionExt, Result};
use self_update::backends::github;
use self_update::update::{Release, ReleaseAsset};
use std::env::consts::{ARCH, FAMILY, OS};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// The latest compute node will always be at this file for a chosen directory.
pub const DKN_LATEST_COMPUTE_FILENAME: &str = "dkn-compute-node_latest";

/// The filename for the version tracker file, simply stores the string for the version.
pub const DKN_VERSION_TRACKER_FILENAME: &str = ".dkn.version";

/// A Dria repostiry enum, to differentiate between compute and launcher.
/// Can maybe add oracle here as well some day!
#[derive(Debug, Clone, Copy)]
pub enum DriaRepository {
    ComputeNode,
    Launcher,
}

impl std::fmt::Display for DriaRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriaRepository::ComputeNode => write!(f, "dkn-compute-node"),
            DriaRepository::Launcher => write!(f, "dkn-compute-launcher"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DriaRelease(Release, DriaRepository);

impl DriaRelease {
    #[inline]
    pub fn name(&self) -> &str {
        &self.0.name
    }

    #[inline]
    pub fn version(&self) -> &str {
        &self.0.version
    }

    /// Returns the filename for the current machine for this release.
    #[inline]
    pub fn to_filename(&self) -> Result<String> {
        if let Some((_, _, ext)) = Self::get_labels() {
            match self.1 {
                DriaRepository::ComputeNode => {
                    Ok(format!("dkn-compute-node_v{}{}", self.version(), ext))
                }
                DriaRepository::Launcher => {
                    Ok(format!("dkn-compute-launcher_v{}{}", self.version(), ext))
                }
            }
        } else {
            Err(eyre!("unsupported OS {} ARCH {}", OS, ARCH))
        }
    }

    /// Returns the os, arch and family extension name for the current machine.
    ///
    /// If the current machine is not supported, returns `None`.
    pub fn get_labels() -> Option<(&'static str, &'static str, &'static str)> {
        let os = match OS {
            "windows" | "linux" => OS,
            "macos" => "macOS", // due to damn capitalization in workflow :/
            _ => return None,
        };

        let arch = match ARCH {
            "x86_64" => "amd64",
            "arm" | "aarch64" => "arm64",
            _ => return None,
        };

        let ext = match FAMILY {
            "windows" => ".exe",
            "unix" => "",
            _ => return None,
        };

        Some((os, arch, ext))
    }

    /// Returns the locally recorded compute node version.
    pub fn get_compute_version(exe_dir: &PathBuf) -> Option<String> {
        let compute_path = exe_dir.join(DKN_VERSION_TRACKER_FILENAME);
        fs::read_to_string(&compute_path).ok()
    }

    /// Updates the locally recorded compute node version, returns the path to the version tracker file.
    pub fn set_compute_version(exe_dir: &PathBuf, version: &str) -> Result<PathBuf> {
        let compute_path = exe_dir.join(DKN_VERSION_TRACKER_FILENAME);
        fs::write(&compute_path, version).wrap_err("could not write version to file")?;

        Ok(compute_path)
    }

    /// Returns the release asset for this machine.
    ///
    /// Selects the asset w.r.t current OS and ARCH and returns one of:
    ///
    /// - `"dkn-compute-binary-linux-amd64`
    /// - `"dkn-compute-binary-linux-arm64`
    /// - `"dkn-compute-binary-macOS-amd64`
    /// - `"dkn-compute-binary-macOS-arm64`
    /// - `"dkn-compute-binary-windows-amd64.exe"`
    #[inline]
    pub fn asset(&self) -> Result<ReleaseAsset> {
        self.0
            .assets
            .iter()
            .find(|asset| {
                let Some((os, arch, ext)) = Self::get_labels() else {
                    return false;
                };

                let target_name = match self.1 {
                    DriaRepository::ComputeNode => {
                        format!("dkn-compute-binary-{}-{}{}", os, arch, ext)
                    }
                    DriaRepository::Launcher => {
                        format!("dkn-compute-launcher-{}-{}{}", os, arch, ext)
                    }
                };
                asset.name == target_name
            })
            .ok_or(eyre!(
                "asset not found for OS {} ARCH {} FAMILY {}",
                OS,
                ARCH,
                FAMILY
            ))
            .cloned()
    }

    /// Downloads this release under the given directory at the given `dest_name`.
    pub async fn download_release(
        &self,
        dest_dir: &Path,
        dest_name: impl AsRef<Path>,
    ) -> Result<PathBuf> {
        if !dest_dir.is_dir() {
            return Err(eyre!(
                "destination directory {} does not exist / not a directory",
                dest_dir.display()
            ));
        }

        let dest_path = dest_dir.join(dest_name);

        let asset = self.asset()?;
        log::info!(
            "Downloading {} (v{}) to {}",
            asset.name,
            self.version(),
            dest_path.display()
        );
        download_asset_via_url(asset.download_url, &dest_path).await?;

        Ok(dest_path)
    }

    /// Returns the latest compute node release.
    #[inline]
    pub async fn from_latest_release(repo: DriaRepository) -> Result<DriaRelease> {
        match repo {
            DriaRepository::ComputeNode => get_compute_releases().await?,
            DriaRepository::Launcher => get_launcher_releases().await?,
        }
        .first()
        .cloned()
        .ok_or_eyre("no releases found")
    }
}

pub async fn download_latest_compute_node(
    exe_dir: &PathBuf,
    local_version: &str,
) -> Result<(Option<PathBuf>, String)> {
    // get latest release & check if we need to update
    let latest_release = DriaRelease::from_latest_release(DriaRepository::ComputeNode).await?;
    let latest_version = latest_release.version();
    if local_version == latest_version {
        return Ok((None, latest_version.into()));
    }

    // download the latest release to the same path
    let latest_path = latest_release
        .download_release(exe_dir, DKN_LATEST_COMPUTE_FILENAME)
        .await?;

    Ok((Some(latest_path), latest_version.into()))
}

pub async fn download_latest_launcher(
    exe_dir: &PathBuf,
    local_version: &str,
) -> Result<(Option<PathBuf>, String)> {
    const TMP_FILE_NAME: &str = ".tmp.launcher";

    // get latest release & check if we need to update
    let latest_release = DriaRelease::from_latest_release(DriaRepository::Launcher).await?;
    let latest_version = latest_release.version();
    if local_version == latest_version {
        return Ok((None, latest_version.into()));
    }

    // download the latest release to a temporary path
    let latest_path = latest_release
        .download_release(&exe_dir, TMP_FILE_NAME)
        .await?;

    Ok((Some(latest_path), latest_version.into()))
}

async fn download_asset_via_url(download_url: String, dest_path: &PathBuf) -> Result<()> {
    let dest_file = fs::File::create(&dest_path)?;
    tokio::task::spawn_blocking(move || {
        self_update::Download::from_url(download_url.as_ref())
            .set_header(
                reqwest::header::ACCEPT,
                // this is unlikely to panic
                "application/octet-stream".parse().unwrap(),
            )
            .show_progress(true)
            .download_to(dest_file)
            .expect("could not download asset")
    })
    .await
    .wrap_err("could not download asset")?;

    // set to read, write, execute
    fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o777))?;

    Ok(())
}

impl std::fmt::Display for DriaRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[inline]
pub async fn get_compute_releases() -> Result<Vec<DriaRelease>> {
    get_releases(DriaRepository::ComputeNode).await
}

#[inline]
pub async fn get_launcher_releases() -> Result<Vec<DriaRelease>> {
    let releases = get_releases(DriaRepository::Launcher).await?;

    // filter version `0.0.x` here because they belong to the old launcher,
    // and the old launcher has zip outputs only
    Ok(releases
        .into_iter()
        .filter(|r| !r.version().starts_with("0.0"))
        .collect())
}

/// Returns the entire list of releases for the given repository, owned by `firstbatchxyz`.
///
/// Due to an [issue](// https://github.com/jaemk/self_update/issues/44) of `self_update` not
/// working within async contexts, we do a blocking task spawn here.
async fn get_releases(repo: DriaRepository) -> Result<Vec<DriaRelease>> {
    tokio::task::spawn_blocking(move || {
        let mut rel_builder = github::ReleaseList::configure();

        rel_builder
            .repo_owner("firstbatchxyz")
            .repo_name(&repo.to_string())
            .build()
            .expect("could not build release list")
            .fetch()
            .expect("could not fetch releases")
            .into_iter()
            .map(|r| DriaRelease(r, repo))
            .collect::<Vec<_>>()
    })
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    #[tokio::test]
    async fn test_download_last_compute_release() {
        let final_release = &super::get_compute_releases().await.unwrap()[0];

        let path = final_release
            .download_release(
                &PathBuf::from_str(".").unwrap(),
                &final_release.to_filename().unwrap(),
            )
            .await
            .unwrap();

        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_download_last_launcher_release() {
        let final_release = &super::get_launcher_releases().await.unwrap()[0];
        println!("final_release: {:?}", final_release);
        let path = final_release
            .download_release(
                &PathBuf::from_str(".").unwrap(),
                &final_release.to_filename().unwrap(),
            )
            .await
            .unwrap();

        assert!(path.exists());
    }
}
