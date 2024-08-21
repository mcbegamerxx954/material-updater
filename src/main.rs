use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, Write},
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
use tempfile::tempfile;
use zip::{
    write::{ExtendedFileOptions, FileOptions},
    ZipArchive, ZipWriter,
};

#[derive(Parser)]
#[clap(name = "Material Updater", version = "0.1.11")]
#[command(version, about, long_about = None, styles = get_style())]
struct Options {
    /// Shader pack fild to update
    #[clap(required = true)]
    file: String,

    /// Output zip compression level
    #[clap(short, long)]
    zip_compression: Option<u32>,

    /// Process the file, but dont write anything
    #[clap(short, long)]
    yeet: bool,

    /// Output version
    #[clap(short, long)]
    target_version: Option<MVersion>,

    /// Output path
    #[arg(short, long)]
    output: Option<PathBuf>,
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
    const fn as_version(&self) -> MinecraftVersion {
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
    let mcversion = match opts.target_version {
        Some(version) => version.as_version(),
        None => {
            const STABLE_LATEST: MinecraftVersion = MinecraftVersion::V1_21_20;
            println!(
                "No target version specified, updating to latest stable: {}",
                STABLE_LATEST
            );
            STABLE_LATEST
        }
    };
    let mut input_file =
        BufReader::new(File::open(&opts.file).with_context(|| "Error while opening input file")?);
    if opts.file.ends_with(".material.bin") {
        let output_filename: PathBuf = match &opts.output {
            Some(output_name) => output_name.to_owned(),
            None => {
                let auto_name = update_filename(&opts.file, &mcversion, ".material.bin")?;
                println!("No output name specified, using {auto_name:?}");
                auto_name
            }
        };
        let mut tmp_file = tempfile()?;
        let mut output_file = file_to_shrodinger(&mut tmp_file, opts.yeet)?;
        println!("Processing input {}", style(opts.file).cyan());
        file_update(&mut input_file, &mut output_file, mcversion)?;
        tmp_file.rewind()?;
        if !opts.yeet {
            let mut output_file = File::create(output_filename)?;
            io::copy(&mut tmp_file, &mut output_file)?;
        }
        return Ok(());
    }
    if opts.file.ends_with(".zip") || opts.file.ends_with(".mcpack") {
        let extension = Path::new(&opts.file)
            .extension()
            .with_context(|| "Input file does not have any extension??, weird")?
            // At this point its valid utf8 soo
            .to_str()
            .unwrap();
        let extension = ".".to_string() + extension;
        let output_filename: PathBuf = match &opts.output {
            Some(output_name) => output_name.to_owned(),
            None => {
                let auto_name = update_filename(&opts.file, &mcversion, &extension)?;
                println!("No output name specified, using {auto_name:?}");
                auto_name
            }
        };
        let mut tmp_file = tempfile()?;
        let mut output_file = file_to_shrodinger(&mut tmp_file, opts.yeet)?;
        println!("Processing input zip {}", style(opts.file).cyan());
        zip_update(
            &mut input_file,
            &mut output_file,
            mcversion,
            opts.zip_compression,
        )?;
        tmp_file.rewind()?;
        if !opts.yeet {
            let mut output_file = File::create(output_filename)?;
            io::copy(&mut tmp_file, &mut output_file)?;
        }
    }
    Ok(())
}
fn file_to_shrodinger<'a>(
    file: &'a mut File,
    dissapear: bool,
) -> anyhow::Result<ShrodingerOutput<'a>> {
    if dissapear {
        Ok(ShrodingerOutput::Nothing)
    } else {
        Ok(ShrodingerOutput::File(file))
    }
}
fn update_filename(
    filename: &str,
    version: &MinecraftVersion,
    postfix: &str,
) -> anyhow::Result<PathBuf> {
    let stripped = filename
        .strip_suffix(postfix)
        .with_context(|| "String does not contain expected postfix")?;
    Ok((stripped.to_string() + "_" + &version.to_string() + postfix).into())
}
fn file_update<R, W>(input: &mut R, output: &mut W, version: MinecraftVersion) -> anyhow::Result<()>
where
    R: Read + Seek,
    W: Write + Seek,
{
    let mut data = Vec::new();
    let _read = input.read_to_end(&mut data)?;
    let material = read_material(&data)?;
    material.write(output, version)?;
    Ok(())
}
fn zip_update<R, W>(
    input: &mut R,
    output: &mut W,
    version: MinecraftVersion,
    compression_level: Option<u32>,
) -> anyhow::Result<()>
where
    R: Read + Seek,
    W: Write + Seek,
{
    let mut input_zip = ZipArchive::new(input)?;
    let mut output_zip = ZipWriter::new(output);
    let mut translated_shaders = 0;
    for index in 0..input_zip.len() {
        let mut file = input_zip.by_index(index)?;
        if !file.name().ends_with(".material.bin") {
            output_zip.raw_copy_file(file)?;
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
        let file_options = FileOptions::<ExtendedFileOptions>::default()
            .compression_level(compression_level.map(|v| v.into()));
        output_zip.start_file(file.name(), file_options)?;
        material.write(&mut output_zip, version)?;
        translated_shaders += 1;
    }
    output_zip.finish()?;
    println!(
        "Ported {} materials in zip to version {}",
        style(translated_shaders.to_string()).green(),
        style(version.to_string()).cyan()
    );
    Ok(())
}

fn read_material(data: &[u8]) -> anyhow::Result<CompiledMaterialDefinition> {
    for version in materialbin::ALL_VERSIONS {
        if let Ok(material) = data.pread_with(0, version) {
            println!("{}", style(format!(" [{version}]")).dim());
            return Ok(material);
        }
    }

    anyhow::bail!("Material file is invalid");
}
enum ShrodingerOutput<'a> {
    File(&'a mut File),
    Nothing,
}
impl<'a> Write for ShrodingerOutput<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::File(f) => f.write(buf),
            Self::Nothing => Ok(buf.len()),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::File(f) => f.flush(),
            Self::Nothing => Ok(()),
        }
    }
}
impl<'a> Seek for ShrodingerOutput<'a> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::File(f) => f.seek(pos),
            Self::Nothing => Ok(0),
        }
    }
}
