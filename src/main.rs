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
use iced::{event, font, Color, Event, Font, Size, Subscription, Theme};
use iced::time::{self, Duration};
use iced::{color, window};
use iced::widget::{button, checkbox, column, combo_box, container, rich_text, row, scrollable, slider, span, text, text_input, toggler, Column, Row, Space};
use std::process::exit;
use std::thread::{self};
use std::sync::mpsc::{self};
use std::vec;
use ug_scraper::types::{DataSetType, DataType, SearchResult, Song, SUPPORTED_DOWNLOAD_TYPES};
use mpris::{Metadata, PlayerFinder, Player};

use crate::backend::formats::{CoralConfig, NotificationType, Value, TAB_DIR};
use crate::backend::{network, system};
use crate::backend::system::{get_config, load_song, store_song};

/// The application's properties
struct ApplicationState {
        screen: Screen,
        theme: Theme,
        /// The theme selected from the dropdown
        selected_theme: Option<Theme>,
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
        /// The current search status
        search_state: SearchState,   
        /// The search depth to use
        search_depth: u8,
        /// The UID of the currently running song
        current_song_uid: String,
        /// Selectable themes
        themes: combo_box::State<Theme>,
        /// Selectable tab filters
        tab_types: combo_box::State<DataSetType>,
        /// The currently selected search filter
        search_filter: Option<DataSetType>,
        /// The current window size
        size: Size,
        /// The height of the head bar
        bar_height: f32,
        /// Wether un-downloadable search results are disabled
        only_downloadable_results: bool,
        /// The currently displayed tab
        current_tab: Song,
        /// The configured size of a line of a tab
        tab_text_size: u8,
        /// The configured chord colour
        chord_colour: Color,
        // The value inside the chord colour input box
        chord_colour_text: String,
        // The padding of the main window contents
        main_padding: f32,
        // Wether the lines before the first chords will be removed
        remove_first_lines: bool,
        // Wether emyty lines will be removed from tabs
        remove_empty_lines: bool,
        // Path to the current tab file
        current_path: String,
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

                notifications.push(NotificationType::Info("INFO".into()));
                notifications.push(NotificationType::Warning("WARNING".into()));
                notifications.push(NotificationType::Error("ERROR".into()));
                notifications.push(NotificationType::Fatal("FATAL".into()));

                let search_filter = if let Value::DataSetTypeOption(o) = config_result.0.get("search_filter"){o}else{None};
                let search_depth = if let Value::Int(v) = config_result.0.get("search_depth"){v}else{2};
                let only_downloadable_results = if let Value::Bool(v) = config_result.0.get("only_downloadable_results"){v}else{false};
                let remove_first_lines = if let Value::Bool(v) = config_result.0.get("remove_first_lines"){v}else{false};
                let remove_empty_lines = if let Value::Bool(v) = config_result.0.get("remove_empty_lines"){v}else{false};
                let tab_text_size = if let Value::Int(v) = config_result.0.get("tab_text_size"){v}else{15};
                let loaded_colour = if let Value::String(v) = config_result.0.get("chord_colour"){v}else{"#fe640b".into()};
                let chord_colour = Color::parse(&loaded_colour).unwrap_or(Color::parse("fe640b").unwrap());

                ApplicationState { 
                        screen: Screen::default(), 
                        theme: get_selected_theme(&mut config_result.0), 
                        selected_theme: Some(get_selected_theme(&mut config_result.0)),
                        search_depth: search_depth as u8,
                        config: config_result.0,
                        channel: mpsc::channel::<Value>(),
                        playing: false,
                        notifications: notifications,
                        song_id_previoes_cycle: String::new(),
                        song_id_display: String::new(),
                        player: player,
                        search_value: String::new(),
                        search_state: SearchState::default(),
                        current_song_uid: String::new(),
                        themes: combo_box::State::new(Theme::ALL.to_vec()),
                        tab_types: combo_box::State::new(SUPPORTED_DOWNLOAD_TYPES.to_vec()),
                        search_filter: search_filter,
                        size: Size::default(),
                        bar_height: 0.0,
                        only_downloadable_results: only_downloadable_results,
                        current_tab: Song::default(),
                        tab_text_size: tab_text_size as u8,
                        chord_colour: chord_colour,
                        chord_colour_text: loaded_colour,
                        main_padding: 10.0,
                        remove_empty_lines: remove_empty_lines,
                        remove_first_lines: remove_first_lines,
                        current_path: String::new(),
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
        /// Update the application
        Update,
        /// Apply a search filter
        ApplySearch(DataSetType),
        /// Reset the search filter
        ResetFilter,
        /// Apply a theme
        ApplyTheme(Theme),
        /// Apply search depth
        ApplySearchDepth(u8),
        /// An iced event occured
        EventOccurred(Event),
        /// en-/disable un-downloadable results
        SetDownloadableOnly(bool),
        /// Set the height of tab lines
        SetTabLineHeight(u8),
        /// Chord colour input
        ChordColourChange(String),
        /// Remove-first-lines toggled
        RemoveFirstLines(bool),
        /// Remove-empty-lines toggled
        RemoveEmptyLines(bool),
        /// Open the current tab file
        OpenTabFile,
        /// Clear all notifications
        ClearNotifications,
        /// Close the application
        CloseApp,
}

#[derive(Default, PartialEq)]
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

                // The main contents of the window
                let contents: Column<'_, Message> = match self.screen {
                        Screen::Tabs => {
                                if self.current_tab.lines.len() != 0 {
                                        let mut main_row: Row<'_, Message> = row![];
                                        let mut new_column = column![];
                                        if let Some(d) = &self.current_tab.metadata {
                                                if let Some(d) = &d.capo {
                                                        new_column = new_column.push(row![
                                                                text(format!("Capo: {}", d.clone()))
                                                                        .font(Font::MONOSPACE)
                                                                        .size(self.tab_text_size as f32)
                                                        ].height(self.tab_text_size as f32));
                                                }
                                                if let Some(d) = &d.tuning {
                                                        new_column = new_column.push(row![
                                                                text(format!("Tuning: {}", d.clone()))
                                                                        .font(Font::MONOSPACE)
                                                                        .size(self.tab_text_size as f32)
                                                        ].height(self.tab_text_size as f32));
                                                }
                                                new_column = new_column.push(row![
                                                        text(" ")
                                                                .font(Font::MONOSPACE)
                                                                .size(self.tab_text_size as f32)
                                                ].height(self.tab_text_size as f32));
                                        }
                                                
                                        let max_lines_per_column = 0.9 * ((self.size.height - self.bar_height - 2.0 * self.main_padding) / self.tab_text_size as f32);
                                        let mut lines_on_column = 3;
                                        let mut first_chords_found = false;
                                        for line in &self.current_tab.lines {
                                                if line.line_type != DataType::Lyric {
                                                        first_chords_found = true;
                                                }
                                                if !self.remove_first_lines || first_chords_found {
                                                        lines_on_column += 1;
                                                        match line.line_type {
                                                                DataType::Chord => {
                                                                        new_column = new_column.push(row![
                                                                                text(line.text_data.clone())
                                                                                        .color(self.chord_colour)
                                                                                        .font(Font::MONOSPACE)
                                                                                        .size(self.tab_text_size as f32)
                                                                        ].height(self.tab_text_size as f32));
                                                                },
                                                                DataType::Lyric => {
                                                                        if !self.remove_empty_lines || line.text_data.len() > 0 {
                                                                                new_column = new_column.push(row![
                                                                                        text(line.text_data.clone())
                                                                                                .font(Font::MONOSPACE)
                                                                                                .size(self.tab_text_size as f32)
                                                                                ].height(self.tab_text_size as f32));
                                                                        }
                                                                }, 
                                                                DataType::SectionTitle => {
                                                                        new_column = new_column.push(row![
                                                                                rich_text([span(
                                                                                        line.text_data.clone())
                                                                                                .font(Font {
                                                                                                        style: font::Style::Italic,
                                                                                                        ..Font::MONOSPACE })
                                                                                                .size(self.tab_text_size as f32),
                                                                                ])
                                                                        ].height(self.tab_text_size as f32));
                                                                }
                                                        }
                                                        if !(lines_on_column < max_lines_per_column as u16) 
                                                                && line.line_type == DataType::Lyric {
                                                                lines_on_column = 0;
                                                                main_row = main_row.push(new_column);
                                                                new_column = column![];
                                                        }
                                                }
                                        }
                                        main_row = main_row.push(new_column);
                                        column![main_row
                                                .spacing(20)]
                                } else {
                                        Column::new()
                                }.padding(self.main_padding)
                        }, 
                        Screen::Search => {
                                match &self.search_state {
                                        SearchState::Finished(s) => {
                                                let mut title_column = column![];
                                                let mut artist_column = column![];
                                                let mut rating_column = column![];
                                                let mut rating_count_column = column![];
                                                let mut type_column = column![];

                                                let mut sorted_results = s.clone();
                                                let mut results: Vec<Vec<SearchResult>> = vec![vec![]];
                                                let mut previous_artist: String = String::new();
                                                let mut artist_vec: Vec<SearchResult> = vec![];
                                                if self.search_filter.is_some() {
                                                        let mut new_results = vec![];
                                                        for result in sorted_results {
                                                                if result.basic_data.data_type == self.search_filter.unwrap() {
                                                                        new_results.push(result.clone());
                                                                }
                                                        }
                                                        sorted_results = new_results;
                                                }
                                                if sorted_results.len() == 0 {
                                                        column![text("No search results!").color(color!(0xe64553))].padding(10)
                                                } else {
                                                        for result in sorted_results {
                                                                if result.basic_data.artist == previous_artist {
                                                                        previous_artist = result.basic_data.artist.clone();
                                                                        artist_vec.push(result);
                                                                } else {
                                                                        results.push(artist_vec.clone());
                                                                        artist_vec = vec![];
                                                                        previous_artist = result.basic_data.artist.clone();
                                                                        artist_vec.push(result);
                                                                }
                                                        }
                                                        results.push(artist_vec.clone());
                                                        let mut sorted_results: Vec<SearchResult> = vec![];
                                                        for mut artist_vec in results {
                                                                artist_vec.sort_by_key(|s| (s.rating_value * -100.0) as i32);
                                                                sorted_results.append(&mut artist_vec);
                                                        }
                                                        for search_result in sorted_results {
                                                                let title = search_result.basic_data.title.clone();
                                                                let artist = search_result.basic_data.artist.clone();
                                                                let url = search_result.basic_data.tab_link.clone();
                                                                let rating_count = search_result.rating_count;
                                                                let rating: String;
                                                                let rating_count_string: String;
                                                                if rating_count > 0 {
                                                                        rating = format!("{:.1$}/5", search_result.rating_value, 1);
                                                                        rating_count_string = format!("x{rating_count:?}");
                                                                } else {
                                                                        rating = "?".into();
                                                                        rating_count_string = "".into();
                                                                }
                                                                let row_height = 40;
                                                                if SUPPORTED_DOWNLOAD_TYPES.contains(&search_result.basic_data.data_type) || !self.only_downloadable_results {
                                                                        if SUPPORTED_DOWNLOAD_TYPES.contains(&search_result.basic_data.data_type) {
                                                                                title_column = title_column.push(row![
                                                                                        button("Download")
                                                                                                .on_press(Message::DownloadTab(url)),
                                                                                        Space::new(10, 0),
                                                                                        text(title),
                                                                                        Space::new(30, 0),
                                                                                ].height(row_height).align_y(Center));
                                                                        } else {
                                                                                title_column = title_column.push(row![
                                                                                        button("Download")
                                                                                                .on_press(Message::Update)
                                                                                                .style(button::secondary),
                                                                                        Space::new(10, 0),
                                                                                        text(title),
                                                                                        Space::new(30, 0),
                                                                                ].height(row_height).align_y(Center));
                                                                        }
                                                                        artist_column = artist_column.push(row![
                                                                                text("by "),
                                                                                text(artist).color(color!(0xea76cb)),
                                                                                Space::new(30, 0),
                                                                        ].height(row_height).align_y(Center));
                                                                        rating_column = rating_column.push(row![
                                                                                text("Rating: "),
                                                                                text(rating).color(color!(0xdd7878)),
                                                                                Space::new(10, 0),
                                                                        ].height(row_height).align_y(Center));
                                                                        rating_count_column = rating_count_column.push(row![
                                                                                text(rating_count_string).color(color!(0xdd7878)),
                                                                                Space::new(30, 0),
                                                                        ].height(row_height).align_y(Center));
                                                                        type_column = type_column.push(row![
                                                                        text(format!("{}", &search_result.basic_data.data_type))
                                                                        .color(color!(0x179299)),
                                                                        Space::new(30, 0),
                                                                        ].height(row_height).align_y(Center));
                                                                }
                                                                
                                                        }
                                                        column![container(
                                                                scrollable(row![
                                                                        title_column, artist_column, rating_column, rating_count_column, type_column,
                                                                ])
                                                                .spacing(10))
                                                                .padding(10)]
                                                        }
                                                },
                                        SearchState::Searching => column![text("Searching...")].padding(self.main_padding),
                                        _ => Column::new()
                                }.align_x(Center)
                                
                        },
                        Screen::Settings => {
                                column![
                                        column![
                                                text("Application")
                                                        .size(25),
                                                row![
                                                        text("Theme"),
                                                        Space::new(10, 0),
                                                        combo_box(&self.themes, "Theme", self.selected_theme.as_ref(), Message::ApplyTheme)
                                                                .width(300),
                                                ].align_y(Center),
                                        ].spacing(20),
                                        column![
                                                text("Search")
                                                        .size(25),
                                                row![
                                                        checkbox("Only show downloadable search results", self.only_downloadable_results)
                                                                .on_toggle(Message::SetDownloadableOnly),
                                                ].align_y(Center),
                                                row![
                                                        text(format!("Search depth: {:02}", self.search_depth)),
                                                        Space::new(10, 0),
                                                        slider(2..=64, self.search_depth, Message::ApplySearchDepth)
                                                                .width(300),
                                                ].align_y(Center),
                                        ].spacing(20),
                                        column![
                                                text("Tabs")
                                                        .size(25),
                                                row![
                                                        checkbox("Remove lines before first chords", self.remove_first_lines)
                                                                .on_toggle(Message::RemoveFirstLines),
                                                ].align_y(Center),
                                                row![
                                                        checkbox("Remove empty lines", self.remove_empty_lines)
                                                                .on_toggle(Message::RemoveEmptyLines),
                                                ].align_y(Center),
                                                row![
                                                        text(format!("Text size: {:02}", self.tab_text_size)),
                                                        Space::new(10, 0),
                                                        slider(5..=99, self.tab_text_size, Message::SetTabLineHeight)
                                                                .width(300),
                                                ].align_y(Center),
                                                row![
                                                        text("Chord colour:"),
                                                        Space::new(10, 0),
                                                        text_input("#fe640b", &self.chord_colour_text)
                                                                .on_input(Message::ChordColourChange)
                                                                .width(300),
                                                ].align_y(Center),
                                                column![
                                                        Space::new(0, 20),
                                                        text("Text will look like this.".to_string())
                                                                .size(self.tab_text_size as f32)
                                                                .font(Font::MONOSPACE),
                                                        text("Chords will look like this.".to_string())
                                                                .size(self.tab_text_size as f32)
                                                                .font(Font::MONOSPACE)
                                                                .color(self.chord_colour),
                                                ],
                                        ].spacing(20),
                                ].padding(10)
                                .spacing(20)
                        }
                };
                
                let bar_button = |label| {
                        button(row![
                                        label
                                ].align_y(Center))
                                .padding([4, 12])
                };
                // The header bar containing basic controls
                let controls: Row<Message> = match self.screen {
                        Screen::Tabs => { row![
                                        bar_button("Tab")
                                                .on_press(Message::TabsPage),
                                        bar_button("Search")
                                                .on_press(Message::SearchPage)
                                                .style(button::secondary),
                                        bar_button("Settings")
                                                .on_press(Message::SettingsPage)
                                                .style(button::secondary),
                                        Space::new(100, 0),
                                        bar_button("Edit...")
                                                .on_press(Message::OpenTabFile),
                                        Space::new(10, 0),
                                        toggler(self.playing)
                                                .label("Play")
                                                .on_toggle(Message::PlayingToggled)
                                ].align_y(Center)
                        },
                        Screen::Search => {
                                let value = &self.search_value;
                                row![
                                        bar_button("Tab")
                                                .on_press(Message::TabsPage)
                                                .style(button::secondary),
                                        bar_button("Search")
                                                .on_press(Message::SearchPage),
                                        bar_button("Settings")
                                                .on_press(Message::SettingsPage)
                                                .style(button::secondary),
                                        Space::new(100, 0),
                                        text_input("Search a tab...", value)
                                                .on_input(Message::UpdateSearchBar)
                                                .width(300),
                                        bar_button("Go!")
                                                .on_press(Message::SearchTabs),
                                        Space::new(10, 0),
                                        bar_button("Clear")
                                                .on_press(Message::ClearSearch),
                                        Space::new(40, 0),
                                        combo_box(&self.tab_types, "Filter", self.search_filter.as_ref(), Message::ApplySearch)
                                                .width(300),
                                        bar_button("Reset")
                                                .on_press(Message::ResetFilter),
                                ].align_y(Center)
                        },
                        Screen::Settings => { row![
                                        bar_button("Tab")
                                                .on_press(Message::TabsPage)
                                                .style(button::secondary),
                                        bar_button("Search")
                                                .on_press(Message::SearchPage)
                                                .style(button::secondary),
                                        bar_button("Settings")
                                                .on_press(Message::SettingsPage),
                                        Space::new(100, 0),
                                ].align_y(Center)
                        },
                }.padding(10)
                .spacing(2)
                .height(self.bar_height);

                // A bar to show messages to the user
                let mut notifications: Column<'_, Message> = column![];
                for notification in self.notifications.clone() {
                        match notification {
                                NotificationType::Info(n) => {
                                        notifications = notifications.push(row![
                                                text(n),
                                        ])
                                },
                                NotificationType::Warning(n) => {
                                        notifications = notifications.push(row![
                                                text(n).color(color![0xdf8e1d]),
                                        ])
                                },
                                NotificationType::Error(n) => {
                                        notifications = notifications.push(row![
                                                text(n).color(color![0xfe640b]),
                                        ])
                                },
                                NotificationType::Fatal(n) => {
                                        notifications = notifications.push(row![
                                                text(n).color(color![0xe64553]),
                                                Space::new(10, 0),
                                                button("Close")
                                                        .on_press(Message::CloseApp),
                                        ].align_y(Center))
                                },
                                _ => (),
                        };
                }
                let message_bar: Row<Message>;
                if self.notifications.len() != 0 {
                        message_bar = row![
                                button("Clear")
                                        .on_press(Message::ClearNotifications),
                                Space::new(10, 0),
                                notifications,
                        ].padding(self.main_padding)
                        .align_y(Center);
                } else {
                        message_bar = row![];
                }
                

                column![controls, message_bar, contents]

        }

        fn subscription(&self) -> Subscription<Message> {
                if let SearchState::Searching = self.search_state {
                        time::every(Duration::from_millis(200)).map(|_| Message::Update)
                } else if self.playing {
                        time::every(Duration::from_millis(200)).map(|_| Message::Update)
                } else {
                        event::listen().map(Message::EventOccurred)
                }
        }


        pub fn update(&mut self, message: Message) {
                // First parts are updating the UI

                // Update dependent of UI location
                match self.screen {
                        Screen::Settings => {
                                self.theme = get_selected_theme(&mut self.config);
                        },
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
                                let song_uid: String = match song_metadata.clone().unwrap().track_id() {
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
                                let song_name: String = match &song_metadata.clone().unwrap().title() {
                                        Some(t) => t.to_string(),
                                        None => "unknown".into()
                                };
                                let song_artist: String = match &song_metadata.unwrap().artists() {
                                        Some(a) => a[0].into(),
                                        None => "unknown".into()
                                };
                                if self.playing {
                                        if self.song_id_previoes_cycle != song_uid {
                                                self.get_song_data_by_uid(&song_uid, 
                                                        format!("{} {}", song_name, song_artist),
                                                        true);
                                        }
                                        self.song_id_previoes_cycle = song_uid.clone();
                                }
                                if self.song_id_display != song_uid {
                                        self.get_song_data_by_uid(&song_uid,
                                                format!("{} {}", song_name, song_artist),
                                                false);
                                }
                                self.current_song_uid = song_uid.into();
                        }
                }

                // Check if any thread returned a value
                if let Ok(v) = self.channel.1.try_recv() {
                        match v {
                                Value::Notification(n) => self.notifications.push(n),
                                Value::SearchResults(s) => self.search_state = SearchState::Finished(s),
                                _ => (),
                        }
                } 

                // Execute commands associated to messages
                match message {
                        Message::TabsPage => self.screen = Screen::Tabs,
                        Message::SearchPage => self.screen = Screen::Search,
                        Message::SettingsPage => self.screen = Screen::Settings,

                        Message::PlayingToggled(s) => self.playing = s,
                        Message::UpdateSearchBar(s) => self.search_value = s,
                        Message::SearchTabs => self.search_tabs(),
                        Message::ClearSearch => self.search_value = "".into(),
                        Message::ClearNotifications => self.notifications = vec![],
                        Message::CloseApp => exit(1),
                        Message::DownloadTab(url) => self.spawn_get_tab_thread(url, self.current_song_uid.to_owned()),
                        Message::ApplySearch(f) => {
                                self.search_filter = Some(f);
                                let _ = self.config.set("search_filter", Value::DataSetTypeOption(Some(f)));
                        },
                        Message::ResetFilter => {
                                let _ = self.config.set("search_filter", Value::DataSetTypeOption(None));
                                self.search_filter = None;
                        },
                        Message::ApplyTheme(t) => {
                                let _ = self.config.set("theme", Value::String(t.to_string()));
                                self.selected_theme = Some(t.clone());
                                self.theme = t;
                        },
                        Message::ApplySearchDepth(d) => {
                                self.search_depth = d;
                                let _ = self.config.set("search_depth", Value::Int(d as i64));
                        },
                        Message::SetTabLineHeight(h) => {
                                self.tab_text_size = h;
                                let _ = self.config.set("tab_text_size", Value::Int(h as i64));
                        }
                        Message::SetDownloadableOnly(s) => {
                                self.only_downloadable_results = s;
                                let _ = self.config.set("only_downloadable_results", Value::Bool(s));
                        },
                        Message::RemoveEmptyLines(s) => {
                                self.remove_empty_lines = s;
                                let _ = self.config.set("remove_empty_lines", Value::Bool(s));
                        },
                        Message::RemoveFirstLines(s) => {
                                self.remove_first_lines = s;
                                let _ = self.config.set("remove_first_lines", Value::Bool(s));
                        },
                        Message::ChordColourChange(c) => {
                                let colour = Color::parse(&c).unwrap_or(Color::parse("#fe640b").unwrap());
                                let _ = self.config.set("chord_colour", Value::String(c.clone()));
                                self.chord_colour = colour;
                                self.chord_colour_text = c;
                        },
                        Message::OpenTabFile => {
                                if let Err(e) = opener::open(std::path::Path::new(&self.current_path)) {
                                        self.show_info(NotificationType::Error(format!("Could not open file: {}", e.to_string())));
                                }
                        },
                        Message::EventOccurred(e) => match e {
                                Event::Window(window::Event::Opened { position: _, size: s }) => {
                                        self.size = s;
                                        self.bar_height = self.size.height / 20.0;
                                },
                                Event::Window(window::Event::Resized(s)) => {
                                        self.size = s;
                                        self.bar_height = self.size.height / 20.0;
                                },
                                _ => (),
                        },
                        _ => (),
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

        /// Search for the selected query
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
                self.current_tab = song;
        }

        /// Load song data or ask for download
        pub fn get_song_data_by_uid(&mut self, song_uid: &String, search_query: String, ask: bool) {
                match system::get_tab_path() {
                        Ok(mut p) => {
                                p.pop();
                                p.push(TAB_DIR);
                                p.push(song_uid.clone() + ".yml");

                                self.current_path = p.to_str().unwrap().to_owned();

                                if self.playing {
                                        if p.is_file() {
                                                match load_song(&song_uid) {
                                                        Ok(s) => self.song_to_ui(s, song_uid.to_owned()),
                                                        Err(e) => self.show_info(NotificationType::Error("Could not load song file: ".to_string() + &e.to_string())),
                                                }
                                        } else if ask {
                                                self.screen = Screen::Search;
                                                self.search_value = search_query;
                                                self.search_tabs();
                                        }
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

fn get_selected_theme(config_object: &mut CoralConfig) -> Theme {
        let selected = if let Value::String(t) = config_object.get("theme"){t}else{"".into()};
        for theme in Theme::ALL {
                if selected.contains(theme.to_string().as_str()) {
                        return theme.to_owned()
                }
        }
        Theme::CatppuccinMocha
}

fn main() {
        let _ = iced::application("Coral-Chords", ApplicationState::update, ApplicationState::view)
                .centered()
                .theme(ApplicationState::theme)
                .subscription(ApplicationState::subscription)
                .run();
}