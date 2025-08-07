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
use backend::system::{set_config, get_config, set_file};
use backend::network::{};
use backend::formats::CoralConfig;
use confy::ConfyError;
use ug_scraper::types::{Song};
use std::thread::sleep;
use std::time::Duration;
use std::{error::Error, thread};

use crate::backend::formats::{NotificationType, Value};

/// Handle errors
/// 
/// Notify the user or do stuff to solve the problem
fn handle_error(error: Box<dyn Error>) {
        match error {
                _ => (),
        }
}

/// Handle download result.
/// 
/// Store if successfull, else notify user
fn handle_download_result(result: Result<Song, Box<dyn Error>>) {
        match result {
                Err(e) => handle_error(e),
                Ok(d) => {
                        if let Err(e) = set_file(d) {
                                handle_error(e);
                        }
                },
        }
}

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

fn main() {
        let global_config = get_config();
        if let Some(e) = global_config.1 {
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
        let mut global_config = global_config.0;
        println!("{global_config:?}");
        global_config.set("john", Value::String("Doe".into()));
        println!("{global_config:?}");
        println!("{}", global_config.get("foo"));
        println!("{}", global_config.get("john"));
        println!("{global_config:?}");       
}