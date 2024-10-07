use colour::*;
use futures_util::StreamExt;
use reqwest::header::USER_AGENT;
use reqwest::{get, Client};
use std::io::Write;

use crate::unity_version::UnityVersion;
use crate::version_parser::DownloadableUnity;

const UNITY_ARCHIVE_URL: &'static str = "https://unity.com/en/releases/editor/archive";
const LINUX_DOWNLOAD_PATTERN: &'static str = "<span class=\"truncate\">Linux</span>";

pub async fn fetch_archive() -> Result<String, String> {
    Ok(get(UNITY_ARCHIVE_URL)
        .await
        .or(Err("Failed to fetch unity archive"))?
        .text()
        .await
        .or(Err("Failed to parse text"))?)
}

pub async fn is_available_for_linux(unity_version: &UnityVersion) -> Result<bool, String> {
    let url = match unity_version.r#type {
        'a' => format!("https://unity.com/en/releases/editor/alpha/{}", unity_version.version()),
        'b' => format!("https://unity.com/en/releases/editor/beta/{}", unity_version.version()),
        'c' => panic!("We don't handle china unity!"),
        'f' | 'p' => format!(
            "https://unity.com/en/releases/editor/whats-new/{}.{}.{}",
            unity_version.major, unity_version.minor, unity_version.build
        ),
        'x' | _ => unreachable!(),
    };

    let client = Client::new();
    let response = client
        .get(url)
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 Edg/127.0.2651.105",
        )
        .send()
        .await
        .or(Err(format!("Failed to get {}", unity_version)))?;

    Ok(response.text().await.or(Err("Failed to parse text"))?.contains(LINUX_DOWNLOAD_PATTERN))
}

pub async fn download_using_changeset(unity_version: &DownloadableUnity) -> Result<Vec<u8>, String> {
    let url = format!(
        "https://download.unity3d.com/download_unity/{}/LinuxEditorInstaller/Unity-{}.tar.xz",
        unity_version.changeset,
        unity_version.inner.version()
    );
    let client = Client::new();
    let response = client
        .get(url)
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 Edg/127.0.2651.105",
        )
        .send()
        .await
        .or(Err(format!("Failed to get {}", unity_version)))?;
    match response.error_for_status() {
        Err(e) => Err(e.to_string()),
        Ok(res) => {
            let total_size = res.content_length().unwrap();

            yellow_ln!("Downloading {} with {:.2}MB", unity_version, total_size / 1024 / 1024);

            let mut buffer = Vec::with_capacity(total_size as usize);
            let mut stream = res.bytes_stream();
            let start = std::time::Instant::now();

            while let Some(item) = stream.next().await {
                let chunk = item.expect("Failed downloading chunk");
                let _ = buffer.write(&chunk).or(Err("Failed write to memory"))?;
            }

            green_ln!("✔ Downloaded {} in {}s", unity_version, start.elapsed().as_secs());

            Ok(buffer)
        }
    }
}
