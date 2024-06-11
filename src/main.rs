use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{
    builder::{
        styling::{AnsiColor, Style},
        Styles,
    },
    Parser, ValueEnum,
};
use console::style;
use materialbin::{CompiledMaterialDefinition, MinecraftVersion};
use scroll::Pread;
use zip::{
    write::{ExtendedFileOptions, FileOptions},
    ZipArchive, ZipWriter,
};
#[derive(Parser)]
#[clap(name = "Material Updater", version = "0.0.1")]
#[command(version, about, long_about = None, styles = get_style())]
struct Options {
    /// Shader pack to update

    #[clap(required = true)]
    file: PathBuf,

    #[clap(required = true)]
    output_ver: MVersion,

    /// Output path
    #[arg(short, long, required = true)]
    output: PathBuf,
}
#[derive(ValueEnum, Clone)]
enum MVersion {
    V1_20_80,
    V1_19_60,
    V1_80_30,
}
impl MVersion {
    fn to_version(self) -> MinecraftVersion {
        match self {
            Self::V1_20_80 => MinecraftVersion::V1_20_80,
            Self::V1_19_60 => MinecraftVersion::V1_19_60,
            Self::V1_80_30 => MinecraftVersion::V1_18_30,
        }
    }
}

const fn get_style() -> Styles {
    Styles::styled()
        .header(AnsiColor::BrightYellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(Style::new().fg_color(None).bold())
        .placeholder(AnsiColor::Green.on_default())
}
fn main() -> anyhow::Result<()> {
    let opts = Options::parse();
    let mcver = opts.output_ver.to_version();
    let pack =
        BufReader::new(File::open(opts.file).with_context(|| "Error while opening input file")?);
    let mut zip = ZipArchive::new(pack)?;
    let output = BufWriter::new(File::create_new(opts.output)?);
    let mut outzip = ZipWriter::new(output);
    let mut translated_shaders = 0;
    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        if !file.name().ends_with(".material.bin") {
            outzip.raw_copy_file(file)?;
            continue;
        }
        print!("Processing file {}", style(file.name()).cyan());
        let mut data = Vec::with_capacity(file.size().try_into()?);
        file.read_to_end(&mut data)?;
        let material = match read_material(&data) {
            Ok(material) => material,
            Err(_) => {
                anyhow::bail!("Material file {} is invalid for all versions", file.name());
            }
        };
        outzip.start_file(file.name(), FileOptions::<ExtendedFileOptions>::default())?;
        material.write(&mut outzip, mcver)?;
        translated_shaders += 1;
    }
    outzip.finish()?;
    println!(
        "Processed {} materials",
        style(translated_shaders.to_string()).green()
    );
    Ok(())
}
fn read_material(data: &[u8]) -> anyhow::Result<CompiledMaterialDefinition> {
    for version in [
        MinecraftVersion::V1_20_80,
        MinecraftVersion::V1_19_60,
        MinecraftVersion::V1_18_30,
    ] {
        if let Ok(material) = data.pread_with(0, version) {
            println!(" [{version}]");
            return Ok(material);
        }
    }

    anyhow::bail!("Material file is invalid");
}
