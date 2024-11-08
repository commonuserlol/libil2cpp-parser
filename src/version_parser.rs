use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::path::PathBuf;
use unity_version::UnityVersion;

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

impl From<(String, String)> for DownloadableUnity {
    fn from((inner, changeset): (String, String)) -> Self {
        DownloadableUnity {
            inner: UnityVersion::from(inner.as_str()),
            changeset,
            is_downloaded: false,
        }
    }
}

impl Into<PathBuf> for DownloadableUnity {
    fn into(self) -> PathBuf {
        PathBuf::from(format!("{}/{}", self.inner.major, self.inner.version()))
    }
}

pub fn parse_unity_versions(pairs: Vec<(String, String)>) -> Vec<DownloadableUnity> {
    let mut unity_versions = pairs.into_iter().map(|e| DownloadableUnity::from(e)).collect::<Vec<_>>();
    unity_versions.sort();
    unity_versions.dedup();

    unity_versions
}
