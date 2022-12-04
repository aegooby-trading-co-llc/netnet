use std::{
    env::{set_var, var},
    fs::{create_dir, read_dir, remove_dir_all, write},
    path::PathBuf,
};

use anyhow::{Error, Result};
use glob::glob;
use prost_build::compile_protos;

fn main() -> Result<()> {
    set_var("OUT_DIR", "src/gen");
    println!("cargo:rerun-if-changed=proto");
    if let Ok(profile) = var("PROFILE") {
        if profile.as_str() == "debug" {
            println!("cargo:rustc-cfg=debug");
        }
    }

    let out_dir = var("OUT_DIR")?;
    let _ = remove_dir_all(out_dir.clone());
    create_dir(out_dir.clone())?;
    let files = glob("proto/**/*.proto")?
        .filter_map(|result| result.ok())
        .collect::<Vec<PathBuf>>();
    compile_protos(&files, &["proto/"])?;
    let contents = read_dir(out_dir.clone())?
        .filter_map(|entry| {
            if entry.as_ref().ok()?.metadata().ok()?.is_file() {
                let modname = entry
                    .ok()?
                    .file_name()
                    .into_string()
                    .ok()?
                    .replace(".rs", "");
                Some(format!("pub mod {modname};\n"))
            } else {
                None
            }
        })
        .reduce(|acc, each| format!("{acc}{each}"))
        .ok_or(Error::msg("failed to generate modules"))?;
    write(PathBuf::new().join(out_dir).join("mod.rs"), contents)?;
    Ok(())
}
