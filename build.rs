use std::{env::var, path::PathBuf};

use anyhow::Result;
use glob::glob;
use prost_build::compile_protos;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=proto");

    if let Ok(profile) = var("PROFILE") {
        if profile.as_str() == "debug" {
            println!("cargo:rustc-cfg=debug");
        }
    }

    let files = glob("proto/**/*.proto")?
        .filter_map(|result| result.ok())
        .collect::<Vec<PathBuf>>();
    compile_protos(&files, &["proto/"])?;
    Ok(())
}
