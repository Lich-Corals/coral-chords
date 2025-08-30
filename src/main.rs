// Coral-Chords - Automatic chords for Spotify
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
use iced::time::{self, Duration};
use iced::widget::{
        button, checkbox, column, combo_box, container, rich_text, row, scrollable, slider, span,
        text, text_input, toggler, Column, Row, Space,
};
use iced::Alignment::Center;
use iced::{color, window};
use iced::{event, font, Color, Event, Font, Size, Subscription, Theme};
use mpris::{Metadata, Player, PlayerFinder};
use std::process::exit;
use std::sync::mpsc::{self};
use std::thread::{self};
use std::vec;
use ug_scraper::types::{DataSetType, DataType, SearchResult, Song, SUPPORTED_DOWNLOAD_TYPES};

use crate::backend::formats::{
        CoralConfig, LoggedSong, NotificationType, ThreadData, Value, TAB_DIR,
};
use crate::backend::network::check_for_newer_version;
use crate::backend::system::{add_to_song_log, current_time, get_config, load_song, store_song};
use crate::backend::{network, system};

/// The application's properties
struct ApplicationState {
        screen: Screen,
        theme: Theme,
        /// The theme selected from the drop-down
        selected_theme: Option<Theme>,
        /// The globally used config object
        config: CoralConfig,
        /// The channel to communicate with other threads
        channel: (
                std::sync::mpsc::Sender<ThreadData>,
                std::sync::mpsc::Receiver<ThreadData>,
        ),
        /// Whether to check for a song-change.
        playing: bool,
        /// Notifications which will be sent to user using the notification bar
        ///
        /// Those notifications shall be received from show_info()
        notifications: Vec<NotificationType>,
        /// The song that was playing during the last cycle.
        /// If changed, it will trigger an update of the UI or a prompt to download a song.
        song_id_previoes_cycle: String,
        /// This will be used to check whether the display is showing the currently running song.
        /// It will not be the case if no tab is locally stored; in this case, the song will be shown on UI as soon as it is downloaded.
        song_id_display: String,
        /// The MPRIS player object used to communicate with audio players
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
        /// Whether un-downloadable search results are disabled
        only_downloadable_results: bool,
        /// The currently displayed tab
        current_tab: Song,
        /// The configured size of a line of a tab
        tab_text_size: u8,
        /// The configured chord colour
        chord_colour: Color,
        /// The value inside the chord colour input box
        chord_colour_text: String,
        /// The padding of the main window contents
        main_padding: f32,
        /// Whether the lines before the first chords will be removed
        remove_first_lines: bool,
        /// Whether empty lines will be removed from tabs
        remove_empty_lines: bool,
        /// Path to the current tab file
        current_path: String,
        /// The configured colour of section headers
        header_colour: Color,
        /// The text displayed in the header colour selection field
        header_colour_text: String,
        /// Whether automatic search queries should be cleaned
        clean_queries: bool,
        /// Whether to show a notification to the user if a new version of the package is available
        notify_about_updates: bool,
        /// Whether to keep a local log of the songs played
        log_played_songs: bool,
        /// The length of the currently playing song in seconds
        current_song_length: u64,
        /// The time stamp at which the currently playing song was started
        start_time_stamp: u64,
        /// Whether the currently playing song was already added to the log file
        song_in_log: bool,
        /// the currently selected metadata colour
        metadata_colour: Color,
        /// The metadata colour input field's text
        metadata_colour_text: String,
        /// The interval in which the user is reminded of string renewal
        ///
        /// This value is negative if the setting is disabled.
        renew_strings_interval: i64,
        /// The last time the user has changed their strings
        last_string_renewal: i64,
}

impl Default for ApplicationState {
        fn default() -> Self {
                let mut config_result = get_config_object();
                let mut notifications: Vec<NotificationType> = vec![];

                if let Some(n) = config_result.1 {
                        notifications.push(n);
                }

                let player = match PlayerFinder::new() {
                        Ok(pf) => {
                                let player_option: Option<Player> = match pf.iter_players() {
                                        Ok(pli) => {
                                                let mut spotify_player: Option<Player> = None;
                                                for player in pli {
                                                        if let Ok(pl) = player
                                                                && pl.bus_name() == "org.mpris.MediaPlayer2.spotify" {
                                                                        spotify_player = Some(pl);
                                                                        break;
                                                        }
                                                }
                                                spotify_player
                                        }
                                        Err(error) => {
                                                notifications.push(NotificationType::Fatal(
                                                        format!(
                                                        "Could not iterate players: {error:?}"
                                                ),
                                                ));
                                                None
                                        }
                                };
                                if player_option.is_none() {
                                        notifications.push(NotificationType::Fatal(
                                                "Could not find Spotify player.".into(),
                                        ));
                                }
                                player_option
                        }
                        Err(e) => {
                                notifications.push(NotificationType::Fatal(
                                        "Could not connect to D-Bus: ".to_string() + &e.to_string(),
                                ));
                                None
                        }
                };

                // Load settings from config file on initialization of the application
                let search_filter =
                        if let Value::DataSetTypeOption(o) = config_result.0.get("search_filter") {
                                o
                        } else {
                                None
                        };
                let search_depth = if let Value::Int(v) = config_result.0.get("search_depth") {
                        v
                } else {
                        2
                };
                let only_downloadable_results =
                        if let Value::Bool(v) = config_result.0.get("only_downloadable_results") {
                                v
                        } else {
                                false
                        };
                let remove_first_lines =
                        if let Value::Bool(v) = config_result.0.get("remove_first_lines") {
                                v
                        } else {
                                false
                        };
                let remove_empty_lines =
                        if let Value::Bool(v) = config_result.0.get("remove_empty_lines") {
                                v
                        } else {
                                false
                        };
                let last_string_renewal =
                        if let Value::Int(v) = config_result.0.get("last_string_renewal") {
                                v
                        } else {
                                0
                        };
                let renew_strings_interval =
                        if let Value::Int(v) = config_result.0.get("renew_strings_interval") {
                                v
                        } else {
                                -60 * 60 * 24 * 31 * 3 // This is about three months; negative to disable feature by default.
                        };
                let tab_text_size = if let Value::Int(v) = config_result.0.get("tab_text_size") {
                        v
                } else {
                        15
                };
                let loaded_colour = if let Value::String(v) = config_result.0.get("chord_colour") {
                        v
                } else {
                        "#fe640b".into()
                };
                let loaded_metadata_colour =
                        if let Value::String(v) = config_result.0.get("metadata_colour") {
                                v
                        } else {
                                "#209fb5".into()
                        };
                let loaded_header_colour =
                        if let Value::String(v) = config_result.0.get("header_colour") {
                                v
                        } else {
                                "#dc8a78".into()
                        };
                let chord_colour =
                        Color::parse(&loaded_colour).unwrap_or(Color::parse("fe640b").unwrap());
                let header_colour = Color::parse(&loaded_header_colour)
                        .unwrap_or(Color::parse("dc8a78").unwrap());
                let metadata_colour = Color::parse(&loaded_metadata_colour)
                        .unwrap_or(Color::parse("209fb5").unwrap());
                let clean_queries = if let Value::Bool(v) = config_result.0.get("clean_queries") {
                        v
                } else {
                        true
                };
                let notify_about_updates =
                        if let Value::Bool(v) = config_result.0.get("notify_about_updates") {
                                v
                        } else {
                                true
                        };

                let first_launch = if let Value::Bool(v) = config_result.0.get("first_launch") {
                        v
                } else {
                        true
                };
                let screen: Screen;
                if first_launch {
                        screen = Screen::Welcome;
                        let _ = config_result.0.set("first_launch", Value::Bool(false));
                } else {
                        screen = Screen::default();
                }

                let log_played_songs =
                        if let Value::Bool(v) = config_result.0.get("log_played_songs") {
                                v
                        } else {
                                true
                        };

                if notify_about_updates && check_for_newer_version() {
                        notifications.push(NotificationType::Info(
                                "A newer version of the crate is available for download.".into(),
                        ));
                }

                if renew_strings_interval > 0
                        && renew_strings_interval + last_string_renewal <= current_time() as i64
                {
                        notifications.push(NotificationType::RenewStrings(
                                "Your strings should be renewed!".into(),
                        ));
                }

                ApplicationState {
                        screen,
                        theme: get_selected_theme(&mut config_result.0),
                        selected_theme: Some(get_selected_theme(&mut config_result.0)),
                        search_depth: search_depth as u8,
                        config: config_result.0,
                        channel: mpsc::channel::<ThreadData>(),
                        playing: false,
                        notifications,
                        song_id_previoes_cycle: String::new(),
                        song_id_display: String::new(),
                        player,
                        search_value: String::new(),
                        search_state: SearchState::default(),
                        current_song_uid: String::new(),
                        themes: combo_box::State::new(Theme::ALL.to_vec()),
                        tab_types: combo_box::State::new(SUPPORTED_DOWNLOAD_TYPES.to_vec()),
                        search_filter,
                        size: Size::default(),
                        bar_height: 0.0,
                        only_downloadable_results,
                        current_tab: Song::default(),
                        tab_text_size: tab_text_size as u8,
                        chord_colour,
                        chord_colour_text: loaded_colour,
                        main_padding: 10.0,
                        remove_empty_lines,
                        remove_first_lines,
                        current_path: String::new(),
                        header_colour,
                        header_colour_text: loaded_header_colour,
                        clean_queries,
                        notify_about_updates,
                        log_played_songs,
                        current_song_length: 0,
                        start_time_stamp: 0,
                        song_in_log: true,
                        metadata_colour,
                        metadata_colour_text: loaded_metadata_colour,
                        renew_strings_interval,
                        last_string_renewal,
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
        /// An iced event occurred
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
        /// Open the licence
        OpenLicence,
        /// Open the readme page
        OpenReadme,
        /// Header colour changed
        HeaderColourChange(String),
        /// Reload the displayed tab
        Reload,
        /// Set query cleaning
        CleanQueries(bool),
        /// Toggle update notifications
        ToggleUpdateNotification(bool),
        /// Whether to log the played songs to a local file or not
        TogglePlayedLog(bool),
        /// Update the colour of the metadata section
        RecolourMetadata(String),
        /// Update the string renewal interval time
        UpdateStringRenewalInterval(u32),
        /// Enable/disable the string renewal reminder
        ToggleStringRenewalReminder(bool),
        /// Set the last string renewal time to the current time stamp
        UpdateStringRenewalTime,
        /// Add two weeks to the last string renewal time
        AddToStringRenewalTime,
        /// Open the coffee page
        OpenCoffeePage,
}

#[derive(Default, PartialEq)]
enum Screen {
        #[default]
        Tabs,
        Search,
        Settings,
        Welcome,
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
                        Screen::Welcome => column![scrollable(column![
                                column![
                                                text(format!("Welcome to Coral-Chords v{}!", env!("CARGO_PKG_VERSION")))
                                                        .size(25),
                                                Space::new(0, 20),
                                                row![
                                                        text("Thank you for trying it out!"),
                                                ],
                                                Space::new(0, 10),
                                                row![
                                                        text("For usage and configuration instructions, take a look at "),
                                                        rich_text([span(
                                                                "the GitHub repository.")
                                                                .link(Message::OpenReadme)
                                                                .underline(true),
                                                        ]),
                                                ],
                                                row![
                                                        text("If you experience any problems or have a suggestion about the application, please submit an issue on the GitHub page above."),
                                                ],
                                                row![
                                                        text!("And if you like this program, maybe think about "),
                                                        rich_text([span(
                                                                "leaving a tip.")
                                                                .link(Message::OpenCoffeePage)
                                                                .underline(true),
                                                        ]),
                                                ],
                                                Space::new(0, 30),
                                                row![
                                                        rich_text([span(
                                                                "Created with ")
                                                                        .color(color!(0x696969))
                                                                        .size(12),
                                                        ]),
                                                        rich_text([span(
                                                                "❤️")
                                                                        .color(color!(0x933030))
                                                                        .size(12),
                                                        ]),
                                                        rich_text([span(
                                                                " by Linus Tibert (Lich-Corals)")
                                                                        .color(color!(0x696969))
                                                                        .size(12),
                                                        ]),
                                                ],
                                                Space::new(0, 5),
                                                rich_text([span(
                                                        format!("Coral-Chords v{}  Copyright (C) 2025  Linus Tibert", env!("CARGO_PKG_VERSION")))
                                                                .color(color!(0x696969))
                                                                .size(11),
                                                ]),
                                                rich_text([span(
                                                        "GNU Affero General Public Licence v3")
                                                                .color(color!(0x696969))
                                                                .link(Message::OpenLicence)
                                                                .size(11)
                                                                .underline(true),
                                                ]),
                                                text("")
                                                        .size(12),
                                                text("")
                                                        .size(12)
                                        ].align_x(Center),
                        ].padding(10)
                                .spacing(20)
                                .align_x(Center)
                                .width(self.size.width - 2.0 * self.main_padding))],
                        Screen::Tabs => {
                                if !self.current_tab.lines.is_empty() {
                                        let mut main_row: Row<'_, Message> = row![];
                                        let mut new_column = column![];
                                        if let Some(d) = &self.current_tab.metadata {
                                                if let Some(d) = &d.capo {
                                                        new_column = new_column.push(row![
                                                                text(format!("Capo: {}", d.clone()))
                                                                        .color(self.metadata_colour)
                                                                        .font(Font::MONOSPACE)
                                                                        .size(self.tab_text_size as f32)
                                                        ].height(self.tab_text_size as f32));
                                                }
                                                if let Some(d) = &d.tuning {
                                                        new_column = new_column.push(row![
                                                                text(format!("Tuning: {}", d.clone()))
                                                                        .color(self.metadata_colour)
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
                                                                        if !self.remove_empty_lines || !line.text_data.is_empty() {
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
                                                                                                .size(self.tab_text_size as f32)
                                                                                                .color(self.header_colour),
                                                                                ]),
                                                                        ].height(self.tab_text_size as f32));
                                                                }
                                                        }
                                                        if (lines_on_column >= max_lines_per_column as u16)
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
                                                if sorted_results.is_empty() {
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
                                .width(self.size.width - 2.0 * self.main_padding)
                        },
                        Screen::Settings => {
                                column![scrollable(column![
                                        column![
                                                text("Application")
                                                        .size(25),
                                                row![
                                                        text("Theme"),
                                                        Space::new(10, 0),
                                                        combo_box(&self.themes, "Theme", self.selected_theme.as_ref(), Message::ApplyTheme)
                                                                .width(300),
                                                ].align_y(Center),
                                                row![
                                                        checkbox("Notify about updates", self.notify_about_updates)
                                                                .on_toggle(Message::ToggleUpdateNotification),
                                                ].align_y(Center),
                                                row![
                                                        checkbox("Log played songs locally", self.log_played_songs)
                                                                .on_toggle(Message::TogglePlayedLog),
                                                ].align_y(Center),
                                                row![
                                                        checkbox("Remind me to replace my strings", self.renew_strings_interval > 0)
                                                                .on_toggle(Message::ToggleStringRenewalReminder),
                                                ].align_y(Center),
                                                row![
                                                        text(format!("String reminder interval: {:02}mo, {:02}d",
                                                                self.renew_strings_interval.abs() / (60*60*24*31),
                                                                (self.renew_strings_interval.abs() % (60*60*24*31)) / (60*60*24))),
                                                        Space::new(10, 0),
                                                        slider(1..=12*31, (self.renew_strings_interval.abs() / (60*60*24)) as u32, Message::UpdateStringRenewalInterval)
                                                                .width(300),
                                                ].align_y(Center),
                                        ].spacing(20)
                                        .align_x(Center),
                                        column![
                                                text("Search")
                                                        .size(25),
                                                row![
                                                        checkbox("Only show downloadable search results", self.only_downloadable_results)
                                                                .on_toggle(Message::SetDownloadableOnly),
                                                ].align_y(Center),
                                                row![
                                                        checkbox("Auto-clean search queries", self.clean_queries)
                                                                .on_toggle(Message::CleanQueries),
                                                ].align_y(Center),
                                                row![
                                                        text(format!("Search depth: {:02}", self.search_depth)),
                                                        Space::new(10, 0),
                                                        slider(1..=64, self.search_depth, Message::ApplySearchDepth)
                                                                .width(300),
                                                ].align_y(Center),
                                        ].spacing(20)
                                        .align_x(Center),
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
                                                        text("Metadata colour:"),
                                                        Space::new(10, 0),
                                                        text_input("#209fb5", &self.metadata_colour_text)
                                                                .on_input(Message::RecolourMetadata)
                                                                .width(300),
                                                ].align_y(Center),
                                                row![
                                                        text("Header colour:"),
                                                        Space::new(10, 0),
                                                        text_input("#dc8a78", &self.header_colour_text)
                                                                .on_input(Message::HeaderColourChange)
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
                                                        text("Some matadata\n ".to_string())
                                                                .size(self.tab_text_size as f32)
                                                                .font(Font::MONOSPACE)
                                                                .color(self.metadata_colour),
                                                        rich_text([span(
                                                                "[Section header]")
                                                                        .font(Font {
                                                                                style: font::Style::Italic,
                                                                                ..Font::MONOSPACE })
                                                                        .size(self.tab_text_size as f32)
                                                                        .color(self.header_colour),
                                                        ]),
                                                        text("A         C     A       B".to_string())
                                                                .size(self.tab_text_size as f32)
                                                                .font(Font::MONOSPACE)
                                                                .color(self.chord_colour),
                                                        text("Tabs will look like this.".to_string())
                                                                .size(self.tab_text_size as f32)
                                                                .font(Font::MONOSPACE),
                                                ],
                                        ].spacing(20)
                                        .align_x(Center),
                                        column![
                                                Space::new(0, 20),
                                                row![
                                                        text("View "),
                                                        rich_text([span(
                                                        "the repository")
                                                                .link(Message::OpenReadme)
                                                                .underline(true),
                                                        ]),
                                                        text(" for more information about the settings"),
                                                ],
                                                text("and how to use this program."),
                                                Space::new(0, 30),
                                                row![
                                                        rich_text([span(
                                                                "Created with ")
                                                                        .color(color!(0x696969))
                                                                        .size(12),
                                                        ]),
                                                        rich_text([span(
                                                                "❤️")
                                                                        .color(color!(0x933030))
                                                                        .size(12),
                                                        ]),
                                                        rich_text([span(
                                                                " by Linus Tibert (Lich-Corals)")
                                                                        .color(color!(0x696969))
                                                                        .size(12),
                                                        ]),
                                                ],
                                                Space::new(0, 5),
                                                rich_text([span(
                                                        format!("Coral-Chords v{}  Copyright (C) 2025  Linus Tibert", env!("CARGO_PKG_VERSION")))
                                                                .color(color!(0x696969))
                                                                .size(11),
                                                ]),
                                                rich_text([span(
                                                        "GNU Affero General Public Licence v3")
                                                                .color(color!(0x696969))
                                                                .link(Message::OpenLicence)
                                                                .size(11)
                                                                .underline(true),
                                                ]),
                                                text("")
                                                        .size(12),
                                                text("")
                                                        .size(12)
                                        ].align_x(Center),
                                ].padding(10)
                                .spacing(20)
                                .align_x(Center)
                                .width(self.size.width - 2.0 * self.main_padding)
                                )]
                        }
                };

                let bar_button = |label| button(row![label].align_y(Center)).padding([4, 12]);
                // The header bar containing basic controls
                let controls: Row<Message> = match self.screen {
                        Screen::Welcome => row![
                                bar_button("Tab")
                                        .on_press(Message::TabsPage)
                                        .style(button::secondary),
                                bar_button("Search")
                                        .on_press(Message::SearchPage)
                                        .style(button::secondary),
                                bar_button("Settings")
                                        .on_press(Message::SettingsPage)
                                        .style(button::secondary),
                                Space::new(100, 0),
                        ]
                        .align_y(Center),
                        Screen::Tabs => row![
                                bar_button("Tab").on_press(Message::TabsPage),
                                bar_button("Search")
                                        .on_press(Message::SearchPage)
                                        .style(button::secondary),
                                bar_button("Settings")
                                        .on_press(Message::SettingsPage)
                                        .style(button::secondary),
                                Space::new(100, 0),
                                bar_button("Edit...").on_press(Message::OpenTabFile),
                                Space::new(10, 0),
                                bar_button("Reload").on_press(Message::Reload),
                                Space::new(10, 0),
                                toggler(self.playing)
                                        .label("Play")
                                        .on_toggle(Message::PlayingToggled)
                        ]
                        .align_y(Center),
                        Screen::Search => {
                                let value = &self.search_value;
                                row![
                                        bar_button("Tab")
                                                .on_press(Message::TabsPage)
                                                .style(button::secondary),
                                        bar_button("Search").on_press(Message::SearchPage),
                                        bar_button("Settings")
                                                .on_press(Message::SettingsPage)
                                                .style(button::secondary),
                                        Space::new(100, 0),
                                        text_input("Search a tab...", value)
                                                .on_input(Message::UpdateSearchBar)
                                                .width(300),
                                        bar_button("Go!").on_press(Message::SearchTabs),
                                        Space::new(10, 0),
                                        bar_button("Clear").on_press(Message::ClearSearch),
                                        Space::new(40, 0),
                                        combo_box(
                                                &self.tab_types,
                                                "Filter",
                                                self.search_filter.as_ref(),
                                                Message::ApplySearch
                                        )
                                        .width(300),
                                        bar_button("Reset").on_press(Message::ResetFilter),
                                ]
                                .align_y(Center)
                        }
                        Screen::Settings => row![
                                bar_button("Tab")
                                        .on_press(Message::TabsPage)
                                        .style(button::secondary),
                                bar_button("Search")
                                        .on_press(Message::SearchPage)
                                        .style(button::secondary),
                                bar_button("Settings").on_press(Message::SettingsPage),
                                Space::new(100, 0),
                        ]
                        .align_y(Center),
                }
                .padding(10)
                .spacing(2);

                // A bar to show messages to the user
                let mut notifications: Column<'_, Message> = column![];
                for notification in self.notifications.clone() {
                        match notification {
                                NotificationType::Info(n) => {
                                        notifications = notifications.push(row![text(n),])
                                }
                                NotificationType::Warning(n) => {
                                        notifications = notifications
                                                .push(row![text(n).color(color![0xdf8e1d]),])
                                }
                                NotificationType::Error(n) => {
                                        notifications = notifications
                                                .push(row![text(n).color(color![0xfe640b]),])
                                }
                                NotificationType::Fatal(n) => {
                                        notifications = notifications.push(row![
                                                text(n).color(color![0xe64553]),
                                                Space::new(10, 0),
                                                button("Close").on_press(Message::CloseApp),
                                        ]
                                        .align_y(Center))
                                }
                                NotificationType::RenewStrings(n) => {
                                        notifications = notifications.push(row![
                                                button("Done")
                                                        .on_press(Message::UpdateStringRenewalTime),
                                                Space::new(10, 0),
                                                button("Add 2 weeks")
                                                        .on_press(Message::AddToStringRenewalTime),
                                                Space::new(10, 0),
                                                text(n).color(color![0xdf8e1d]),
                                        ]
                                        .align_y(Center))
                                }
                                _ => (),
                        };
                }
                let message_bar: Row<Message> = if !self.notifications.is_empty() {
                        row![
                                button("Clear").on_press(Message::ClearNotifications),
                                Space::new(10, 0),
                                notifications,
                        ]
                        .padding(self.main_padding)
                        .align_y(Center)
                } else {
                        row![]
                };

                column![controls, message_bar, contents]
        }

        fn subscription(&self) -> Subscription<Message> {
                if let SearchState::Searching = self.search_state {
                        time::every(Duration::from_millis(200)).map(|_| Message::Update)
                } else if self.playing {
                        Subscription::batch(vec![
                                time::every(Duration::from_millis(200)).map(|_| Message::Update),
                                event::listen().map(Message::EventOccurred),
                        ])
                } else {
                        event::listen().map(Message::EventOccurred)
                }
        }

        pub fn update(&mut self, message: Message) {
                // First parts are updating the UI

                // Update dependent of UI location
                if self.screen == Screen::Settings {
                        self.theme = get_selected_theme(&mut self.config);
                }

                // Following is updating the program's main logic
                let song_metadata: Option<Metadata>;
                if self.player.is_some() {
                        let player = self.player.as_ref().unwrap();

                        song_metadata = match player.get_metadata() {
                                Ok(md) => Some(md),
                                Err(e) => {
                                        self.show_info(NotificationType::Error(
                                                "Could not connect to D-Bus: ".to_string()
                                                        + &e.to_string(),
                                        ));
                                        None
                                }
                        };
                        if let Some(song_metadata) = song_metadata {
                                let song_uid: String = match song_metadata.track_id() {
                                        Some(id) => {
                                                if id.to_string().contains("spotify") {
                                                        id.to_string()
                                                                .replace("/com/spotify/track/", "")
                                                                .replace("/", "")
                                                                .replace(" ", "")
                                                } else {
                                                        self.show_info(NotificationType::Error("This song doesn't have a recognized song ID format.".into()));
                                                        "unknown".into()
                                                }
                                        }
                                        None => {
                                                self.show_info(NotificationType::Error(
                                                        "Could not get song ID.".into(),
                                                ));
                                                "unknown".into()
                                        }
                                };
                                let song_name: String = match &song_metadata.title() {
                                        Some(t) => t.to_string(),
                                        None => "unknown".into(),
                                };
                                let song_artist: String = match &song_metadata.artists() {
                                        Some(a) => a[0].into(),
                                        None => "unknown".into(),
                                };
                                let song_length: u64 = match &song_metadata.length_in_microseconds()
                                {
                                        Some(l) => *l / 1_000_000,
                                        None => 180,
                                };
                                self.current_song_length = song_length;
                                if self.playing {
                                        if self.song_id_previoes_cycle != song_uid {
                                                self.start_time_stamp = current_time();
                                                self.song_in_log = false;
                                                self.screen = Screen::Tabs;
                                                self.get_song_data_by_uid(
                                                        &song_uid,
                                                        format!(
                                                                "{} {}",
                                                                self.clean_search_query(&song_name),
                                                                song_artist
                                                        ),
                                                        true,
                                                );
                                        }
                                        self.song_id_previoes_cycle = song_uid.clone();
                                }
                                if self.song_id_display != song_uid {
                                        self.song_in_log = false;
                                        self.start_time_stamp = current_time();
                                        self.get_song_data_by_uid(
                                                &song_uid,
                                                format!(
                                                        "{} {}",
                                                        self.clean_search_query(&song_name),
                                                        song_artist
                                                ),
                                                false,
                                        );
                                }
                                if current_time() - self.start_time_stamp
                                        >= self.current_song_length / 2
                                        && self.log_played_songs
                                        && !self.song_in_log
                                {
                                        let current_song = LoggedSong {
                                                name: self.current_tab.basic_data.title.clone(),
                                                artist: self.current_tab.basic_data.artist.clone(),
                                                id: self.current_song_uid.clone(),
                                                length_s: self.current_song_length,
                                                timestamp: current_time(),
                                        };
                                        add_to_song_log(current_song).unwrap_or_else(|error| {
                                            self.show_info(NotificationType::Error(format!("Something went wrong adding the current song to the log: {}", error)));
                                        });
                                        self.song_in_log = true;
                                }
                                self.current_song_uid = song_uid;
                        }
                }

                // Check if any thread returned a value
                if let Ok(v) = self.channel.1.try_recv() {
                        match v {
                                ThreadData::Notification(n) => self.notifications.push(n),
                                ThreadData::SearchResults(s) => {
                                        self.search_state = SearchState::Finished(s)
                                }
                                ThreadData::DownloadFinishedSignal => {
                                        if self.playing {
                                                self.screen = Screen::Tabs;
                                        }
                                }
                                _ => (),
                        }
                }

                // Execute commands associated to messages
                match message {
                        Message::TabsPage => self.screen = Screen::Tabs,
                        Message::SearchPage => self.screen = Screen::Search,
                        Message::SettingsPage => self.screen = Screen::Settings,

                        Message::PlayingToggled(s) => {
                                self.playing = s;
                                self.start_time_stamp = current_time();
                        }
                        Message::UpdateSearchBar(s) => self.search_value = s,
                        Message::SearchTabs => self.search_tabs(),
                        Message::ClearSearch => self.search_value = "".into(),
                        Message::ClearNotifications => self.notifications = vec![],
                        Message::CloseApp => exit(1),
                        Message::DownloadTab(url) => {
                                self.get_tab(url, self.current_song_uid.to_owned())
                        }
                        Message::ApplySearch(f) => {
                                self.search_filter = Some(f);
                                let _ = self
                                        .config
                                        .set("search_filter", Value::DataSetTypeOption(Some(f)));
                        }
                        Message::ResetFilter => {
                                let _ = self
                                        .config
                                        .set("search_filter", Value::DataSetTypeOption(None));
                                self.search_filter = None;
                        }
                        Message::ApplyTheme(t) => {
                                let _ = self.config.set("theme", Value::String(t.to_string()));
                                self.selected_theme = Some(t.clone());
                                self.theme = t;
                        }
                        Message::ToggleStringRenewalReminder(s) => {
                                if !s {
                                        self.renew_strings_interval =
                                                -self.renew_strings_interval.abs();
                                } else {
                                        self.renew_strings_interval =
                                                self.renew_strings_interval.abs();
                                        if self.last_string_renewal == 0 {
                                                self.last_string_renewal = current_time() as i64;
                                                let _ = self.config.set(
                                                        "last_string_renewal",
                                                        Value::Int(self.last_string_renewal),
                                                );
                                        }
                                }
                                let _ = self.config.set(
                                        "renew_strings_interval",
                                        Value::Int(self.renew_strings_interval),
                                );
                        }
                        Message::UpdateStringRenewalInterval(i) => {
                                if self.renew_strings_interval > 0 {
                                        self.renew_strings_interval = (i as i64) * 60 * 60 * 24;
                                } else {
                                        self.renew_strings_interval = -((i as i64) * 60 * 60 * 24);
                                }
                                let _ = self.config.set(
                                        "renew_strings_interval",
                                        Value::Int(self.renew_strings_interval),
                                );
                        }
                        Message::AddToStringRenewalTime => {
                                self.last_string_renewal += (60 * 60 * 24 * 14) as i64;
                                let _ = self.config.set(
                                        "last_string_renewal",
                                        Value::Int(self.last_string_renewal),
                                );
                                self.show_info(NotificationType::Info(
                                        "You will be reminded in 14 days.".to_string(),
                                ));
                        }
                        Message::UpdateStringRenewalTime => {
                                self.last_string_renewal = current_time() as i64;
                                let _ = self.config.set(
                                        "last_string_renewal",
                                        Value::Int(current_time() as i64),
                                );
                                self.show_info(NotificationType::Info(
                                        "String renewal noted.".to_string(),
                                ));
                        }
                        Message::ApplySearchDepth(d) => {
                                self.search_depth = d;
                                let _ = self.config.set("search_depth", Value::Int(d as i64));
                        }
                        Message::SetTabLineHeight(h) => {
                                self.tab_text_size = h;
                                let _ = self.config.set("tab_text_size", Value::Int(h as i64));
                        }
                        Message::SetDownloadableOnly(s) => {
                                self.only_downloadable_results = s;
                                let _ = self
                                        .config
                                        .set("only_downloadable_results", Value::Bool(s));
                        }
                        Message::RemoveEmptyLines(s) => {
                                self.remove_empty_lines = s;
                                let _ = self.config.set("remove_empty_lines", Value::Bool(s));
                        }
                        Message::ToggleUpdateNotification(s) => {
                                self.notify_about_updates = s;
                                let _ = self.config.set("notify_about_updates", Value::Bool(s));
                        }
                        Message::TogglePlayedLog(s) => {
                                self.log_played_songs = s;
                                let _ = self.config.set("log_played_songs", Value::Bool(s));
                        }
                        Message::RemoveFirstLines(s) => {
                                self.remove_first_lines = s;
                                let _ = self.config.set("remove_first_lines", Value::Bool(s));
                        }
                        Message::CleanQueries(s) => {
                                self.clean_queries = s;
                                let _ = self.config.set("clean_queries", Value::Bool(s));
                        }
                        Message::RecolourMetadata(c) => {
                                let colour = Color::parse(&c)
                                        .unwrap_or(Color::parse("#209fb5").unwrap());
                                let _ = self
                                        .config
                                        .set("metadata_colour", Value::String(c.clone()));
                                self.metadata_colour = colour;
                                self.metadata_colour_text = c;
                        }
                        Message::ChordColourChange(c) => {
                                let colour = Color::parse(&c)
                                        .unwrap_or(Color::parse("#fe640b").unwrap());
                                let _ = self.config.set("chord_colour", Value::String(c.clone()));
                                self.chord_colour = colour;
                                self.chord_colour_text = c;
                        }
                        Message::Reload => {
                                self.song_id_display = String::new();
                        }
                        Message::HeaderColourChange(c) => {
                                let colour = Color::parse(&c)
                                        .unwrap_or(Color::parse("#dc8a78").unwrap());
                                let _ = self.config.set("header_colour", Value::String(c.clone()));
                                self.header_colour = colour;
                                self.header_colour_text = c;
                        }
                        Message::OpenTabFile => {
                                if let Err(e) =
                                        opener::open(std::path::Path::new(&self.current_path))
                                {
                                        self.show_info(NotificationType::Error(format!(
                                                "Could not open file: {}",
                                                e
                                        )));
                                }
                        }
                        Message::OpenLicence => {
                                if let Err(e) = opener::open(std::path::Path::new(
                                        "https://www.gnu.org/licenses/agpl-3.0.en.html",
                                )) {
                                        self.show_info(NotificationType::Error(format!(
                                                "Could not open web link: {}",
                                                e
                                        )));
                                }
                        }
                        Message::OpenCoffeePage => {
                                if let Err(e) = opener::open(std::path::Path::new(
                                        "https://buymeacoffee.com/lichcorals",
                                )) {
                                        self.show_info(NotificationType::Error(format!(
                                                "Could not open web link: {}",
                                                e
                                        )));
                                }
                        }
                        Message::OpenReadme => {
                                if let Err(e) = opener::open(std::path::Path::new(
                                        "https://github.com/Lich-Corals/coral-chords",
                                )) {
                                        self.show_info(NotificationType::Error(format!(
                                                "Could not open web link: {}",
                                                e
                                        )));
                                }
                        }
                        Message::EventOccurred(e) => match e {
                                Event::Window(window::Event::Opened {
                                        position: _,
                                        size: s,
                                }) => {
                                        self.size = s;
                                        self.bar_height = self.size.height / 20.0;
                                }
                                Event::Window(window::Event::Resized(s)) => {
                                        self.size = s;
                                        self.bar_height = self.size.height / 20.0;
                                }
                                _ => (),
                        },
                        _ => (),
                }
        }

        /// Remove non-title elements from a song title (e.g., " - 2019 Remaster")
        fn clean_search_query(&self, query: &str) -> String {
                if self.clean_queries {
                        let split_markers: Vec<&str> = vec![" - ", " / ", " ("];
                        for split_marker in split_markers {
                                if query.contains(split_marker) {
                                        return query.split(split_marker).collect::<Vec<&str>>()[0]
                                                .to_string();
                                }
                        }
                }
                query.to_owned()
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
                let search_depth: u8 = match self.config.get("search_depth") {
                        Value::Int(d) => d as u8,
                        _ => 2,
                };
                thread::spawn(
                        move || {
                                match network::search(search_query.as_str(), search_depth) {
                        Ok(s) => {
                                if let Err(e) = tx.send(ThreadData::SearchResults(s))
                                        && let Err(e) = tx.send(ThreadData::Notification(
                                                NotificationType::Error("Could not send search results to main thread: ".to_string() + &e.to_string()))) {
                                        println!("Could not send message: {}", e);
                                }
                        },
                        Err(e) => if let Err(e) = tx.send(ThreadData::Notification(
                                        NotificationType::Error("Something went wrong getting the search results: ".to_string() + &e))) {
                                println!("Could not send message: {}", e);
                        },
                }
                        },
                );
        }

        /// Download and store a tab locally
        pub fn get_tab(&mut self, url: String, song_uid: String) {
                let tx = self.channel.0.clone();
                // A thread is spawned to prevent freezing UI
                thread::spawn(move || match network::get_tab(&url) {
                        Ok(s) => match store_song(s, song_uid.as_str()) {
                                Ok(_) => {
                                        if let Err(e) = tx.send(ThreadData::DownloadFinishedSignal)
                                        {
                                                println!("Could not download-finish-signal: {}", e);
                                        }
                                }
                                Err(e) => {
                                        if let Err(e) = tx.send(ThreadData::Notification(
                                                NotificationType::Error(
                                                        "Could not store song to local file: "
                                                                .to_string()
                                                                + &e.to_string(),
                                                ),
                                        )) {
                                                println!("Could not send message: {}", e);
                                        }
                                }
                        },
                        Err(e) => {
                                if let Err(e) =
                                        tx.send(ThreadData::Notification(NotificationType::Error(
                                                "Something went wrong downloading the tab: "
                                                        .to_string()
                                                        + &e,
                                        )))
                                {
                                        println!("Could not send message: {}", e);
                                }
                        }
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
                                                match load_song(song_uid) {
                                                        Ok(s) => self
                                                                .song_to_ui(s, song_uid.to_owned()),
                                                        Err(e) => self.show_info(
                                                                NotificationType::Error(
                                                                        "Could not load song file: "
                                                                                .to_string()
                                                                                + &e.to_string(),
                                                                ),
                                                        ),
                                                }
                                        } else if ask {
                                                self.screen = Screen::Search;
                                                self.search_value = search_query;
                                                self.search_tabs();
                                        }
                                }
                        }
                        Err(e) => self.show_info(NotificationType::Fatal(
                                "Could not get tab path: ".to_string() + &e.to_string(),
                        )),
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
        let selected = if let Value::String(t) = config_object.get("theme") {
                t
        } else {
                "".into()
        };
        for theme in Theme::ALL {
                if selected == theme.to_string().as_str() {
                        return theme.to_owned();
                }
        }
        Theme::Dark
}

fn main() {
        let _ = iced::application(
                "Coral-Chords",
                ApplicationState::update,
                ApplicationState::view,
        )
        .centered()
        .theme(ApplicationState::theme)
        .subscription(ApplicationState::subscription)
        .run();
}
