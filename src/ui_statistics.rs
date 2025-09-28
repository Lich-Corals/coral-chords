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

use crate::backend::system::get_song_log;
use crate::{ApplicationState, Message};
use iced::widget::{
        checkbox, column, combo_box, progress_bar, rich_text, row, scrollable, slider, span, text,
        text_input, Column, Space,
};
use iced::Alignment::Center;

struct SongStat {
        name: String,
        artist: String,
        count: u16,
}

pub fn build_statistics_page<'a>(application_state: &'a ApplicationState) -> Column<'a, Message> {
        let col: Column<'a, Message>;
        if !application_state.log_played_songs {
                col = column![text("The setting 'Log played songs locally' needs to be enabled to get statistics!")];
        } else {
                let song_log = get_song_log();
                if song_log.1.is_none() {
                        let song_log = song_log.0;
                        for song in song_log {}
                        col = column![scrollable(column!(progress_bar(0.0..=100.0, 50.0)))];
                } else {
                        col = column![text(format!(
                                "Could not load statistics: {}",
                                song_log.1.unwrap()
                        ))];
                }
        }
        col.padding(20)
}
