# Material updater
This tool can update shader packs to latest version. *Or downgrade them as you wish.*

Parsing code ported to rust from the tool [MaterialBinTool](https://github.com/ddf8196/MaterialBinTool) made by [ddf8196](https://github.com/ddf8196)

## Usage
1. Download binary for your platform from [releases](https://github.com/mcbegamerxx954/material-updater/releases/latest).
2. Extract the archive.
3. If on windows, drag and drop your pack to the exe file, else run the tool in terminal

<!--
<pre>
Usage: material-updater [OPTIONS] <FILE>

Arguments:
  <FILE>  Shader pack file to update

Options:
  -z, --zip-compression <ZIP_COMPRESSION>
          Output zip compression level
  -y, --yeet
          Process the file, but dont write anything
  -t, --target-version <TARGET_VERSION>
          Output version [possible values: v1-21-20, v1-20-80, v1-19-60, v1-18-30]
  -o, --output <OUTPUT>
          Output path
  -h, --help
          Print help
  -V, --version
          Print version

</pre>
-->

## Example
`./material-updater AF-TrulyDefault-Android.mcpack -t v1-20-80 -o azify.mcpack`

This command ports the materialbins in the zip file `AF-TrulyDefault-Android.mcpack` to 1.20.80 materials and outputs the result to azify.mcpack, showing the version of the files its processing.  

> **Versions you can use:**  
> `v1-21-20` `v1-20-80` `v1-19-60` `v1-18-30` 