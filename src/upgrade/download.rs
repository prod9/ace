pub fn build_download_url(version: &semver::Version, target: &str) -> String {
    format!(
        "https://github.com/prod9/ace/releases/download/v{version}/ace-{target}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_url_includes_version_and_target() {
        let version = semver::Version::new(0, 4, 0);
        let url = build_download_url(&version, "aarch64-apple-darwin");
        assert_eq!(
            url,
            "https://github.com/prod9/ace/releases/download/v0.4.0/ace-aarch64-apple-darwin"
        );
    }

    #[test]
    fn download_url_adds_v_prefix() {
        let version = semver::Version::new(1, 0, 0);
        let url = build_download_url(&version, "x86_64-unknown-linux-gnu");
        assert!(url.contains("/v1.0.0/"), "URL should contain v-prefixed version: {url}");
    }
}
