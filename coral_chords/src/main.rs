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

use coral_chords_backend::{get_song_data_from_url, store_song, CoralChordsData, CoralChordsError};
use std::{thread};

fn handle_coral_error(error: CoralChordsError) {
        match error {
                CoralChordsError::InvalidPageType => println!("Invalid page type."),
                CoralChordsError::ReqError(e) => println!("Web request returned error: {}", e),
                CoralChordsError::UnknownType => println!("Type not found.")
        }
}

fn handle_download_result(result: CoralChordsData) {
        match result {
                CoralChordsData::Error(e) => handle_coral_error(e),
                CoralChordsData::Data(d) => {
                        match store_song(d) {
                                Err(e) => handle_coral_error(e),
                                _ => (),
                        }
                },
        }
}

fn main() {
        let url_to_get = "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741"; 
        let get_chords_thread: thread::JoinHandle<CoralChordsData> =  thread::spawn(|| get_song_data_from_url(url_to_get));

        let downloaded_chords: CoralChordsData = get_chords_thread.join().unwrap();

        handle_download_result(downloaded_chords);
}