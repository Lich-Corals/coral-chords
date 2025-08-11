# Coral-Chords
[![Coffee Logo](https://img.shields.io/badge/-Buy%20me%20a%20coffee-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=black)](https://www.coff.ee/lichcorals)

An application to download and sync tabs from Ultimate Guitar with the music playing on Spotify.

## Features
- Searching for tabs
- Downloading tabs
- Syncing tabs with Spotify

## Installation
To install Coral-Chords, download the [latest release](https://github.com/Lich-Corals/coral-chords/releases) and follow these instructions.

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

## Issues?
If you experience any bugs or have a suggestion about the application, you are welcome to [submit an issue on github](https://github.com/Lich-Corals/coral-chords/issues)!