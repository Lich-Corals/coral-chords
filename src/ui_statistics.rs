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
use iced::widget::{column, progress_bar, row, scrollable, text, Column, Space};
use iced::Alignment::Center;

/// Stores basic data about a single song while evaluation of the song log.
#[derive(Debug)]
struct SongStat {
        name: String,
        artist: String,
        count: u16,
}

impl Default for SongStat {
        fn default() -> Self {
                SongStat {
                        name: "undefined".to_string(),
                        artist: "Jax Doe".to_string(),
                        count: 0,
                }
        }
}

/// Stores basic data about a single artist while evaluation of the song log.
#[derive(Debug)]
struct ArtistStat {
        name: String,
        count: u16,
}

impl Default for ArtistStat {
        fn default() -> Self {
                ArtistStat {
                        name: "Jax Doe".to_string(),
                        count: 0,
                }
        }
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
                        let statistics_range_s: u64 = 8 * 24 * 60 * 60;
                        let day_s: u64 = 24 * 60 * 60;
                        let mut included_stats: Vec<SongStat> = vec![SongStat::default()];
                        let mut artist_stats: Vec<ArtistStat> = vec![ArtistStat::default()];
                        let mut durations_per_day: Vec<u64> = Vec::new();
                        for i in 0..(statistics_range_s / day_s) {
                                durations_per_day.insert(i as usize, 0);
                        }
                        // This is the time stamp at the end of the day:
                        // (current_time() / day_s) = (current_time_float() / day_s as f64).floor()
                        let current_time = (current_time() / day_s) * day_s + day_s;
                        let mut total_played_songs = 0;
                        for song in song_log {
                                if song.timestamp > current_time - statistics_range_s {
                                        total_played_songs += 1;
                                        let day_number: u64 =
                                                (current_time - song.timestamp) / day_s;
                                        durations_per_day[day_number as usize] += song.length_s;
                                        let mut song_in_stats: bool = false;
                                        for stat in included_stats.iter_mut() {
                                                if stat.name == song.name
                                                        && stat.artist == song.artist
                                                {
                                                        song_in_stats = true;
                                                        stat.count += 1;
                                                }
                                        }
                                        if !song_in_stats {
                                                included_stats.push(SongStat {
                                                        name: song.name.clone(),
                                                        artist: song.artist.clone(),
                                                        count: 1,
                                                });
                                        }
                                        let mut artist_in_stats: bool = false;
                                        for stat in artist_stats.iter_mut() {
                                                if stat.name == song.artist {
                                                        stat.count += 1;
                                                        artist_in_stats = true;
                                                }
                                        }
                                        if !artist_in_stats {
                                                artist_stats.push(ArtistStat {
                                                        name: song.artist,
                                                        count: 1,
                                                });
                                        }
                                }
                        }

                        temp_col = temp_col.push(text("Your statistics of the past week").size(25));
                        temp_col = temp_col.push(Space::new(0, 15));
                        temp_col = temp_col.push(text("Playtime per day").size(18));
                        temp_col = temp_col.push(Space::new(0, 10));
                        let max_mins = *durations_per_day.iter().max().unwrap() as f32 / 60.0;
                        let min_mins = *durations_per_day.iter().min().unwrap() as f32 / 60.0;
                        for (i, secs) in durations_per_day.iter().enumerate() {
                                temp_col = temp_col.push(row![
                                        text(format!("{} day(s) ago: {:04}min.", i, secs / 60)),
                                        Space::new(10, 0),
                                        progress_bar(
                                                min_mins..=(max_mins + max_mins / 8.0),
                                                *secs as f32 / 60.0
                                        ),
                                ]
                                .align_y(Center));
                        }
                        let mut favourite_song = &SongStat::default();
                        for stat in &included_stats {
                                if stat.count > favourite_song.count {
                                        favourite_song = stat;
                                }
                        }
                        let mut favourite_artist = &ArtistStat::default();
                        for stat in &artist_stats {
                                if stat.count > favourite_artist.count {
                                        favourite_artist = stat;
                                }
                        }

                        temp_col = temp_col.push(Space::new(0, 15));
                        temp_col = temp_col.push(text(format!(
                                "Different songs played: {}",
                                included_stats.len() - 1
                        ))
                        .size(18));
                        temp_col = temp_col
                                .push(text(format!("by {} artist(s)", artist_stats.len() - 1)));
                        temp_col = temp_col.push(Space::new(0, 15));
                        temp_col = temp_col.push(text(format!(
                                "Songs played in total: {}",
                                total_played_songs
                        ))
                        .size(18));
                        temp_col = temp_col.push(Space::new(0, 15));
                        temp_col = temp_col.push(text(format!(
                                "Favourite song: {}",
                                favourite_song.name
                        ))
                        .size(18));
                        temp_col = temp_col.push(text(format!(
                                "by {}; played {} time(s)",
                                favourite_song.artist, favourite_song.count
                        )));
                        temp_col = temp_col.push(Space::new(0, 15));
                        temp_col = temp_col.push(text(format!(
                                "Favourite artist: {}",
                                favourite_artist.name
                        ))
                        .size(18));
                        temp_col = temp_col.push(text(format!(
                                "you have played this artist {} time(s)",
                                favourite_artist.count
                        )));
                        col = column![scrollable(temp_col)];
                } else {
                        col = column![text(format!(
                                "Could not load statistics: {}",
                                song_log.1.unwrap()
                        ))];
                }
        }
        col.padding(20)
}
