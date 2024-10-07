use itertools::Itertools;
use markdown_table::*;
use std::path::PathBuf;
use std::{collections::HashMap, fs::read_to_string};

use crate::version_parser::DownloadableUnity;

struct ProcessableUnity<'a> {
    inner: &'a DownloadableUnity,
    metadata_version: u8,
}

fn map_unity_version_to_metadata(unity: &Vec<DownloadableUnity>) -> Vec<ProcessableUnity> {
    let dirs = unity.iter().filter(|e| e.is_downloaded).collect::<Vec<_>>();

    let mut map = Vec::with_capacity(dirs.len());
    for unity_version in dirs {
        let as_pathbuf: PathBuf = <DownloadableUnity as Into<PathBuf>>::into(unity_version.clone());
        if !as_pathbuf.exists() {
            continue;
        }

        let vm = as_pathbuf.join("vm");
        let mut content = read_to_string(vm.join("MetadataCache.cpp")).unwrap();

        let idx = content.find("version == ").unwrap_or_else(|| {
            content = read_to_string(vm.join("GlobalMetadata.cpp")).unwrap();
            content.find("version ==").expect("Failed to determine metadata version assertion index")
        });
        map.push(ProcessableUnity {
            inner: unity_version,
            metadata_version: content[idx + 11..idx + 13].parse().expect("Got non-u8 metadata version"),
        });
    }

    map
}

pub fn create_table(unity: &Vec<DownloadableUnity>) -> impl Iterator<Item = (u16, String)> + use<'_> {
    let heading = ["Unity version", "Metadata version", "Changeset", "Single-header struct", "Single-header API"]
        .map(|e| Heading::new(e.to_string(), Some(HeadingAlignment::Center)))
        .to_vec();

    let mapping = map_unity_version_to_metadata(unity);
    let majors = mapping.iter().map(|e| e.inner.inner.major);
    let majors_nodup = majors.clone().dedup();
    let mut groups = HashMap::with_capacity(8);
    majors_nodup.for_each(|e| {
        groups.insert(e, Vec::with_capacity(majors.clone().filter(|m| e == *m).count()));
    });
    mapping.into_iter().for_each(|unity| {
        let major = unity.inner.inner.major;
        let as_pathbuf: PathBuf = <DownloadableUnity as Into<PathBuf>>::into(unity.inner.clone());
        groups.get_mut(&major).unwrap().push(vec![
            unity.inner.inner.version(),
            unity.metadata_version.to_string(),
            unity.inner.changeset.to_string(),
            format!("Go to [struct.h]({})", as_pathbuf.join("struct.h").to_str().unwrap().to_string()),
            format!("Go to [api.h]({})", as_pathbuf.join("api.h").to_str().unwrap().to_string()),
        ]);
    });
    groups.into_iter().sorted_by_key(|(major, _)| *major).map(move |(major, unitys)| {
        let mut table = MarkdownTable::new(unitys);
        table.with_headings(heading.clone());
        (major, table.as_markdown().unwrap())
    })
}
