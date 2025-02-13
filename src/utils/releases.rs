//! Utilities for releases & version management.

use std::env::consts::{ARCH, FAMILY, OS};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use eyre::{eyre, Context, OptionExt, Result};
use self_update::backends::github;
use self_update::update::{Release, ReleaseAsset};

#[derive(Debug, Clone)]
pub struct DriaRelease(Release);

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
            Ok(format!("dkn-compute-node_v{}{}", self.version(), ext))
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

    /// Returns the release asset for this machine.
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

                asset.name == format!("dkn-compute-binary-{}-{}{}", os, arch, ext)
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
        let dest_file = fs::File::create(&dest_path)?;

        let asset = self.asset()?;
        eprintln!(
            "Downloading {} (v{}) to {}",
            asset.download_url,
            self.version(),
            dest_path.display()
        );
        tokio::task::spawn_blocking(move || {
            self_update::Download::from_url(&asset.download_url)
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

        Ok(dest_path)
    }

    /// Returns the latest compute node release.
    #[inline]
    pub async fn get_latest_compute_release() -> Result<DriaRelease> {
        get_compute_releases()
            .await?
            .first()
            .cloned()
            .ok_or_eyre("no releases found")
    }
}

impl std::fmt::Display for DriaRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[inline]
pub async fn get_compute_releases() -> Result<Vec<DriaRelease>> {
    get_releases("dkn-compute-node").await
}

#[inline]
#[allow(unused)] // TODO: !!!
pub async fn get_launcher_releases() -> Result<Vec<DriaRelease>> {
    get_releases("dkn-compute-launcher").await
}

/// Returns the entire list of releases for the given repository, owned by `firstbatchxyz`.
///
/// Due to an [issue](// https://github.com/jaemk/self_update/issues/44) of `self_update` not
/// working within async contexts, we do a blocking task spawn here.
async fn get_releases(repo_name: &'static str) -> Result<Vec<DriaRelease>> {
    tokio::task::spawn_blocking(move || {
        let mut rel_builder = github::ReleaseList::configure();

        rel_builder
            .repo_owner("firstbatchxyz")
            .repo_name(repo_name)
            .build()
            .expect("could not build release list")
            .fetch()
            .expect("could not fetch releases")
            .into_iter()
            .map(DriaRelease)
            .collect::<Vec<_>>()
    })
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    #[tokio::test]
    async fn test_compute_releases() {
        let releases = super::get_compute_releases().await.unwrap();
        assert!(!releases.is_empty());
        // eprintln!("{:#?}", releases[0]);
    }

    #[tokio::test]
    async fn test_launcher_releases() {
        let releases = super::get_launcher_releases().await.unwrap();
        assert!(!releases.is_empty());
        // eprintln!("{:#?}", releases[0]);
    }

    #[tokio::test]
    async fn test_download_last_release() {
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
}
