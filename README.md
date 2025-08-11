# Coral-Chords
[![Coffee Logo](https://img.shields.io/badge/-Buy%20me%20a%20coffee-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=black)](https://www.coff.ee/lichcorals)

An application to download and sync tabs from Ultimate Guitar with the music playing on Spotify.

<p align="center">
  <img alt="cch_themes_transp" src="https://github.com/user-attachments/assets/5ee115fb-6d6b-41b2-a81b-200df17d54b2" />
  <br/>
  Tab sync with Spotify and an integrated search page for tabs; easy to configure and with a wide variety of themes customization options
</p>

## Features
- Searching for tabs
- Downloading tabs
- Syncing tabs with Spotify

## Installation

> IMPORTANT NOTICE  
> Your system needs to use the D-Bus to be able to run this application.

To install Coral-Chords, download the [latest release](https://github.com/Lich-Corals/coral-chords/releases) and follow these instructions.
> [!NOTE]
> You can also build the binary yourself using `cargo build --release` in the cloned project directory.

1. Extract the downloaded file
2. Copy the binary file to `./local/bin` with the name `coral_chords`:
```bash
cp coral_chords ~/.local/bin/coral_chords
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
