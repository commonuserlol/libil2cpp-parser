use regex::Regex;

use crate::unity_version::UnityVersion;
use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::path::PathBuf;
use std::sync::LazyLock;

static UNITYHUB_VERSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"unityhub:\/\/([0-9]+)\.([0-9]+)\.([0-9]+)\.?([abcfpx]+)([0-9]+)\/[0-9a-z]{12}").unwrap());

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct DownloadableUnity {
    pub inner: UnityVersion,
    pub changeset: String,
    pub is_downloaded: bool,
}

impl Display for DownloadableUnity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.inner, self.changeset)
    }
}

impl Ord for DownloadableUnity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl From<&str> for DownloadableUnity {
    fn from(value: &str) -> Self {
        let mut r#match = value.split("/");
        r#match.next();
        r#match.next();
        let version = r#match.next().unwrap();
        let changeset = r#match.next().unwrap();

        DownloadableUnity {
            inner: UnityVersion::from(version),
            changeset: changeset.to_string(),
            is_downloaded: false,
        }
    }
}

impl Into<PathBuf> for DownloadableUnity {
    fn into(self) -> PathBuf {
        PathBuf::from(format!("{}/{}", self.inner.major, self.inner.version()))
    }
}

pub fn parse_unity_versions(text: &String) -> Vec<DownloadableUnity> {
    let mut unity_versions = UNITYHUB_VERSION_REGEX
        .find_iter(text.as_str())
        .map(|e| DownloadableUnity::from(e.as_str()))
        .collect::<Vec<_>>();
    unity_versions.sort();
    unity_versions.dedup();

    unity_versions
}
