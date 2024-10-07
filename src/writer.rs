use std::{
    borrow::Cow,
    fs,
    io::{ErrorKind, Read},
    path::Path,
};
use tar::Archive;
use xz2::read::XzDecoder;

use crate::unity_version::UnityVersion;

pub struct LibIl2CppWriter<'a> {
    unity_version: &'a UnityVersion,
    buffer: Vec<u8>,
}

impl<'a> LibIl2CppWriter<'a> {
    pub fn new(unity_version: &'a UnityVersion, buffer: Vec<u8>) -> Self {
        Self { unity_version, buffer }
    }

    #[inline]
    fn output_directory(&self) -> String {
        format!("{}/{}", self.unity_version.major, self.unity_version.version())
    }

    #[inline]
    fn format_output_directory(&self, raw_path: &Cow<'_, Path>) -> String {
        format!(
            "{}/{}",
            self.output_directory(),
            raw_path.as_os_str().to_str().unwrap().replace("Editor/Data/il2cpp/libil2cpp/", "")
        )
    }

    fn ensure_version(&self) -> Result<(), &str> {
        fs::create_dir_all(self.output_directory()).or(Err("Failed to create directory"))
    }

    pub fn write(&self) -> Result<(), String> {
        let decoder = XzDecoder::new(self.buffer.as_slice());
        let mut tar = Archive::new(decoder);

        let _ = self.ensure_version().or(Err("Failed to create version directory"));

        for entry in tar.entries().unwrap() {
            let raw_entry = entry.unwrap();
            let raw_path = raw_entry.path().unwrap();

            if !raw_path.to_str().unwrap().contains("libil2cpp") {
                continue;
            }
            let output = self.format_output_directory(&raw_path);
            let size = raw_entry.size();

            let mut buffer = vec![];
            let _ = raw_entry.take(size).read_to_end(&mut buffer).unwrap();

            match fs::write(&output, buffer) {
                Ok(_) => {}
                Err(e) => {
                    match e.kind() {
                        ErrorKind::IsADirectory => {
                            let _ = fs::create_dir_all(output).unwrap();
                        }
                        _ => unreachable!(),
                    }
                    continue;
                }
            }
        }

        Ok(())
    }
}
