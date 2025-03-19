use eyre::{eyre, Context, Result};
use self_update::backends::github;
use self_update::update::{Release, ReleaseAsset};
use std::env::consts::{ARCH, FAMILY, OS};
use std::fs;
use std::path::{Path, PathBuf};

use super::{DKN_VERSION_TRACKER_FILE, PROGRESS_BAR_CHARS, PROGRESS_BAR_TEMPLATE};

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

/// A Dria release, which is a release from the `firstbatchxyz` repository.
///
/// This struct wraps around the `self_update::update::Release` struct and adds
/// some utility functions for the release management.
#[derive(Debug, Clone)]
pub struct DriaRelease(Release, DriaRepository);

impl std::fmt::Display for DriaRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl DriaRelease {
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.0.name
    }

    #[inline(always)]
    pub fn version(&self) -> &str {
        &self.0.version
    }

    /// Returns the filename for the current machine for this release.
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
            Err(eyre!("unsupported platform: {}-{}", ARCH, OS))
        }
    }

    /// Returns the `os`, `arch` and `family` extension name for the current machine.
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
    ///
    /// Returns `None` if the version tracker file does not exist or could not be read.
    #[inline]
    pub fn get_compute_version(exe_dir: &Path) -> Option<String> {
        let compute_path = exe_dir.join(DKN_VERSION_TRACKER_FILE);
        fs::read_to_string(&compute_path).ok()
    }

    /// Updates the locally recorded compute node version, returns the path to the version tracker file.
    #[inline]
    pub fn set_compute_version(exe_dir: &Path, version: &str) -> Result<PathBuf> {
        let compute_path = exe_dir.join(DKN_VERSION_TRACKER_FILE);
        fs::write(&compute_path, version).wrap_err("could not write version to file")?;

        Ok(compute_path)
    }

    /// Selects the asset w.r.t current OS and ARCH.
    ///
    /// ### Returns
    /// The release asset for this machine; for example, the compute node binaries will look like one of the following:
    ///
    /// - `"dkn-compute-binary-linux-amd64`
    /// - `"dkn-compute-binary-linux-arm64`
    /// - `"dkn-compute-binary-macOS-amd64`
    /// - `"dkn-compute-binary-macOS-arm64`
    /// - `"dkn-compute-binary-windows-amd64.exe"`
    ///
    /// ### Errors
    /// - If an asset could not be found for the current OS and ARCH.
    pub fn asset(&self) -> Result<ReleaseAsset> {
        let Some((os, arch, ext)) = Self::get_labels() else {
            return Err(eyre!("unsupported platform: {}-{}", ARCH, OS));
        };

        self.0
            .assets
            .iter()
            .find(|asset| {
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
            .ok_or(eyre!("asset not found for {}-{}", os, arch,))
            .cloned()
    }

    /// Downloads this release under the given directory at the given `dest_name`.
    ///
    /// ### Arguments
    /// - `dest_dir`: The directory where the release will be downloaded.
    /// - `dest_name`: The name of the downloaded release.
    /// - `show_progress`: Log download progress to stdout
    ///
    /// ### Returns
    /// The path to the downloaded release.
    ///
    /// ### Errors
    /// - If the destination directory does not exist or is not a directory.
    /// - If the asset could not be found for the current OS and ARCH.
    /// - If the asset could not be downloaded.
    pub async fn download_release(
        &self,
        dest_dir: &Path,
        dest_name: impl AsRef<Path>,
        show_progress: bool,
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
        download_asset_via_url(asset.download_url, &dest_path, show_progress).await?;

        Ok(dest_path)
    }
}

/// Downloads the asset from the given URL to the given path.
///
/// The downloaded file will be first written to a temporary file,
/// and when the download is finished it will be renamed to actualy destination.
/// This prevents corrupt files when the download is interrupted.
async fn download_asset_via_url(
    download_url: String,
    dest_path: &PathBuf,
    show_progress: bool,
) -> Result<()> {
    // TODO: use tempfile here
    // download asset to a tempfile
    let tmp_file = dest_path.with_file_name(format!(
        "tmp_{}",
        dest_path.file_name().unwrap_or_default().to_string_lossy()
    ));
    let tmp_dest = fs::File::create(&tmp_file)?;
    tokio::task::spawn_blocking(move || {
        self_update::Download::from_url(download_url.as_ref())
            .set_progress_style(PROGRESS_BAR_TEMPLATE.into(), PROGRESS_BAR_CHARS.into())
            .set_header(
                reqwest::header::ACCEPT,
                // this is unlikely to panic
                "application/octet-stream".parse().unwrap(),
            )
            .show_progress(show_progress)
            .download_to(tmp_dest)
            .expect("could not download asset")
    })
    .await
    .wrap_err("could not download asset")?;

    // rename from tempfile to dest_path
    fs::rename(tmp_file, dest_path)?;

    // set to read, write, execute in Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dest_path, fs::Permissions::from_mode(0o777))?;
    }

    Ok(())
}

/// Returns the entire list of releases for the given repository, owned by `firstbatchxyz`.
///
/// Due to an [issue](https://github.com/jaemk/self_update/issues/44) of `self_update` not
/// working within async contexts, we do a blocking task spawn here.
///
/// While the returned list is sorted, the latest may not be the first element.
/// Use [`get_latest_release`] to get the latest release instead.
pub(crate) async fn get_releases(repo: DriaRepository) -> Result<Vec<DriaRelease>> {
    let releases = tokio::task::spawn_blocking(move || {
        let mut rel_builder = github::ReleaseList::configure();

        rel_builder
            .repo_owner("firstbatchxyz")
            .repo_name(&repo.to_string())
            .build()
            .expect("could not build ReleaseList")
            .fetch()
            .expect("could not fetch releases")
            .into_iter()
            .map(|r| DriaRelease(r, repo))
            .collect::<Vec<_>>()
    })
    .await
    .wrap_err("could not get releases")?;

    // filter out the launcher releases that are not at least 0.1.0
    if let DriaRepository::Launcher = repo {
        return Ok(releases
            .into_iter()
            .filter(|r| !r.version().starts_with("0.0."))
            .collect());
    }

    Ok(releases)
}

/// Returns the latest release for the given repository.
///
/// Due to an [issue](https://github.com/jaemk/self_update/issues/44) of `self_update` not
/// working within async contexts, we do a blocking task spawn here.
///
/// This respects the `latest` tag, so even if the version tag is lower than the actual latest,
/// it will return the tagged-as-latest release.
pub(crate) async fn get_latest_release(repo: DriaRepository) -> Result<DriaRelease> {
    let result = tokio::task::spawn_blocking(move || {
        github::Update::configure()
            .repo_owner("firstbatchxyz")
            .repo_name(&repo.to_string())
            .bin_name(Default::default()) // ignored within `get_latest_release`
            .current_version(Default::default()) // ignored within `get_latest_release`
            .build()
            .expect("could not build ReleaseUpdate")
            .get_latest_release()
            .map(|r| DriaRelease(r, repo))
            .unwrap()
    })
    .await
    .wrap_err("could not get latest release")?;

    // check if the launcher version is at least 0.1.0
    if let DriaRepository::Launcher = repo {
        if result.version().starts_with("0.0.") {
            return Err(eyre!("latest launcher must be at least 0.1.0"));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::DriaRepository;

    #[tokio::test]
    async fn test_download_latest_compute_release() {
        let final_release = super::get_latest_release(DriaRepository::ComputeNode)
            .await
            .unwrap();

        let path = final_release
            .download_release(
                &PathBuf::from_str(".").unwrap(),
                &final_release.to_filename().unwrap(),
                false,
            )
            .await
            .unwrap();

        assert!(path.exists());
    }

    #[tokio::test]
    async fn test_download_latest_launcher_release() {
        let final_release = super::get_latest_release(DriaRepository::Launcher)
            .await
            .unwrap();

        let path = final_release
            .download_release(
                &PathBuf::from_str(".").unwrap(),
                &final_release.to_filename().unwrap(),
                false,
            )
            .await
            .unwrap();

        assert!(path.exists());
    }
}
