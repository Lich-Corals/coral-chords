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

use coral_chords_backend::data_preparation::{get_song_data_from_url};
use coral_chords_backend::types_and_constants::{CoralChordsError, SongData};
use coral_chords_backend::system_access::{store_song};
use std::{thread};

fn handle_coral_error(error: CoralChordsError) {
        match error {
                CoralChordsError::InvalidPageType => println!("Invalid page type."),
                CoralChordsError::ReqError(e) => println!("Web request returned error: {}", e),
                CoralChordsError::UnknownType => println!("Type not found."),
        }
}

fn handle_download_result(result: Result<SongData, CoralChordsError>) {
        match result {
                Err(e) => handle_coral_error(e),
                Ok(d) => {
                        match store_song(d) {
                                Err(e) => handle_coral_error(e),
                                _ => (),
                        }
                },
        }
}

fn main() {
        let url_to_get = "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741"; 
        let get_chords_thread: thread::JoinHandle<Result<SongData, CoralChordsError>> =  thread::spawn(|| get_song_data_from_url(url_to_get));

        let downloaded_chords  = get_chords_thread.join().unwrap();

        //handle_download_result(downloaded_chords);

        let valid_page_urls = vec!["https://tabs.ultimate-guitar.com/tab/queen/dont-stop-me-now-chords-519549",
                        "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741",
                        "https://tabs.ultimate-guitar.com/tab/led-zeppelin/stairway-to-heaven-tabs-9488",
                        "https://tabs.ultimate-guitar.com/tab/olli-schulz/wenn-es-gut-ist-ukulele-1381967",
                        "https://tabs.ultimate-guitar.com/tab/bloc-party/this-modern-love-bass-180218",
                        "https://tabs.ultimate-guitar.com/tab/phil-collins/in-the-air-tonight-drums-880599",
                        "https://tabs.ultimate-guitar.com/tab/blink-182/feeling-this-bass-104175",
                        "https://tabs.ultimate-guitar.com/tab/pink-floyd/empty-spaces-bass-147995"];
        for valid_page_url in valid_page_urls {
                println!("Testing {}", valid_page_url);
                get_song_data_from_url(valid_page_url);
        }
}