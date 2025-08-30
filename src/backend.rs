// Coral-Chords - Automatic chords for Spotify
// Copyright (C) 2025  Linus Tibert
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public Licence as published
// by the Free Software Foundation, either version 3 of the Licence, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public Licence for more details.
//
// You should have received a copy of the GNU Affero General Public Licence
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

/// Access the local file system
pub mod system {
        use crate::backend::formats::{CoralConfig, LoggedSong, TAB_DIR};
        use confy::{self, get_configuration_file_path, ConfyError};
        use std::{
                path::PathBuf,
                time::{Duration, SystemTime, UNIX_EPOCH},
        };
        use ug_scraper::types::*;

        /// Save a given song as a local file
        pub fn store_song(song: Song, song_uid: &str) -> Result<(), ConfyError> {
                confy::store(
                        "Coral-Chords",
                        Some((TAB_DIR.to_string() + "/" + song_uid).as_str()),
                        song,
                )?;
                Ok(())
        }

        /// Read a requested song file
        pub fn load_song(song_uid: &str) -> Result<Song, ConfyError> {
                confy::load::<Song>(
                        "Coral-Chords",
                        Some((TAB_DIR.to_string() + "/" + song_uid).as_str()),
                )
        }

        /// Returns the current system time as UNIX time stamp or 1 if an error occurs
        pub fn current_time() -> u64 {
                SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_else(|_| Duration::from_secs(1))
                        .as_secs()
        }

        /// Get the local song log
        pub fn get_song_log() -> (Vec<LoggedSong>, Option<ConfyError>) {
                let result = confy::load::<Vec<LoggedSong>>("Coral-Chords", Some("song_log"));
                match result {
                        Ok(c) => (c, None),
                        Err(e) => (vec![], Some(e)),
                }
        }

        /// Add a song to the log
        pub fn add_to_song_log(song: LoggedSong) -> Result<(), ConfyError> {
                let mut song_log_data = get_song_log();
                if let Some(e) = song_log_data.1 {
                        return Err(e);
                }
                song_log_data.0.push(song);
                confy::store("Coral-Chords", Some("song_log"), song_log_data.0)?;
                Ok(())
        }

        /// Read the config file or return default
        pub fn get_config() -> (CoralConfig, Option<ConfyError>) {
                let result = confy::load::<CoralConfig>("Coral-Chords", Some("config"));
                match result {
                        Ok(c) => (c, None),
                        Err(e) => (CoralConfig::default(), Some(e)),
                }
        }

        /// Write the config file
        pub fn set_config(config: CoralConfig) -> Result<(), ConfyError> {
                confy::store("Coral-Chords", Some("config"), config)?;
                Ok(())
        }

        pub fn get_tab_path() -> Result<PathBuf, ConfyError> {
                get_configuration_file_path("Coral-Chords", Some(TAB_DIR))
        }
}

/// Formats used by Coral-Chords
pub mod formats {
        use crate::backend::system::set_config;
        use confy::ConfyError;
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;
        use std::fmt;
        use ug_scraper::types::*;

        /// The subdirectory of the config path where tab files are stored.
        pub const TAB_DIR: &str = "tabs";

        /// Possible types of notifications which may be shown to the user
        #[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
        pub enum NotificationType {
                #[default]
                /// This does nothing but allowing the `Default` trait
                Default,
                /// Casual information
                Info(String),
                /// An important warning; possibly solvable by the user
                Warning(String),
                /// An error; this is most likely a problem in the code or with the user giving an invalid input
                Error(String),
                /// An error which will close the application after being read.
                Fatal(String),
                /// The strings have to be changed.
                RenewStrings(String),
        }

        impl fmt::Display for NotificationType {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "{}", self.to_owned())
                }
        }

        /// A wrapper to store different kinds of data in a single HashMap
        ///
        /// Used to store values in the configuration file
        #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
        pub enum Value {
                #[default]
                None,
                String(String),
                Bool(bool),
                Int(i64),
                Float(f64),
                DataSetTypeOption(Option<DataSetType>),
        }

        impl fmt::Display for Value {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:?}", self)
                }
        }

        /// To send data between threads
        #[derive(Debug, Clone, Default, PartialEq)]
        pub enum ThreadData {
                #[default]
                None,
                Notification(NotificationType),
                SearchResults(Vec<SearchResult>),
                DownloadFinishedSignal,
        }

        impl fmt::Display for ThreadData {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:?}", self)
                }
        }

        /// The type used as a song in the song log
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct LoggedSong {
                pub name: String,
                pub artist: String,
                pub id: String,
                pub length_s: u64,
                pub timestamp: u64,
        }

        /// The configuration used by CCh
        ///
        /// Uses HashMaps because old settings would be deleted when new ones are added to existing config files.
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct CoralConfig {
                pub config: HashMap<String, Value>,
        }

        impl Default for CoralConfig {
                fn default() -> CoralConfig {
                        CoralConfig {
                                config: HashMap::from([
                                        ("theme".into(), Value::String("Theme::Dark".into())),
                                        ("search_depth".into(), Value::Int(2)),
                                        ("search_filter".into(), Value::DataSetTypeOption(None)),
                                        ("only_downloadable_results".into(), Value::Bool(false)),
                                        ("tab_text_size".into(), Value::Int(15)),
                                        ("chord_colour".into(), Value::String("#fe640b".into())),
                                        ("header_colour".into(), Value::String("#dc8a78".into())),
                                        ("remove_empty_lines".into(), Value::Bool(false)),
                                        ("remove_first_lines".into(), Value::Bool(true)),
                                        ("clean_queries".into(), Value::Bool(true)),
                                        ("notify_about_updates".into(), Value::Bool(true)),
                                        ("log_played_songs".into(), Value::Bool(true)),
                                        ("metadata_colour".into(), Value::String("#209fb5".into())),
                                        (
                                                "renew_strings_interval".into(),
                                                Value::Int(-60 * 60 * 24 * 31 * 3), // About three months; negative to disable feature by default.
                                        ),
                                        ("last_string_renewal".into(), Value::Int(0)),
                                        ("first_launch".into(), Value::Bool(true)),
                                ]),
                        }
                }
        }

        impl fmt::Display for CoralConfig {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:?}", self)
                }
        }

        impl CoralConfig {
                /// Set a value in the configuration file and write it.
                pub fn set(&mut self, key: &str, value: Value) -> Result<(), ConfyError> {
                        for default_entry in CoralConfig::default().config {
                                self.config
                                        .entry(default_entry.0)
                                        .or_insert(default_entry.1);
                        }
                        if self.config.contains_key(key) {
                                self.config.insert(key.into(), value);
                        }
                        set_config(self.to_owned())
                }

                /// Get a value from the config file or the default
                pub fn get(&mut self, key: &str) -> Value {
                        if self.config.contains_key(key) {
                                self.config.get(key).unwrap().to_owned()
                        } else if CoralConfig::default().config.contains_key(key) {
                                if let Err(_) = self.set(
                                        key,
                                        CoralConfig::default().config.get(key).unwrap().clone(),
                                ) {};
                                CoralConfig::default().config.get(key).unwrap().to_owned()
                        } else {
                                Value::None
                        }
                }
        }
}

/// Accessing UG
pub mod network {
        use e_crate_version_checker::prelude::version::is_newer_version_available;
        use e_crate_version_checker::register_user_crate;
        use ug_scraper::search_scraper::get_search_results;
        use ug_scraper::tab_scraper::get_song_data;
        use ug_scraper::types::{SearchResult, Song};

        /// Get a tab
        ///
        /// Designed to be used in parallel with the main process.
        pub fn get_tab(url: &str) -> Result<Song, String> {
                match get_song_data(url, true) {
                        Ok(s) => Ok(s),
                        Err(e) => Err(e.to_string()),
                }
        }

        pub fn search(query: &str, depth: u8) -> Result<Vec<SearchResult>, String> {
                match get_search_results(query, depth) {
                        Ok(s) => Ok(s),
                        Err(e) => Err(e.to_string()),
                }
        }

        pub fn check_for_newer_version() -> bool {
                register_user_crate!();

                let crate_name = "Coral-Chords";

                is_newer_version_available(env!("CARGO_PKG_VERSION"), crate_name).unwrap_or(false)
        }
}
