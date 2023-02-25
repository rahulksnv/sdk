use lazy_static::lazy_static;
use semver::Version;

pub mod cache;

lazy_static! {
    // This expect cannot happen, we make sure that CARGO_PKG_VERSION is correct.
    static ref VERSION: Version =
        Version::parse("0.12.1").expect("Cannot parse version.");

    static ref VERSION_STR: String = "0.12.1".to_string();
}

/// Returns the version of DFX that was built.
/// In debug, add a timestamp of the upstream compilation at the end of version to ensure all
/// debug runs are unique (and cached uniquely).
/// That timestamp is taken from the DFX_TIMESTAMP_DEBUG_MODE_ONLY env var that is set in
/// Nix.
pub fn dfx_version() -> &'static Version {
    &VERSION
}

pub fn dfx_version_str() -> &'static str {
    &VERSION_STR
}
