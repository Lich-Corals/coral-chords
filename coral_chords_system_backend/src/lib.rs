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

use crate::types_and_constants::{SongData, CoralChordsError};

pub fn store_song(song_data: SongData) -> Result<bool, CoralChordsError> {
        todo!("store song")
}

pub fn read_file(path: String) -> Result<SongData, CoralChordsError> {
        todo!("Read files")
}

pub fn decode_file(raw_data: String) -> Result<SongData, CoralChordsError> {
        todo!("Decode raw data")
}
