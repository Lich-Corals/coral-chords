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

use crate::{ApplicationState, Message};
use iced::widget::{
        button, checkbox, column, combo_box, rich_text, row, scrollable, slider, span, text,
        text_input, Column, Space,
};
use iced::Alignment::Center;
use iced::{color, font, Font};

pub fn build_settings_page<'a>(application_state: &'a ApplicationState) -> Column<'a, Message> {
        let mut col = column![column![
                column![
                        text("Application").size(25),
                        row![
                                text("Theme"),
                                Space::new(10, 0),
                                combo_box(
                                        &application_state.themes,
                                        "Theme",
                                        application_state.selected_theme.as_ref(),
                                        Message::ApplyTheme
                                )
                                .width(300),
                        ]
                        .align_y(Center),
                        row![checkbox(
                                "Notify about updates",
                                application_state.notify_about_updates
                        )
                        .on_toggle(Message::ToggleUpdateNotification),]
                        .align_y(Center),
                ]
                .spacing(20)
                .align_x(Center),
                column![
                        text("Search").size(25),
                        row![checkbox(
                                "Only show downloadable search results",
                                application_state.only_downloadable_results
                        )
                        .on_toggle(Message::SetDownloadableOnly),]
                        .align_y(Center),
                ]
                .spacing(20)
                .align_x(Center),
                column![
                        text("Tabs").size(25),
                        row![
                                text(format!("Text size: {:02}", application_state.tab_text_size)),
                                Space::new(10, 0),
                                slider(
                                        5..=99,
                                        application_state.tab_text_size,
                                        Message::SetTabLineHeight
                                )
                                .width(300),
                        ]
                        .align_y(Center),
                        row![
                                text("Metadata colour:"),
                                Space::new(10, 0),
                                text_input("#209fb5", &application_state.metadata_colour_text)
                                        .on_input(Message::RecolourMetadata)
                                        .width(300),
                        ]
                        .align_y(Center),
                        row![
                                text("Header colour:"),
                                Space::new(10, 0),
                                text_input("#dc8a78", &application_state.header_colour_text)
                                        .on_input(Message::HeaderColourChange)
                                        .width(300),
                        ]
                        .align_y(Center),
                        row![
                                text("Chord colour:"),
                                Space::new(10, 0),
                                text_input("#fe640b", &application_state.chord_colour_text)
                                        .on_input(Message::ChordColourChange)
                                        .width(300),
                        ]
                        .align_y(Center),
                        column![
                                Space::new(0, 20),
                                text("Some matadata\n ".to_string())
                                        .size(application_state.tab_text_size as f32)
                                        .font(Font::MONOSPACE)
                                        .color(application_state.metadata_colour),
                                rich_text([span("[Section header]")
                                        .font(Font {
                                                style: font::Style::Italic,
                                                ..Font::MONOSPACE
                                        })
                                        .size(application_state.tab_text_size as f32)
                                        .color(application_state.header_colour),]),
                                text("A         C     A       B".to_string())
                                        .size(application_state.tab_text_size as f32)
                                        .font(Font::MONOSPACE)
                                        .color(application_state.chord_colour),
                                text("Tabs will look like this.".to_string())
                                        .size(application_state.tab_text_size as f32)
                                        .font(Font::MONOSPACE),
                        ],
                ]
                .spacing(20)
                .align_x(Center),
        ]
        .padding(10)
        .spacing(20)
        .align_x(Center)];

        if application_state.show_advanced_settings {
                col = column![
                        col,
                        Space::new(0, 20),
                        column![
                                text("Advanced").size(25),
                                row![
                                        checkbox(
                                                "Log played songs locally",
                                                application_state.log_played_songs
                                        )
                                        .on_toggle(Message::TogglePlayedLog),
                                        Space::new(10, 0),
                                        button("Show statistics")
                                                .on_press(Message::StatisticsPage)
                                                .padding([4, 10])
                                ]
                                .align_y(Center),
                                row![checkbox(
                                        "Remind me to replace my strings",
                                        application_state.renew_strings_interval > 0
                                )
                                .on_toggle(Message::ToggleStringRenewalReminder),]
                                .align_y(Center),
                                row![
                                        text(format!(
                                                "String reminder interval: {:02}mo, {:02}d",
                                                application_state.renew_strings_interval.abs()
                                                        / (60 * 60 * 24 * 31),
                                                (application_state.renew_strings_interval.abs()
                                                        % (60 * 60 * 24 * 31))
                                                        / (60 * 60 * 24)
                                        )),
                                        Space::new(10, 0),
                                        slider(
                                                1..=12 * 31,
                                                (application_state.renew_strings_interval.abs()
                                                        / (60 * 60 * 24))
                                                        as u32,
                                                Message::UpdateStringRenewalInterval
                                        )
                                        .width(300),
                                ]
                                .align_y(Center),
                                row![checkbox(
                                        "Auto-clean search queries",
                                        application_state.clean_queries
                                )
                                .on_toggle(Message::CleanQueries),]
                                .align_y(Center),
                                row![
                                        text(format!(
                                                "Search depth: {:02}",
                                                application_state.search_depth
                                        )),
                                        Space::new(10, 0),
                                        slider(
                                                1..=64,
                                                application_state.search_depth,
                                                Message::ApplySearchDepth
                                        )
                                        .width(300),
                                ]
                                .align_y(Center),
                                row![checkbox(
                                        "Remove lines before first chords",
                                        application_state.remove_first_lines
                                )
                                .on_toggle(Message::RemoveFirstLines),]
                                .align_y(Center),
                                row![checkbox(
                                        "Remove empty lines from tabs",
                                        application_state.remove_empty_lines
                                )
                                .on_toggle(Message::RemoveEmptyLines),]
                                .align_y(Center),
                        ]
                        .align_x(Center)
                        .spacing(20),
                ]
                .align_x(Center);
        }

        col = column![scrollable(
                column![
                        col,
                        Space::new(0, 20),
                        column![
                                Space::new(0, 20),
                                row![
                                        text("View "),
                                        rich_text([span("the repository")
                                                .link(Message::OpenReadme)
                                                .underline(true),]),
                                        text(" for more information about the settings"),
                                ],
                                text("and how to use this program."),
                                Space::new(0, 30),
                                row![
                                        rich_text([span("Created with ")
                                                .color(color!(0x696969))
                                                .size(12),]),
                                        rich_text([span("❤️").color(color!(0x933030)).size(12),]),
                                        rich_text([span(" by Linus Tibert (Lich-Corals)")
                                                .color(color!(0x696969))
                                                .size(12),]),
                                ],
                                Space::new(0, 5),
                                rich_text([span(format!(
                                        "Coral-Chords v{}  Copyright (C) 2025  Linus Tibert",
                                        env!("CARGO_PKG_VERSION")
                                ))
                                .color(color!(0x696969))
                                .size(11),]),
                                rich_text([span("GNU Affero General Public Licence v3")
                                        .color(color!(0x696969))
                                        .link(Message::OpenLicence)
                                        .size(11)
                                        .underline(true),]),
                                text("").size(12),
                                text("").size(12)
                        ]
                        .align_x(Center),
                ]
                .width(application_state.size.width - 2.0 * application_state.main_padding)
                .align_x(Center)
        )];

        col
}
