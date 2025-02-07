use std::env::consts::{ARCH, OS};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use eyre::{eyre, Context, Result};
use self_update::backends::github;
use self_update::update::{Release, ReleaseAsset};

#[derive(Debug, Clone)]
pub struct DriaRelease(Release);

impl DriaRelease {
    #[inline]
    pub fn name(&self) -> &str {
        &self.0.name
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
    pub fn asset(&self) -> Result<&ReleaseAsset> {
        self.0
            .assets
            .iter()
            .find(|asset| {
                let os = match OS {
                    "windows" | "linux" | "macos" => OS,
                    _ => return false,
                };

                let arch = match ARCH {
                    "x86_64" => "amd64",
                    "arm" | "aarch64" => "arm64",
                    _ => return false,
                };
                let name_lowercase = asset.name.to_lowercase();
                name_lowercase.contains(os) && name_lowercase.contains(arch)
            })
            .ok_or(eyre!("asset not found for OS: {} & ARCH: {}", OS, ARCH))
    }

    pub async fn download_release(&self) -> Result<PathBuf> {
        let asset = self.asset()?;
        let path = PathBuf::from(&asset.name);
        let dest_file = fs::File::create(&path)?;

        let download_url = asset.download_url.clone();
        println!("Downloading {}", download_url);
        tokio::task::spawn_blocking(move || {
            self_update::Download::from_url(&download_url)
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
        fs::set_permissions(&path, fs::Permissions::from_mode(0o777))?;

        // let bin_name = std::path::PathBuf::from(asset.name.clone());
        // self_update::Extract::from_source(&tmp_file)
        // .archive(self_update::ArchiveKind::Tar(Some(
        //     self_update::Compression::Gz,
        // )))
        // .extract_file(&tmp_dir.path(), &bin_name)?;

        // let new_exe = tmp_dir.path().join(bin_name);
        // println!("Downloaded at: {}", new_exe.display());
        // self_replace::self_replace(new_exe)?;

        Ok(path)
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
            .map(|r| DriaRelease(r))
            .collect::<Vec<_>>()
    })
    .await
    .map_err(Into::into)
}

/// Downloads an asset to the given file.

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_compute_releases() {
        let releases = super::get_compute_releases().await.unwrap();
        assert!(releases.len() > 20);
        println!("{:#?}", releases[0]);
    }

    #[tokio::test]
    async fn test_launcher_releases() {
        let releases = super::get_launcher_releases().await.unwrap();
        println!("{:#?}", releases.last().unwrap());
    }

    #[tokio::test]
    async fn test_download_last_release() {
        let final_release = &super::get_compute_releases().await.unwrap()[0];

        let path = final_release.download_release().await.unwrap();
        assert!(path.exists());
    }
}
