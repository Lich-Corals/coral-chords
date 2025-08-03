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

use coral_chords_backend::{get_song_data_from_url, CoralChordsData, CoralError};
use std::thread;

fn handle_coral_error(error: CoralError) {
        match error {
                CoralError::InvalidPageType => println!("Invalid page type."),
                CoralError::ReqError(e) => println!("Web request returned error: {}", e),
        }
}

fn main() {
        let url_to_get = "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741"; 
        let get_chords_thread: thread::JoinHandle<CoralChordsData> =  thread::spawn(|| get_song_data_from_url(url_to_get));

        match get_chords_thread.join().unwrap() {
                CoralChordsData::Chords(d) => println!("Chords: {}", d),
                CoralChordsData::Tab(d) => println!("Tab: {}", d),
                CoralChordsData::Error(e) => handle_coral_error(e),
        }
}