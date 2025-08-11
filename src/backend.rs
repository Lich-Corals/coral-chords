// Coral-Chords - Automatic chords for spotify
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
        use std::{path::PathBuf};
        use ug_scraper::types::*;
        use confy::{self, ConfyError, get_configuration_file_path};
        use crate::backend::formats::{CoralConfig, TAB_DIR};

        /// Save a given song as a local file
        pub fn store_song(song: Song, song_uid: &str) -> Result<(), ConfyError> {
                confy::store("Coral-Chords", Some((TAB_DIR.to_string() + "/" + song_uid).as_str()), song)?;
                Ok(())
        }

        /// Read a requested song file
        pub fn load_song(song_uid: &str) -> Result<Song, ConfyError> {
                confy::load::<Song>("Coral-Chords", Some((TAB_DIR.to_string() + "/" + song_uid).as_str()))          
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
        use std::fmt;
        use confy::ConfyError;
        use ug_scraper::types::*;
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;
        use crate::backend::system::{set_config};

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
        }

        impl fmt::Display for NotificationType {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self)
                }
        }

        /// A wrapper to store different kinds of data in a single HashMap
        /// 
        /// Used to store values in the configuration file and to send data between threads
        #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
        pub enum Value {
                #[default]
                None,
                String(String),
                Bool(bool),
                Int(i64),
                Float(f64),
                Notification(NotificationType),
                SearchResults(Vec<SearchResult>),
                DataSetTypeOption(Option<DataSetType>),
        }

        impl fmt::Display for Value {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:?}", self)
                }
        }

        /// The uration used by CCh
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
                                        ("theme".into(), Value::String("Theme::CatppuccinMocha".into())),
                                        ("search_depth".into(), Value::Int(2)),
                                        ("search_filter".into(), Value::DataSetTypeOption(None)),
                                        ("only_downloadable_results".into(), Value::Bool(false)),
                                        ("tab_text_size".into(), Value::Int(15)),
                                        ("chord_colour".into(), Value::String("#fe640b".into())),
                                        ("header_colour".into(), Value::String("#dc8a78".into())),
                                        ("remove_empty_lines".into(), Value::Bool(false)),
                                        ("remove_first_lines".into(), Value::Bool(true)),
                                ])
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
                                self.config.entry(default_entry.0).or_insert(default_entry.1);
                        }
                        if self.config.contains_key(key) {
                                self.config.insert(key.into(), value);
                        }
                        set_config(self.to_owned())
                }

                /// Get a value from the config file or the default
                pub fn get(&mut self, key: &str) -> Value {
                        if self.config.contains_key(key) {
                                return self.config.get(key).unwrap().to_owned()
                        } else if CoralConfig::default().config.contains_key(key) {
                                if let Err(_) = self.set(key, CoralConfig::default().config.get(key).unwrap().clone()){};
                                return CoralConfig::default().config.get(key).unwrap().to_owned()
                        } else {
                                return Value::None
                        }
                }
        }
}

/// Accessing UG
pub mod network {
        use ug_scraper::types::{SearchResult, Song};
        use ug_scraper::tab_scraper::get_song_data;
        use ug_scraper::search_scraper::get_search_results;

        /// Get a tab
        /// 
        /// Designed to be used in parallel with the main process.
        pub fn get_tab(url: &str) -> Result<Song, String> {
                match get_song_data(url, true) {
                        Ok(s) => Ok(s),
                        Err(e) => Err(e.to_string()),
                }
        }

        pub fn search(query: &str, depth: u8) -> Result<Vec<SearchResult>, String>{
                match get_search_results(query, depth) {
                        Ok(s) => Ok(s),
                        Err(e) => Err(e.to_string()),
                }
        }
}