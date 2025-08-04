// UG-Tab-Scraper - A rust api for downloading UG tabs
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

pub mod tab_scraper;

pub mod types_and_constants {
        pub const END_OF_CHORDS_DELIM: &str = "&quot;,&quot;revision_id&quot;:";
        pub const START_OF_CHORDS_DELIM: &str = "&quot;:{&quot;wiki_tab&quot;:{&quot;content&quot;:&quot;";
        pub const HTML_BLACKLIST: [&str; 1] = ["&quot;type&quot;:&quot;Video&quot;"];
        pub const DETAIL_REGEX: &str = r"&quot;adsupp_binary_blocked&quot;:null,&quot;meta&quot;:\{[&quot;capo&quot;:]*(\d*)[,]*&quot;[tonality&quot;:&quot;]*(\w*)[&quot;,&quot;]*tuning&quot;:\{&quot;name&quot;:&quot;([^:]*)&quot;,&quot;value&quot;:&quot;([^:]*)&quot;,";
        pub const TYPE_REGEX: &str = r"tab&quot;:\{&quot;id&quot;:\d+,&quot;song_id&quot;:\d+,&quot;song_name&quot;:&quot;[^:]+&quot;,&quot;artist_id&quot;:\d+,&quot;artist_name&quot;:&quot;([^:]+)&quot;,&quot;type&quot;:&quot;([\w\s]+)&quot;,&quot;part&quot;:";

        #[derive(Debug, PartialEq)]
        pub enum CoralChordsError {
                InvalidPageType,
                UnknownType,
                ReqError(String),
        }

        #[derive(Debug, PartialEq)]
        pub enum CoralChordsDataType {
                Chords,
                Tab,
                Ukulele,
                Bass,
                Drums,
                Error(CoralChordsError),
        }

        #[derive(Debug)]
        pub enum DataLineType {
                Chord,
                Lyric,
                Section,
                Title,
                Capo,
                Tuning,
                TuningName,
                Tonality,
        }

        #[derive(Debug)]
        pub struct DataLine {
                pub line_type: DataLineType,
                pub text_data: String,
        }

        #[derive(Debug)]
        pub struct SongData {
                pub data_type: CoralChordsDataType,
                pub lines: Vec<DataLine>,
                pub metadata: SongMetadata,
                pub basic_data: BasicSongData,
        }

        #[derive(Debug, PartialEq, std::default::Default)]
        pub struct BasicSongData {
                pub title: String,
                pub artist: String,
                pub song_id: String, 
                pub tab_link: String,
        }

        #[derive(Debug, PartialEq, std::default::Default)]
        pub struct SongMetadata {
                pub capo: String,
                pub tonality: String,
                pub tuning_name: String,
                pub tuning: String,
        }
}