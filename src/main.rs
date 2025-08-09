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
use ug_scraper::types::{SearchResult, Song};
use mpris::{Metadata, PlayerFinder, Player};

use crate::backend::formats::{CoralConfig, NotificationType, Value, TAB_DIR, get_theme};
use crate::backend::{network, system};
use crate::backend::system::{get_config, load_song, store_song};

use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler, Column, Row, Space};

/// The application's properties
struct ApplicationState {
        screen: Screen,
        theme: Theme,
        /// The globally used config object
        config: CoralConfig,
        /// The channel to communicate with other threads
        channel: (std::sync::mpsc::Sender<Value>, std::sync::mpsc::Receiver<Value>),
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
        /// It will not be the case if no tab is locally stored; in this case, the song will be shown on UI as soon as it is downloaded.
        song_id_display: String,
        /// The mpris player object used to communicate with audio players
        player: Option<Player>,
        /// The current value in the search bar
        search_value: String,
        /// The placeholder for the search bar
        search_placeholder: String,
        /// The current search status
        search_state: SearchState,   
        /// The UID of the currently running song
        current_song_uid: String,     
}

impl Default for ApplicationState {
        fn default() -> Self {
                let mut config_result = get_config_object();
                let mut notifications: Vec<NotificationType> = vec![];

                if let Some(n) = config_result.1 {
                        notifications.push(n);
                }

                let player = match PlayerFinder::new() {
                        Ok(pf) => match pf.find_active() {
                                Ok(p) => Some(p),
                                Err(e) => {
                                        notifications.push(NotificationType::Fatal("Could not find any media player: ".to_string() + &e.to_string()));
                                        None
                                },
                        },
                        Err(e) => {
                                notifications.push(NotificationType::Fatal("Could not connect to D-Bus: ".to_string() + &e.to_string()));
                                None
                        },
                };

                ApplicationState { 
                        screen: Screen::default(), 
                        theme: get_theme(&mut config_result.0), 
                        config: config_result.0,
                        channel: mpsc::channel::<Value>(),
                        playing: false,
                        notifications: notifications,
                        song_id_previoes_cycle: String::new(),
                        song_id_display: String::new(),
                        player: player,
                        search_value: String::new(),
                        search_placeholder: String::new(),
                        search_state: SearchState::default(),
                        current_song_uid: String::new(),
                }
        }
}

/// Messages used to trigger actions from the UI
#[derive(Debug, Clone)]
enum Message {
        /// Changed to tabs page
        TabsPage,
        /// Changed to search page
        SearchPage,
        /// Changed to settings page
        SettingsPage,
        /// Toggled playing toggle
        PlayingToggled(bool),
        /// Update the value of the search bar
        UpdateSearchBar(String),
        /// Launch a search for the current value
        SearchTabs,
        /// Clear the search bar
        ClearSearch,
        /// Download a tab
        DownloadTab(String),
}

#[derive(Default)]
enum Screen {
        #[default]
        Tabs,
        Search,
        Settings,
}

#[derive(Debug, Default)]
enum SearchState {
        #[default]
        None,
        Searching,
        Finished(Vec<SearchResult>),
}

impl ApplicationState {
        pub fn theme(&self) -> Theme {
                self.theme.clone()
        }

        /// Function to get the current view
        pub fn view(&self) -> Column<'_, Message> {
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
                        Screen::Search => {
                                let value = &self.search_value;
                                let placeholder = &self.search_placeholder;
                                row![
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
                                        Space::new(100, 1),
                                        text_input(placeholder, value)
                                                .on_input(Message::UpdateSearchBar)
                                                .width(300),
                                        button("Go!")
                                                .on_press(Message::SearchTabs)
                                                .padding(button_padding),
                                        Space::new(10, 1),
                                        button("Clear")
                                                .on_press(Message::ClearSearch)
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
                                let mut column = column![];
                                if let SearchState::Finished(s) = &self.search_state {
                                        for search_result in s {
                                                let title = search_result.basic_data.title.as_str();
                                                let url = search_result.basic_data.tab_link.clone();
                                                column = column.push(row![
                                                        button("Download")
                                                                .on_press(Message::DownloadTab(url)),
                                                        Space::new(10, 1),
                                                        text(title),
                                                        Space::new(30, 10),
                                                ]);
                                        }
                                }
                                column![container(
                                        scrollable(column))
                                        .padding(10)]
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
                println!("{:?}", self.search_state);

                // First parts are updating the UI

                // Execute commands associated to messages
                match message {
                        Message::TabsPage => self.screen = Screen::Tabs,
                        Message::SearchPage => self.screen = Screen::Search,
                        Message::SettingsPage => self.screen = Screen::Settings,

                        Message::PlayingToggled(s) => self.playing = s,
                        Message::UpdateSearchBar(s) => self.search_value = s,
                        Message::SearchTabs => self.search_tabs(),
                        Message::ClearSearch => self.search_value = "".into(),
                        Message::DownloadTab(url) => self.spawn_get_tab_thread(url, self.current_song_uid.to_owned()),
                }

                // Update dependent of UI location
                match self.screen {
                        Screen::Settings => {
                                self.theme = get_theme(&mut self.config);
                        },
                        Screen::Search => {
                                self.search_placeholder = match self.search_state {
                                        SearchState::Searching => "Searching...".into(),
                                        _ => "Search a tab...".into(),
                                }
                        }
                        _ => (),
                }

                // Following is updating the program's main logic
                let song_metadata: Option<Metadata>;
                if self.player.is_some() {
                        let player = self.player.as_ref().unwrap();

                        song_metadata = match player.get_metadata() {
                                Ok(md) => Some(md),
                                Err(e) => {
                                        self.show_info(NotificationType::Error("Could not connect to D-Bus: ".to_string() + &e.to_string()));
                                        None
                                }
                        };
                        if song_metadata.is_some() {
                                let song_uid: String = match song_metadata.unwrap().track_id() {
                                        Some(id) => {
                                                if id.to_string().contains("spotify") {
                                                        id.to_string().replace("/com/spotify/track/", "")
                                                        .replace("/", "")
                                                        .replace(" ", "")
                                                } else {
                                                        "unknown".into()
                                                }   
                                        }
                                        None => "unknown".into(),
                                };
                                if self.playing {
                                        if self.song_id_previoes_cycle != song_uid {
                                                self.get_song_data_by_uid(&song_uid, true);
                                        }
                                        if self.song_id_display != song_uid {
                                                self.get_song_data_by_uid(&song_uid, false);
                                        }
                                        self.song_id_previoes_cycle = song_uid.clone();
                                }
                                self.current_song_uid = song_uid.into();
                        }
                }

                // Check if any thread returned a value
                if let Ok(v) = self.channel.1.try_recv() {
                        match v {
                                Value::Notification(n) => self.notifications.push(n),
                                Value::SearchResults(s) => self.search_to_ui(s),
                                _ => println!("Received: {}", v),
                        }
                } 
        }

        /// Show info to the user
        /// 
        /// This is most likely not final.
        /// ...Those will show popups or similar using GUI instead of println!()
        /// 
        /// Might need to be changed to accept data from threads
        pub fn show_info(&mut self, content: NotificationType) {
                self.notifications.push(content);
        }

        /// Tell the update function to show search results
        pub fn search_to_ui(&mut self, search_results: Vec<SearchResult>) {
                self.search_state = SearchState::Finished(search_results)
        }

        pub fn search_tabs(&mut self) {
                let tx = self.channel.0.clone();
                self.search_state = SearchState::Searching;
                let search_query = self.search_value.clone();
                let search_depth: u8 = match self.config.get("search_depth".into()) {
                        Value::Int(d) => d as u8,
                        _ => 2,
                };
                thread::spawn(move || match network::search(search_query.as_str(), search_depth) {
                        Ok(s) => {
                                if let Err(e) = tx.send(Value::SearchResults(s)) {
                                        if let Err(e) = tx.send(Value::Notification(
                                                        NotificationType::Error("Could not send search results to main thread: ".to_string() + &e.to_string()))) {
                                                println!("Could not send message: {}", e);
                                        }
                                }
                        },
                        Err(e) => if let Err(e) = tx.send(Value::Notification(
                                        NotificationType::Error("Something went wrong getting the search results: ".to_string() + &e))) {
                                println!("Could not send message: {}", e);
                        },
                });
        }

        /// Download and store a tab locally
        pub fn spawn_get_tab_thread(&mut self, url: String, song_uid: String) {
                let tx = self.channel.0.clone();
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
        }

        /// Write a given Song's lines to UI 
        pub fn song_to_ui(&mut self, song: Song, song_uid: String) {
                self.song_id_display = song_uid;
                todo!("show the given tab to the user (project lines to UI)")
        }

        /// Load song data or ask for download
        pub fn get_song_data_by_uid(&mut self, song_uid: &String, ask: bool) {
                match system::get_tab_path() {
                        Ok(mut p) => {
                                p.pop();
                                p.push(TAB_DIR);
                                p.push(song_uid.clone() + ".yml");

                                if p.is_file() {
                                        match load_song(&song_uid) {
                                                Ok(s) => self.song_to_ui(s, song_uid.to_owned()),
                                                Err(e) => self.show_info(NotificationType::Error("Could not load song file: ".to_string() + &e.to_string())),
                                        }
                                } else if ask {
                                        self.screen = Screen::Search;
                                }
                        }
                        Err(e) => self.show_info(NotificationType::Fatal("Could not get tab path: ".to_string() + &e.to_string())),
                }
        }
}

/// Wrapper for getting a config object to use as global
fn get_config_object() -> (CoralConfig, Option<NotificationType>) {
        let config = get_config();
        let mut message = None;
        if let Some(e) = config.1 {
                if let ConfyError::BadYamlData(_) = e {
                        message = Some(NotificationType::Warning(
                                "Could not read config file; using defaults. Help: Restarting may solve the issue.".into()));
                } else {
                        message = Some(NotificationType::Warning(
                                "An error occured while reading the configuration file; using defaults. - ".to_string() + &e.to_string()));
                }
        }
        (config.0, message)
}

fn main() {
        let _ = iced::application("Coral-Chords", ApplicationState::update, ApplicationState::view)
                .centered()
                .theme(ApplicationState::theme)
                .run();
}