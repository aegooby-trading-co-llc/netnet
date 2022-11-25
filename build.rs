use std::path::PathBuf;

use anyhow::Result;
use glob::glob;
use prost_build::compile_protos;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=proto");
    let files = glob("proto/**/*.proto")?
        .filter_map(|result| result.map_or(None, |path| Some(path)))
        .collect::<Vec<PathBuf>>();
    compile_protos(&files, &["proto/"])?;
    Ok(())
}
