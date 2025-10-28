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

use crate::{ApplicationState, Message, Screen, SearchState};
use iced::widget::{button, combo_box, row, text_input, toggler, Row, Space};
use iced::Alignment::Center;

pub fn build_controls<'a>(application_state: &'a ApplicationState) -> Row<'a, Message> {
        let bar_button = |label| button(row![label].align_y(Center)).padding([4, 12]);
        match application_state.screen {
                Screen::Statistics => row![
                        bar_button("Tab")
                                .on_press(Message::TabsPage)
                                .style(button::secondary),
                        bar_button("Search")
                                .on_press(Message::SearchPage)
                                .style(button::secondary),
                        bar_button("Settings")
                                .on_press(Message::SettingsPage)
                                .style(button::secondary),
                        bar_button("Statistics").on_press(Message::StatisticsPage),
                        Space::new(100, 0),
                ],
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
                        toggler(application_state.playing)
                                .label("Play")
                                .on_toggle(Message::PlayingToggled)
                ]
                .align_y(Center),
                Screen::Search => {
                        let value = &application_state.search_value;
                        let mut search_bar = text_input("Search a tab...", value)
                                .on_input(Message::UpdateSearchBar)
                                .on_submit(Message::SearchTabs)
                                .width(300);
                        if application_state.search_state == SearchState::Searching {
                                search_bar = text_input("Searching...", value).width(300);
                        }
                        row![
                                bar_button("Tab")
                                        .on_press(Message::TabsPage)
                                        .style(button::secondary),
                                bar_button("Search").on_press(Message::SearchPage),
                                bar_button("Settings")
                                        .on_press(Message::SettingsPage)
                                        .style(button::secondary),
                                Space::new(100, 0),
                                search_bar,
                                bar_button("Go!").on_press(Message::SearchTabs),
                                Space::new(10, 0),
                                bar_button("Clear").on_press(Message::ClearSearch),
                                Space::new(40, 0),
                                combo_box(
                                        &application_state.tab_types,
                                        "Filter",
                                        application_state.search_filter.as_ref(),
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
                        bar_button("Statistics")
                                .on_press(Message::StatisticsPage)
                                .style(button::secondary),
                        Space::new(100, 0),
                        toggler(application_state.show_advanced_settings)
                                .label("Advanced Settings")
                                .on_toggle(Message::SetAdvancedSettings),
                ]
                .align_y(Center),
        }
        .padding(10)
        .spacing(2)
}
