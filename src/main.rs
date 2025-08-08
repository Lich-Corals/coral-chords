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
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use crate::backend::formats::{CoralConfig, GlobalReceivers, NotificationType, Value};
use crate::backend::{network, system};
use crate::backend::system::{get_config, store_song};

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
                NotificationType::Default => return,
        };
        println!("{message:?}");
}

/// Download and store a tab locally
pub fn spawn_get_tab_thread(url: String, rec: &mut GlobalReceivers) {
        let (tx, rx) = mpsc::channel::<Value>();
        // A thread is spawned to prevent freezing UI
        thread::spawn(move || match network::get_tab(&url) {
                Ok(s) => if let Err(e) = store_song(s, "rickroll") {
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

/// Wrapper for getting a config object
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

fn main() {
        let mut global_config: CoralConfig = get_config_object();
        let mut global_receivers: GlobalReceivers = GlobalReceivers(vec![]);

        println!("{global_config:?}");
        //global_config.set("john", Value::String("Doe".into())).unwrap();
        println!("{global_config:?}");
        println!("{}", global_config.get("foo"));
        println!("{}", global_config.get("john"));
        //global_config.set("foo", Value::Bool(!if let Value::Bool(b) = global_config.get("foo"){b}else{false})).unwrap();
        println!("{global_config:?}");

        
        spawn_get_tab_thread("https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741".into(), &mut global_receivers);
        
        loop {
                global_receivers.update();
        }

        let loaded_rickroll = system::load_song("rickroll");
        println!("{loaded_rickroll:?}");
}