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

use crate::backend::system::{current_time, get_song_log};
use crate::{ApplicationState, Message};
use iced::widget::{column, scrollable, text, Column};

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
                        let mut temp_col: Column<'a, Message> = Column::new();
                        let statistics_range_s: u64 = 7 * 24 * 60 * 60;
                        let day_s: u64 = 24 * 60 * 60;
                        let mut included_stats: Vec<SongStat> = Vec::new();
                        let mut durations_per_day: Vec<u64> = Vec::new();
                        for i in 0..(statistics_range_s / day_s) {
                                durations_per_day.insert(i as usize, 0);
                        }
                        let current_time = (current_time() / day_s) * day_s + day_s;
                        for song in song_log {
                                if song.timestamp > current_time - statistics_range_s {
                                        let day_number: u64 =
                                                (current_time - song.timestamp) / day_s;
                                        durations_per_day[day_number as usize] += song.length_s;
                                        for i in 0..included_stats.len() {
                                                let stat = &mut included_stats[i];
                                                if stat.name == song.name
                                                        && stat.artist == song.artist
                                                {
                                                        stat.count += 1;
                                                } else {
                                                        included_stats.push(SongStat {
                                                                name: song.name.clone(),
                                                                artist: song.artist.clone(),
                                                                count: 1,
                                                        });
                                                }
                                        }
                                }
                        }
                        col = column![scrollable(column!())];
                } else {
                        col = column![text(format!(
                                "Could not load statistics: {}",
                                song_log.1.unwrap()
                        ))];
                }
        }
        col.padding(20)
}
