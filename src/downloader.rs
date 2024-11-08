use colour::*;
use futures_util::StreamExt;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::version_parser::DownloadableUnity;
use unity_version::{UnityVersion, UnityVersionType};

const GRAPHQL_URL: &'static str = "https://services.unity.com/graphql";
const VERSIONS: [&'static str; 9] = ["5", "2017", "2018", "2019", "2020", "2021", "2022", "2023", "6000"];
const LINUX_DOWNLOAD_PATTERN: &'static str = "<span class=\"truncate\">Linux</span>";

#[derive(Serialize)]
struct GraphQLPayloadVariables {
    pub limit: u16,
    pub version: String,
}

#[derive(Serialize)]
struct GraphQLPayload {
    #[serde(rename = "operationName")]
    pub operation_name: String,
    pub query: String,
    pub variables: GraphQLPayloadVariables,
}

#[derive(Deserialize)]
struct GraphQLReponseNode {
    version: String,
    #[serde(rename = "unityHubDeepLink")]
    unity_hub_url: String,
}

#[derive(Deserialize)]
struct GraphQLResponseEdge {
    node: GraphQLReponseNode,
}

#[derive(Deserialize)]
struct GraphQLResponseUnityReleases {
    edges: Vec<GraphQLResponseEdge>,
}

#[derive(Deserialize)]
struct GraphQLResponseDummy {
    #[serde(rename = "getUnityReleases")]
    get_unity_releases: GraphQLResponseUnityReleases,
}

#[derive(Deserialize)]
struct _GraphQLResponseDummy {
    data: GraphQLResponseDummy,
}

pub async fn fetch_versions() -> Vec<(String, String)> {
    let mut vec = Vec::with_capacity(1500);

    for version in VERSIONS {
        let payload = GraphQLPayload {
            operation_name: "GetRelease".into(),
            query: r#"query GetRelease($limit: Int, $skip: Int, $version: String!, $stream: [UnityReleaseStream!]) {getUnityReleases(limit: $limit skip: $skip stream: $stream version: $version entitlements: [XLTS]) {totalCount edges {node {version entitlements releaseDate unityHubDeepLink  stream }}}}"#.into(),
            variables: GraphQLPayloadVariables { limit: 1000, version: version.into() }
        };
        let client = Client::new();
        let content = client
            .post(GRAPHQL_URL)
            .body(serde_json::to_string(&payload).unwrap())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let data: _GraphQLResponseDummy = serde_json::from_str(content.as_str()).unwrap();
        let total = data.data.get_unity_releases;
        total.edges.into_iter().for_each(|edge| {
            let node = edge.node;
            let lifetime = format!("://{}/", node.version);
            let mut changeset_split = node.unity_hub_url.split(lifetime.as_str());
            vec.push((node.version, changeset_split.nth(1).unwrap().to_string()))
        });
    }

    vec
}

pub async fn is_available_for_linux(unity_version: &UnityVersion) -> Result<bool, String> {
    let url = match unity_version.r#type {
        UnityVersionType::Alpha => format!("https://unity.com/en/releases/editor/alpha/{}", unity_version.version()),
        UnityVersionType::Beta => format!("https://unity.com/en/releases/editor/beta/{}", unity_version.version()),
        UnityVersionType::Final | UnityVersionType::Patch => format!(
            "https://unity.com/en/releases/editor/whats-new/{}.{}.{}",
            unity_version.major, unity_version.minor, unity_version.build
        ),
        _ => todo!(),
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
