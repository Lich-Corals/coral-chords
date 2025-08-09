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
use iced::Alignment::Center;
use iced::{Theme};
use std::thread::{self};
use std::sync::mpsc::{self};
use ug_scraper::types::{Song};
use mpris::{Metadata, PlayerFinder};

use crate::backend::formats::{CoralConfig, GlobalReceivers, NotificationType, Value, TAB_DIR, get_theme};
use crate::backend::{network, system};
use crate::backend::system::{get_config, load_song, store_song};

use iced::widget::{button, column, row, Column, Row, Space, toggler};

/// The application's properties
struct ApplicationState {
        screen: Screen,
        theme: Theme,
        /// The globally used config object
        config: CoralConfig,
        /// Receivers used to communicate with threads
        receivers: GlobalReceivers,
        /// Wether to check for a song-change.
        playing: bool,
        /// Notifications which will be sent to user using the notification bar
        /// 
        /// Those notifications shall be received from show_info() 
        notifications: Vec<NotificationType>,
        /// The song that was playing during the last cycle.
        /// If changed, it will trigger an update of the UI or a prompt to download a song.
        song_id_previoes_cycle: String,
        /// This will be used to check wether the display is showing the currently running song.
        /// It will not be the case if no tob is locally stored; in this case, the song will be shown on UI as soon as it is downloaded.
        song_id_display: String,

}

impl Default for ApplicationState {
        fn default() -> Self {
                ApplicationState { 
                        screen: Screen::default(), 
                        theme: get_theme(get_config_object()), 
                        config: get_config_object(),
                        receivers: GlobalReceivers(vec![]),
                        playing: false,
                        notifications: vec![],
                        song_id_previoes_cycle: String::new(),
                        song_id_display: String::new(),
                }
        }
}

/// Messages used to trigger actions from the UI
#[derive(Debug, Clone, Copy)]
enum Message {
        /// Changed to tabs page
        TabsPage,
        /// Changed to search page
        SearchPage,
        /// Changed to settings page
        SettingsPage,
        /// Toggled playing toggle
        PlayingToggled(bool),
}

impl ApplicationState {
        pub fn theme(&self) -> Theme {
                self.theme.clone()
        }

        /// Function to get the current view
        pub fn view(&self) -> Column<Message> {
                let button_padding: [u16; 2] = [4, 12];
                // The header bar containing basic controls
                let controls: Row<Message> = match self.screen {
                        Screen::Tabs => { row![
                                        button("Tab")
                                                .on_press(Message::TabsPage)
                                                .padding(button_padding),
                                        button("Search")
                                                .on_press(Message::SearchPage)
                                                .style(button::secondary)
                                                .padding(button_padding),
                                        button("Settings")
                                                .on_press(Message::SettingsPage)
                                                .style(button::secondary)
                                                .padding(button_padding),
                                        Space::new(100, 1),
                                        toggler(self.playing)
                                                .label("Play")
                                                .on_toggle(Message::PlayingToggled)
                                ].align_y(Center)
                        },
                        Screen::Search => { row![
                                        button("Tab")
                                                .on_press(Message::TabsPage)
                                                .style(button::secondary)
                                                .padding(button_padding),
                                        button("Search")
                                                .on_press(Message::SearchPage)
                                                .padding(button_padding),
                                        button("Settings")
                                                .on_press(Message::SettingsPage)
                                                .style(button::secondary)
                                                .padding(button_padding),
                                ]
                        },
                        Screen::Settings => { row![
                                        button("Tab")
                                                .on_press(Message::TabsPage)
                                                .style(button::secondary)
                                                .padding(button_padding),
                                        button("Search")
                                                .on_press(Message::SearchPage)
                                                .style(button::secondary)
                                                .padding(button_padding),
                                        button("Settings")
                                                .on_press(Message::SettingsPage)
                                                .padding(button_padding),
                                ]
                        },
                };

                // The main contents of the window
                let contents = match self.screen {
                        Screen::Tabs => {
                                Column::new()
                        }, 
                        Screen::Search => {
                                Column::new()
                        },
                        Screen::Settings => {
                                column![

                                ]
                        }
                };

                // A bar sitting at the bottom of the window to show messages to the user
                let message_bar: Row<Message> = row![

                ];

                column![controls, contents, message_bar]

        }

        pub fn update(&mut self, message: Message) {
                match message {
                        Message::TabsPage => self.screen = Screen::Tabs,
                        Message::SearchPage => self.screen = Screen::Search,
                        Message::SettingsPage => self.screen = Screen::Settings,

                        Message::PlayingToggled(s) => self.playing = s,
                        _ => (),
                }

                match self.screen {
                        Screen::Settings => {
                                self.theme = get_theme(self.config.clone());
                        },
                        _ => (),
                }

                self.receivers.update();
        }
}

#[derive(Default)]
enum Screen {
        #[default]
        Tabs,
        Search,
        Settings,
}

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

        let _ = iced::application("Coral-Chords", ApplicationState::update, ApplicationState::view)
                .centered()
                .theme(ApplicationState::theme)
                .run();

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
        }
}