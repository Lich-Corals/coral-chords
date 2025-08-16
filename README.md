# Coral-Chords
[![GitHub](https://img.shields.io/badge/-GitHub-181717?style=for-the-badge&logo=GitHub&logoColor=white)](https://github.com/Lich-Corals/coral-chords)
[![Crates](https://img.shields.io/badge/-Crates.io-ffc933?style=for-the-badge&logo=rust&logoColor=black)](https://crates.io/crates/Coral-Chords)
[![Coffee Logo](https://img.shields.io/badge/-Buy%20me%20a%20coffee-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=black)](https://www.coff.ee/lichcorals)

An application to download and sync tabs from Ultimate Guitar with the music playing on Spotify.

<p align="center">
  <img alt="cch_themes_transp" src="https://github.com/user-attachments/assets/5ee115fb-6d6b-41b2-a81b-200df17d54b2" />
  <br/>
  Tab sync with Spotify and an integrated search page for tabs; easy to configure and with a wide variety of themes and customization options
</p>

## Features
- Searching for tabs
- Downloading tabs
- Syncing tabs with Spotify

## Installation

> [!IMPORTANT]  
> Your system needs to use the D-Bus to be able to run this application.
> This means it is mainly built for Linux based systems, although it may run on macOS too if the D-Bus is installed.

You can install Coral-Chords using `cargo` or manually.

### Cargo installation
Just run the following command to install:
```bash
cargo install Coral-Chords
```
You can now run `Coral-Chords` from your terminal to launch the application.
If you want to have an entry for your system's application launcher, you need to create it yourself or follow the manual installation steps.

### Manual installation
Download and extract the [latest release](https://github.com/Lich-Corals/coral-chords/releases) and follow those instructions:

> [!NOTE]
> You can also build the binary yourself using `cargo build --release` in the cloned project directory.
> Building this package yourself requires the development tools of `openssl` to be installed using your system's package manager.

1. Extract the downloaded file
2. Copy the binary file to `./local/bin` with the name `coral_chords`:
```bash
cp Coral-Chords ~/.local/bin/coral_chords
```
3. Copy the .desktop file to `.local/share/applications/coral-chords.desktop:
```bash
cp coral-chords.desktop ~/.local/share/applications/coral-chords.desktop
```
4. Make both files executable:
```bash
chmod +x ~/.local/share/applications/coral-chords.desktop ~/.local/bin/coral_chords
```

Now you should be able to launch the application using your system's default application launcher.

## Documentation
The code is widely commented with cargo-doc compatible comments for you to either read them as they are or to build the documentation using `cargo doc --no-deps`.
A more general documentation and feature guide is available [here](https://github.com/Lich-Corals/coral-chords/blob/mistress/latex/documentation.pdf).

## Issues?
If you experience any bugs or have a suggestion about the application, you are welcome to [submit an issue on GitHub](https://github.com/Lich-Corals/coral-chords/issues)!
