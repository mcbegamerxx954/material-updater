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
    read::ZipFile,
    write::{ExtendedFileOptions, FileOptions},
    ZipArchive, ZipWriter,
};
use zune_inflate::DeflateOptions;
#[derive(Parser)]
#[clap(name = "Material Updater", version = "0.1.1")]
#[command(version, about, long_about = None, styles = get_style())]
struct Options {
    /// Shader pack fild to update
    #[clap(required = true)]
    file: PathBuf,

    /// Output zip compression level
    #[clap(short, long)]
    zip_compression: Option<u32>,

    /// Output version
    #[clap(short, long, required = true)]
    target_verion: MVersion,

    /// Output path
    #[arg(short, long, required = true)]
    output: PathBuf,
}
// Hack for clap support
#[derive(ValueEnum, Clone)]
enum MVersion {
    V1_21_20,
    V1_20_80,
    V1_19_60,
    V1_18_30,
}
impl MVersion {
    fn to_version(self) -> MinecraftVersion {
        match self {
            Self::V1_20_80 => MinecraftVersion::V1_20_80,
            Self::V1_19_60 => MinecraftVersion::V1_19_60,
            Self::V1_18_30 => MinecraftVersion::V1_18_30,
            Self::V1_21_20 => MinecraftVersion::V1_21_20,
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
    let mcver = opts.target_verion.to_version();
    let pack =
        BufReader::new(File::open(opts.file).with_context(|| "Error while opening input file")?);
    let mut zip = ZipArchive::new(pack)?;
    let output = BufWriter::new(File::create_new(opts.output)?);
    let mut outzip = ZipWriter::new(output);
    let mut translated_shaders = 0;
    for index in 0..zip.len() {
        let mut file = zip.by_index_raw(index)?;
        if !file.name().ends_with(".material.bin") {
            outzip.raw_copy_file(file)?;
            continue;
        }
        print!("Processing file {}", style(file.name()).cyan());
        let mut data = fast_decompress(&mut file)?;
        let material = match read_material(&data) {
            Ok(material) => material,
            Err(_) => {
                anyhow::bail!("Material file {} is invalid for all versions", file.name());
            }
        };
        let file_options = FileOptions::<ExtendedFileOptions>::default()
            .compression_level(opts.zip_compression.and_then(|v| Some(v.into())));
        outzip.start_file(file.name(), file_options)?;
        material.write(&mut outzip, mcver)?;
        translated_shaders += 1;
    }
    outzip.finish()?;
    println!(
        "Ported {} materials in zip to version {}",
        style(translated_shaders.to_string()).green(),
        style(mcver.to_string()).cyan()
    );
    Ok(())
}
fn fast_decompress(zipfile: &mut ZipFile) -> anyhow::Result<Vec<u8>> {
    let mut output = Vec::with_capacity(zipfile.size().try_into()?);
    let _data = zipfile.read_to_end(&mut output)?;
    let options = DeflateOptions::default().set_size_hint(zipfile.size().try_into()?);
    let mut decoder = zune_inflate::DeflateDecoder::new_with_options(&output, options);
    let decompressed = decoder.decode_deflate()?;
    Ok(decompressed)
}
fn read_material(data: &[u8]) -> anyhow::Result<CompiledMaterialDefinition> {
    for version in materialbin::ALL_VERSIONS {
        if let Ok(material) = data.pread_with(0, version) {
            println!(" [{version}]");
            return Ok(material);
        }
    }

    anyhow::bail!("Material file is invalid");
}
