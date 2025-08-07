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
        use std::{error::Error, path::PathBuf};
        use ug_scraper::types::*;
        use confy::{self, ConfyError};
        use crate::backend::formats::CoralConfig;

        /// Save a given song as a local file
        pub fn set_file(song: Song) -> Result<(), Box<dyn Error>> {
                todo!("store song")
        }
        /// Read a requested song file
        pub fn get_file(path: String) -> Result<Song, ConfyError> {
                todo!("Read files")
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
        pub fn set_config(config: CoralConfig) -> Result<(), Box<dyn Error>> {
                confy::store("Coral-Chords", Some("config"), config)?;
                println!("{:?}", confy::get_configuration_file_path("Coral-Chords", Some("config")).unwrap());
                Ok(())
        }

        pub fn set_config_property<T>(name: &str, value: T) {

        }

        /// Get the path to the locally stored tabs
        /// 
        /// Should use confy's config path + "/tabs" or something similar.
        pub fn get_tabs_path() -> PathBuf {
                todo!("get chord path")
        }
}

/// Formats used by Coral-Chords
pub mod formats {
        use std::hash::Hash;
        use std::{default, error::Error};
        use std::fmt;
        use ug_scraper::types::*;
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;

        /// Decode loaded CCh files
        /// 
        /// Might not be needed if using a new struct and confy for storing and loading tabs
        /// Otherwise, it should be a good idea to load and save using serde and serialization, rendering this function useless too.
        pub fn decode_file(raw_data: String) -> Result<Song, Box<dyn Error>> {
                todo!("Decode raw data")
        }

        /// Possible types of notifications which may be shown to the user
        #[derive(Debug, Clone, Default)]
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
        }

        impl fmt::Display for NotificationType {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", self)
                }
        }

        /// The configuration used by CCh
        /// 
        /// Uses HashMaps because old settings would be deleted when new ones are added to existing config files.
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct CoralConfig {
                str: HashMap<String, String>,
                bool: HashMap<String, bool>,
                int: HashMap<String, i64>,
                float: HashMap<String, f64>,
        }

        impl Default for CoralConfig {
                fn default() -> CoralConfig {
                        CoralConfig {
                                str: HashMap::from([
                                        ("foo".to_string(), "bar".to_string()),
                                        ("bar".to_string(), "foo".to_string()),
                                ]),
                                bool: HashMap::from([
                                        ("foo".to_string(), true),
                                        ("bar".to_string(), false),
                                ]),
                                int: HashMap::from([
                                        ("foo".to_string(), 64),
                                        ("bar".to_string(), i64::MAX),
                                ]),
                                float: HashMap::from([
                                        ("foo".to_string(), 0.0),
                                        ("bar".to_string(), f64::MAX),
                                ])
                        }
                }
        }

        impl fmt::Display for CoralConfig {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:?}", self)
                }
        }
}

/// Accessing UG
pub mod network {
        use ug_scraper::types::{Song};
        use std::error::Error;

        /// Get a tab
        /// 
        /// Designed to be used in parallel with the main process.
        pub fn get_tab(url: &str) -> Result<Song, Box<dyn Error>> {
                todo!("download tabs wrapper")
        }
}