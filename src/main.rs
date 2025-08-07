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
use ug_scraper::types::{Song};

use crate::backend::formats::{NotificationType, Value, CoralConfig};
use crate::backend::{network, system};
use crate::backend::system::{get_config, store_song};

/// Show info to the user
/// 
/// This is most likely not final.
/// ...Those will show popups or similar using GUI instead of println!()
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
pub fn get_tab(url: &str) {
        match network::get_tab(url) {
                Ok(s) => if let Err(e) = store_song(s, "rickroll") {
                        show_info(NotificationType::Error("Could not store song to local file: ".to_string() + &e.to_string()));        
                },
                Err(e) => show_info(NotificationType::Error("Something went wrong downloading the tab: ".to_string() + &e)),
        };
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

fn main() {
        let mut global_config: CoralConfig = get_config_object();

        println!("{global_config:?}");
        global_config.set("john", Value::String("Doe".into())).unwrap();
        println!("{global_config:?}");
        println!("{}", global_config.get("foo"));
        println!("{}", global_config.get("john"));
        global_config.set("foo", Value::Bool(!if let Value::Bool(b) = global_config.get("foo"){b}else{false})).unwrap();
        println!("{global_config:?}");

        get_tab("https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741");
        let loaded_rickroll = system::load_song("rickroll");
        println!("{loaded_rickroll:?}");
}