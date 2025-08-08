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

mod backend;

use confy::ConfyError;
use std::mem::replace;
use std::thread::{self};
use std::sync::mpsc::{self};
use ug_scraper::types::{Song};
use mpris::{Metadata, PlayerFinder};

use crate::backend::formats::{CoralConfig, GlobalReceivers, NotificationType, Value, TAB_DIR};
use crate::backend::{network, system};
use crate::backend::system::{get_config, load_song, store_song};

/// Show info to the user
/// 
/// This is most likely not final.
/// ...Those will show popups or similar using GUI instead of println!()
/// 
/// Might need to be changed to accept data from threads
pub fn show_info(content: NotificationType) {
        let message: String = match content {
                NotificationType::Info(c) => "Info: ".to_string() + &c,
                NotificationType::Warning(c) => "WARNING: ".to_string() + &c,
                NotificationType::Error(c) => "ERROR: ".to_string() + &c,
                NotificationType::Fatal(c) => "FATAL ERROR: ".to_string() + &c,
                NotificationType::Default => return,
        };
        println!("{message:?}");
}

/// Download and store a tab locally
pub fn spawn_get_tab_thread(url: String, song_uid: String, rec: &mut GlobalReceivers) {
        let (tx, rx) = mpsc::channel::<Value>();
        // A thread is spawned to prevent freezing UI
        thread::spawn(move || match network::get_tab(&url) {
                Ok(s) => if let Err(e) = store_song(s, song_uid.as_str()) {
                        if let Err(e) = tx.send(Value::Notification(
                                        NotificationType::Error("Could not store song to local file: ".to_string() + &e.to_string()))) {
                                println!("Could not send message: {}", e);
                        };
                },
                Err(e) => if let Err(e) = tx.send(Value::Notification(
                                NotificationType::Error("Something went wrong downloading the tab: ".to_string() + &e))) {
                        println!("Could not send message: {}", e);
                },
        });
        rec.0.push(rx);
}

/// Wrapper for getting a config object to use as global
fn get_config_object() -> CoralConfig {
        let config = get_config();
        if let Some(e) = config.1 {
                if let ConfyError::BadYamlData(_) = e {
                        show_info(NotificationType::Warning(
                                "Could not read config file; using defaults. Help: Restarting may solve the issue.".into()
                        ));
                } else {
                        show_info(NotificationType::Warning(
                                "An error occured while reading the configuration file; using defaults. - ".to_string() + &e.to_string()
                        ));
                }
        }
        config.0
}

/// Receiving the values sent by threads
/// 
/// Gets triggered by the [`crate::backend::formats::GlobalReceivers::update`] function.
pub fn handle_received_value(value: Value) {
        println!("VALUE RECEIVED {}", value);
}

/// Load song data or ask for download
pub fn get_song_data_by_uid(song_uid: &String, ask: bool) {
        match system::get_tab_path() {
                Ok(mut p) => {
                        p.pop();
                        p.push(TAB_DIR);
                        p.push(song_uid.clone() + ".yml");

                        if p.is_file() {
                                match load_song(&song_uid) {
                                        Ok(s) => song_to_ui(s),
                                        Err(e) => show_info(NotificationType::Error("Could not load song file: ".to_string() + &e.to_string())),
                                }
                        } else if ask {
                                todo!("Ask user to download a tab (show search results)")
                        }
                }
                Err(e) => show_info(NotificationType::Fatal("Could not get tab path: ".to_string() + &e.to_string())),
        }
}

/// Write a given Song's lines to UI 
pub fn song_to_ui(song: Song) {
        todo!("show the given tab to the user (project lines to UI)")
}

fn main() {
        let mut global_config: CoralConfig = get_config_object();
        let mut global_receivers: GlobalReceivers = GlobalReceivers(vec![]);

        // THIS MUST HAPPEN WHEN POPUPS ARE POSSIBLE.
        let player = match PlayerFinder::new() {
                Ok(pf) => match pf.find_active() {
                        Ok(p) => Some(p),
                        Err(e) => {
                                show_info(NotificationType::Fatal("Could not find any media player: ".to_string() + &e.to_string()));
                                None
                        },
                },
                Err(e) => {
                        show_info(NotificationType::Fatal("Could not connect to D-Bus: ".to_string() + &e.to_string()));
                        None
                },
        };

        let mut song_metadata: Option<Metadata>;
        if player.is_some() {
                let player = player.unwrap();

                // This should be saved at the end of every loop cycle.
                let mut song_id_previoes_cycle: String = String::new();
                // This will be used to check wether the display is showing the currently running song.
                // It will not be the case if no tob is locally stored; in this case, the song will be shown on UI as soon as it is downloaded.
                let mut song_id_display: String = String::new();

                // FOLLOWING WILL BE IN THE UPDATE LOOP BELOW!
                song_metadata = match player.get_metadata() {
                        Ok(md) => Some(md),
                        Err(e) => {
                                show_info(NotificationType::Error("Could not connect to D-Bus: ".to_string() + &e.to_string()));
                                None
                        }
                };
                if song_metadata.is_some() {
                        let song_uid: String = match song_metadata.unwrap().track_id() {
                                Some(id) => id.to_string().replace("/com/spotify/track/", "")
                                        .replace("/", "")
                                        .replace(" ", ""),
                                None => "unknown".into(),
                        };
                        if song_id_previoes_cycle != song_uid {
                                get_song_data_by_uid(&song_uid, true);
                        }
                        if song_id_display != song_uid {
                                get_song_data_by_uid(&song_uid, false);
                        }
                }
                
                loop {
                        global_receivers.update();
                }
        }


        println!("{global_config:?}");
        global_config.set("john", Value::String("Doe".into())).unwrap();
        println!("{global_config:?}");
        println!("{}", global_config.get("foo"));
        println!("{}", global_config.get("john"));
        global_config.set("foo", Value::Bool(!if let Value::Bool(b) = global_config.get("foo"){b}else{false})).unwrap();
        println!("{global_config:?}");        
}