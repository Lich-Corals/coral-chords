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

use crate::SearchState;
use crate::{ApplicationState, Message};
use iced::color;
use iced::widget::{button, column, container, row, scrollable, text, Column, Space};
use iced::Alignment::Center;
use ug_scraper::types::{SearchResult, SUPPORTED_DOWNLOAD_TYPES};

pub fn build_search_page<'a>(application_state: &ApplicationState) -> Column<'a, Message> {
        match &application_state.search_state {
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
                        if application_state.search_filter.is_some() {
                                let mut new_results = vec![];
                                for result in sorted_results {
                                        if result.basic_data.data_type
                                                == application_state.search_filter.unwrap()
                                        {
                                                new_results.push(result.clone());
                                        }
                                }
                                sorted_results = new_results;
                        }
                        if sorted_results.is_empty() {
                                column![text("No search results!").color(color!(0xe64553))]
                                        .padding(10)
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
                                        artist_vec
                                                .sort_by_key(|s| (s.rating_value * -100.0) as i32);
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
                                                rating = format!(
                                                        "{:.1$}/5",
                                                        search_result.rating_value, 1
                                                );
                                                rating_count_string = format!("x{rating_count:?}");
                                        } else {
                                                rating = "?".into();
                                                rating_count_string = "".into();
                                        }
                                        let row_height = 40;
                                        if SUPPORTED_DOWNLOAD_TYPES
                                                .contains(&search_result.basic_data.data_type)
                                                || !application_state.only_downloadable_results
                                        {
                                                if SUPPORTED_DOWNLOAD_TYPES.contains(
                                                        &search_result.basic_data.data_type,
                                                ) {
                                                        title_column = title_column.push(row![
                                                                button("Download").on_press(
                                                                        Message::DownloadTab(url)
                                                                ),
                                                                Space::new(10, 0),
                                                                text(title),
                                                                Space::new(30, 0),
                                                        ]
                                                        .height(row_height)
                                                        .align_y(Center));
                                                } else {
                                                        title_column = title_column.push(row![
                                                                button("Download")
                                                                        .on_press(Message::Update)
                                                                        .style(button::secondary),
                                                                Space::new(10, 0),
                                                                text(title),
                                                                Space::new(30, 0),
                                                        ]
                                                        .height(row_height)
                                                        .align_y(Center));
                                                }
                                                artist_column = artist_column.push(row![
                                                        text("by "),
                                                        text(artist).color(color!(0xea76cb)),
                                                        Space::new(30, 0),
                                                ]
                                                .height(row_height)
                                                .align_y(Center));
                                                rating_column = rating_column.push(row![
                                                        text("Rating: "),
                                                        text(rating).color(color!(0xdd7878)),
                                                        Space::new(10, 0),
                                                ]
                                                .height(row_height)
                                                .align_y(Center));
                                                rating_count_column =
                                                        rating_count_column.push(row![
                                                                text(rating_count_string)
                                                                        .color(color!(0xdd7878)),
                                                                Space::new(30, 0),
                                                        ]
                                                        .height(row_height)
                                                        .align_y(Center));
                                                type_column = type_column.push(row![
                                                        text(format!(
                                                                "{}",
                                                                &search_result.basic_data.data_type
                                                        ))
                                                        .color(color!(0x179299)),
                                                        Space::new(30, 0),
                                                ]
                                                .height(row_height)
                                                .align_y(Center));
                                        }
                                }
                                column![container(
                                        scrollable(row![
                                                title_column,
                                                artist_column,
                                                rating_column,
                                                rating_count_column,
                                                type_column,
                                        ])
                                        .spacing(10)
                                )
                                .padding(10)]
                        }
                }
                SearchState::Searching => {
                        column![text("Searching...")].padding(application_state.main_padding)
                }
                _ => Column::new(),
        }
        .align_x(Center)
        .width(application_state.size.width - 2.0 * application_state.main_padding)
}
