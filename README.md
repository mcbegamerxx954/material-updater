# Material updater
This tool can update shader packs to latest version.
Or downgrade them as you wish.

parsing code ported to rust from the tool [MaterialBinTool](https://github.com/ddf8196/MaterialBinTool) made by [ddf8196](https://github.com/ddf8196)

## Usage
1. Download binary for your platform from [releases](https://github.com/mcbegamerxx954/material-updater/releases/latest).
2. Extract the archive.
3. If on windows, drag and drop your pack to the exe file, else run the tool in terminal

## Example
``` ./material-updater AF-TrulyDefault-Android.mcpack -t V1-20-80 -o azify.mcpack ```

This command ports the materialbins in the zip file ```AF-TrulyDefault-Android.mcpack``` to 1.20.80 and outputs the result to azify.mcpack, showing the version of the files its processing


