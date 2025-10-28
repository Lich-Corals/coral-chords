# Changelog

## [0.6.2]
### Added
- Search-bar lock while searching
### Fixed
- Running search blocking hot-keys
### Updated dependencies
- ug_scraper to 0.2.6
- e_crate_version_checker to 0.1.35

## [0.6.1]
### Added
- Total playtime to statistics
### Fixed
- `Esus*` and `Asus*` being replaced with `Ebus*` and `Abus*` if German chord name replacement was enabled

## [0.6.0]
### Added
- A page to show statistics of the last seven days
- A shortcut to get to the statistics (Digit4)
- A button next-too the settings button in the bar to get to the statistics (only visible on settings page)
### Updated dependencies
- serde to 1.0.228

## [0.5.3]
### Added
- A switch to toggle some advanced settings, making the settings page cleaner
### Updated dependencies
- serde to 1.0.226

## [0.5.2]
### Added
- Option to create a blanc file if no search results could be found
- Launch search when 'Enter' is pressed on search screen
### Changed dependencies
- Removed serde default features
### Updated dependencies
- serde to 1.0.221

## [0.5.1]
### Added
- Keyboard shortcuts for next/previous songs

## [0.5.0]
### Added
- Keyboard shortcuts to control the media player
- Keyboard shortcuts to control the UI
- Keyboard shortcuts to edit and reload the current tab
### Code-changes
- Split off some UI-builders into separate files 
### Updated dependencies
- opener to 0.8.3

## [0.4.1]
### Updated
- ug-scraper dependency
### Fixed
- Chord lines getting interpreted as section headers if square brackets were present in them

## [0.4.0]
### Added
- A welcome screen for new users
### Fixed
- Full tabs getting hidden if no chords are found
- Tab rows getting split if the page length limit is reached

## [0.3.0]
### Added
- An optional feature to remind the user of replacing their instrument's strings regularly

## [0.2.3]
### Fixed
- Not switching back to tab view if downloaded song is played and search view active

## [0.2.2]
### Added
- Error messages if the song ID is available or in an invalid format
### Fixed
- Possible wrong player selection if another MPRIS player is running (e.g.: Youtube in Firefox)

## [0.2.1]
### Added
- A setting to change the colour of the optional metadata at the top of the displayed tab

## [0.2.0]
### Added
- Local logging of played songs
- Setting to disable the feature above

## [0.1.4]
### Added
- Notification if newer version of package is available on crates.io
- Setting to disable the feature above
### Fixed
- Settings not scrollable when the window is too small
### Added dependencies
- e_crate_version_checker

## [0.1.3]
### Added
- Automatic return to "Tab" view after a download has finished while "play" is enabled
### Fixed
- Invalid search result URLs when tab name contains numbers
- Search depth specifier having a minimum of two

## [0.1.0]
### Added
- Application
