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
        use std::error::Error;
        use ug_scraper::types::*;

        /// Save a given song as a local file
        pub fn store_file(song: Song) -> Result<(), Box<dyn Error>> {
        todo!("store song")
}
        /// Read a requested song file
        pub fn read_file(path: String) -> Result<Song, Box<dyn Error>> {
                todo!("Read files")
        }
}

/// Formats used by Coral-Chords
pub mod formats {
        use std::error::Error;
        use ug_scraper::types::*;

        /// Decode loaded CCh files
        pub fn decode_file(raw_data: String) -> Result<Song, Box<dyn Error>> {
                todo!("Decode raw data")
        }
}

/// Accessing UG
pub mod network {
        use ug_scraper::types::{Song};
        use std::error::Error;

        /// Get a tab
        /// 
        /// Designed to be used from a thread
        pub fn get_tab(url: &str) -> Result<Song, Box<dyn Error>> {
                todo!("download tabs wrapper")
        }
}