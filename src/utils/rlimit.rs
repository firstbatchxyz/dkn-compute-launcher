/// Sets the resource limits for the process (`uname -n`).
///
/// We use the [`rlimit`] crate to set the resource limits for the process,
/// which internally uses the `getrlimit` and `setrlimit` system calls.
///
/// See more info about this [here](https://www.gnu.org/software/libc/manual/html_node/Limits-on-Resources.html).
///
/// Note that if `soft` limit is greater than `hard` limit it will require admin privileges to set the limits.
#[inline]
pub fn configure_rlimit() {
    // set file-descriptor limits in Unix, not needed in Windows
    #[cfg(unix)]
    {
        use rlimit::{setrlimit, Resource};

        // default hard limits, in case `getrlimit` fails
        #[cfg(target_os = "macos")]
        const DEFAULT_HARD_LIMIT: u64 = 40 * 1024 * 1024; // usually unlimited, but this is a good default
        #[cfg(not(target_os = "macos"))]
        const DEFAULT_HARD_LIMIT: u64 = 1024 * 1024; // 1048576

        // defaults soft limits, in case `getrlimit` fails
        #[cfg(target_os = "macos")]
        const DEFAULT_SOFT_LIMIT: u64 = (1024 * 1024) - 1; // 1048575
        #[cfg(not(target_os = "macos"))]
        const DEFAULT_SOFT_LIMIT: u64 = 1024;

        let (soft, hard) = Resource::NOFILE
            .get()
            .unwrap_or((DEFAULT_SOFT_LIMIT, DEFAULT_HARD_LIMIT));

        // we target the soft limit to be 10% of the hard limit
        let target_soft = hard / 10;

        // only do this if soft-limit is below the target
        if soft < target_soft {
            if let Err(e) = setrlimit(Resource::NOFILE, DEFAULT_SOFT_LIMIT, DEFAULT_HARD_LIMIT) {
                log::error!(
                "Failed to set file-descriptor limits: {}, you may need to run as administrator!",
                e
            );
            } else {
                log::warn!(
                    "Using new resource limits (soft / hard): {} / {}",
                    soft,
                    hard
                );
            }
        } else {
            log::info!(
                "Using existing resource limits (soft / hard): {} / {}",
                soft,
                hard
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_rlimit() {
        let _ = env_logger::builder()
            .filter(None, log::LevelFilter::Off)
            .filter_module("dkn_compute_launcher", log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        configure_rlimit();
    }
}
