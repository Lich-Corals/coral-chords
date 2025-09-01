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
use iced::widget::{column, rich_text, row, span, text, Column, Row};
use iced::{font, Font};
use ug_scraper::types::{DataSetType, DataType};

pub fn build_tabs_page<'a>(application_state: &ApplicationState) -> Column<'a, Message> {
        let mut main_row: Row<'_, Message> = row![];
        let mut new_column = column![];
        if let Some(d) = &application_state.current_tab.metadata {
                if let Some(d) = &d.capo {
                        new_column = new_column.push(row![text(format!("Capo: {}", d.clone()))
                                .color(application_state.metadata_colour)
                                .font(Font::MONOSPACE)
                                .size(application_state.tab_text_size as f32)]
                        .height(application_state.tab_text_size as f32));
                }
                if let Some(d) = &d.tuning {
                        new_column = new_column.push(row![text(format!("Tuning: {}", d.clone()))
                                .color(application_state.metadata_colour)
                                .font(Font::MONOSPACE)
                                .size(application_state.tab_text_size as f32)]
                        .height(application_state.tab_text_size as f32));
                }
                new_column = new_column.push(row![text(" ")
                        .font(Font::MONOSPACE)
                        .size(application_state.tab_text_size as f32)]
                .height(application_state.tab_text_size as f32));
        }
        let mut max_lines_per_column = 0.9
                * ((application_state.size.height
                        - application_state.bar_height
                        - 2.0 * application_state.main_padding)
                        / application_state.tab_text_size as f32);
        if application_state.current_tab.basic_data.data_type == DataSetType::Tab {
                max_lines_per_column -= 5.0; // Remove five lines to prevent overflow of tabs
        }
        let mut lines_on_column = 3; // Is at three for the first column to make room for potential metadata entries
        let mut first_chords_found = false;
        for line in &application_state.current_tab.lines {
                if line.line_type != DataType::Lyric {
                        first_chords_found = true;
                }
                if !application_state.remove_first_lines
                        || first_chords_found
                        || application_state.current_tab.basic_data.data_type == DataSetType::Tab
                {
                        lines_on_column += 1;
                        match line.line_type {
                                DataType::Chord => {
                                        new_column = new_column.push(row![text(line
                                                .text_data
                                                .clone())
                                        .color(application_state.chord_colour)
                                        .font(Font::MONOSPACE)
                                        .size(application_state.tab_text_size as f32)]
                                        .height(application_state.tab_text_size as f32));
                                }
                                DataType::Lyric => {
                                        if !application_state.remove_empty_lines
                                                || !line.text_data.is_empty()
                                        {
                                                new_column = new_column.push(row![text(line
                                                        .text_data
                                                        .clone())
                                                .font(Font::MONOSPACE)
                                                .size(application_state.tab_text_size as f32)]
                                                .height(application_state.tab_text_size as f32));
                                        }
                                }
                                DataType::SectionTitle => {
                                        new_column = new_column.push(row![rich_text([span(line
                                                .text_data
                                                .clone())
                                        .font(Font {
                                                style: font::Style::Italic,
                                                ..Font::MONOSPACE
                                        })
                                        .size(application_state.tab_text_size as f32)
                                        .color(application_state.header_colour),]),]
                                        .height(application_state.tab_text_size as f32));
                                }
                        }
                        if (lines_on_column >= max_lines_per_column as u16)
                                && line.line_type == DataType::Lyric
                        {
                                if application_state.current_tab.basic_data.data_type
                                        == DataSetType::Tab
                                {
                                        println!("TAB: {}", line.text_data);
                                        let mut hyphens_in_line: f32 = 0.0;
                                        for character in line.text_data.chars() {
                                                if character == '-' {
                                                        hyphens_in_line += 1.0;
                                                }
                                        }
                                        println!("{}", hyphens_in_line);
                                        if hyphens_in_line / line.text_data.len() as f32 <= 0.3
                                                || line.text_data.len() < 5
                                        {
                                                // Only start a new
                                                // column if amount
                                                // of hyphens is <=
                                                // 30% of the line
                                                lines_on_column = 0;
                                                main_row = main_row.push(new_column);
                                                new_column = column![];
                                                println!("BREAK");
                                        }
                                } else {
                                        lines_on_column = 0;
                                        main_row = main_row.push(new_column);
                                        new_column = column![];
                                }
                        }
                }
        }
        main_row = main_row.push(new_column);
        column![main_row.spacing(20)]
}
