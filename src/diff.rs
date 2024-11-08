use std::{env::current_dir, fs, process::Command, sync::LazyLock};
use unity_version::UnityVersion;

static ARGS: LazyLock<Vec<String>> = LazyLock::new(|| ["-P", "-D", "NET_4_0", "-I"].map(|e| e.to_string()).to_vec());

pub enum PostProcessError {
    NotDownloaded,
    AlreadyPreprocessed,
    InvocationFailed,
    NonUtf8Output,
}

fn post_process(header: String) -> String {
    let start = header
        .find("typedef struct Il2CppClass Il2CppClass;")
        .or(header.find("typedef uint32_t Il2CppMethodSlot;"))
        .expect("Failed to find start offset");
    header.split_at(start).1.to_string()
}

pub fn generate_single_header(unity_version: &UnityVersion) -> Result<(), PostProcessError> {
    let mut compiler_args = ARGS.clone();
    let root = current_dir().unwrap().join(unity_version.major.to_string()).join(unity_version.version());

    if !root.exists() {
        return Err(PostProcessError::NotDownloaded);
    }

    let is_struct_exists = root.join("struct.h").exists();
    let is_api_exists = root.join("api.h").exists();
    if is_api_exists && is_struct_exists {
        return Err(PostProcessError::AlreadyPreprocessed);
    }

    if !is_struct_exists {
        let object_internals = root.join("il2cpp-object-internals.h");
        let global_metadata_file_internals = root.join("vm").join("GlobalMetadataFileInternals.h");

        compiler_args.push(root.to_str().unwrap().to_string());

        if global_metadata_file_internals.exists() {
            compiler_args.push("-include".to_string());
            compiler_args.push(global_metadata_file_internals.to_str().unwrap().to_string());
        }

        compiler_args.push("-include".to_string());
        compiler_args.push(root.join("vm").join("MemoryInformation.h").to_str().unwrap().to_string());
        compiler_args.push("-include".to_string());
        compiler_args.push(object_internals.to_str().unwrap().to_string());

        let struct_runner = Command::new("cpp").args(compiler_args).output().or(Err(PostProcessError::InvocationFailed))?;
        let struct_output = String::from_utf8(struct_runner.stdout).or(Err(PostProcessError::NonUtf8Output))?;
        fs::write(root.join("struct.h"), post_process(struct_output)).unwrap();
    }

    if !is_api_exists {
        let mut api_compiler_args = ARGS.clone();
        api_compiler_args.push(root.to_str().unwrap().to_string());
        api_compiler_args.push("-include".to_string());
        api_compiler_args.push(root.join("il2cpp-api.h").to_str().unwrap().to_string());

        let api_runner = Command::new("cpp")
            .args(api_compiler_args)
            .output()
            .or(Err(PostProcessError::InvocationFailed))?;
        let api_output = String::from_utf8(api_runner.stdout).or(Err(PostProcessError::NonUtf8Output))?;
        fs::write(root.join("api.h"), post_process(api_output)).unwrap();
    }

    Ok(())
}

pub fn diff() {
    assert_eq!(Command::new("./diff.sh").status().unwrap().success(), true);
}
