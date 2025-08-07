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
                let result = confy::load::<CoralConfig>("Coral-Chords", None);
                match result {
                        Ok(c) => (c, None),
                        Err(e) => (CoralConfig::default(), Some(e)),
                }
        }

        /// Write the config file
        pub fn set_config(config: CoralConfig) -> Result<(), Box<dyn Error>> {
                confy::store("Coral-Chords", None, config)?;
                println!("{:?}", confy::get_configuration_file_path("Coral-Chords", None).unwrap());
                Ok(())
        }

        pub fn get_chords_path() -> PathBuf {
                todo!("get chord path")
        }
}

/// Formats used by Coral-Chords
pub mod formats {
        use std::{default, error::Error};
        use std::fmt;
        use ug_scraper::types::*;
        use serde::{Deserialize, Serialize};

        /// Decode loaded CCh files
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
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct CoralConfig {
                foo: bool,
        }

        impl Default for CoralConfig {
                fn default() -> CoralConfig {
                        CoralConfig { foo: true }
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