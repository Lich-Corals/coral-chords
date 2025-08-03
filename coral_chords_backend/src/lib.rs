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

use ureq::{get, Error as ReqError};
use html_escape::{decode_html_entities};
use regex::Regex;

const END_OF_CHORDS_DELIM: &str = "&quot;,&quot;revision_id&quot;:";
const START_OF_CHORDS_DELIM: &str = "&quot;:{&quot;wiki_tab&quot;:{&quot;content&quot;:&quot;";
const HTML_BLACKLIST: [&str; 1] = ["&quot;type&quot;:&quot;Video&quot;"];
const DETAIL_REGEX: &str = r"&quot;:\{&quot;capo&quot;:(\d*),&quot;[tonality&quot;:&quot;]*(\w*)[&quot;,&quot;]*tuning&quot;:\{&quot;name&quot;:&quot;([^:]*)&quot;,&quot;value&quot;:&quot;([^:]*)&quot;,";
const TYPE_REGEX: &str = r"tab&quot;:\{&quot;id&quot;:\d+,&quot;song_id&quot;:\d+,&quot;song_name&quot;:&quot;[^:]+&quot;,&quot;artist_id&quot;:\d+,&quot;artist_name&quot;:&quot;([^:]+)&quot;,&quot;type&quot;:&quot;([\w\s]+)&quot;,&quot;part&quot;:";

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
pub enum CoralChordsData {
        Data(SongData),
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
        line_type: DataLineType,
        text_data: String,
}

#[derive(Debug)]
pub struct SongData {
        data_type: CoralChordsDataType,
        lines: Vec<DataLine>,
}

pub fn get_song_data_from_url(url: &str) -> CoralChordsData {
        let raw_html: String;
        match get_raw_html(url) {
                Ok(s) => raw_html = s,
                Err(e) => match try_to_fix_url(e, url) {
                        Ok(s) => raw_html = s,
                        Err(e) => return CoralChordsData::Error(CoralChordsError::ReqError(e.to_string())),
                },
        }
        match get_type(&raw_html) {
                Ok(d) => {
                        extratc_data(&raw_html, d)
                },
                Err(e) => CoralChordsData::Error(e)
        }
}

pub fn store_song(song_data: SongData) -> Result<bool, CoralChordsError> {
        todo!("store song")
}

fn unescape_string(string: &str) -> String{
        decode_html_entities(string).to_string().replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\r", "\r")
                .replace("\\n", "\n")
}

fn extratc_data(raw_html: &str, data_type: CoralChordsDataType) -> CoralChordsData {
        let string_parts: Vec<&str> = raw_html.split(END_OF_CHORDS_DELIM).collect();
        let raw_data: &str = string_parts[0].split(START_OF_CHORDS_DELIM).collect::<Vec<&str>>()[1];
        let formatted_string_lines = unescape_string(raw_data);
        match data_type {
                CoralChordsDataType::Error(e) => return CoralChordsData::Error(e),
                _ => (),
        }

        let mut clean_lines: Vec<DataLine> = Vec::new();
        for line in formatted_string_lines.lines() {
                clean_lines.push(clean_and_evaluate(line));
        }

        todo!("extract and return data");
}

fn clean_and_evaluate(line: &str) -> DataLine {
        let mut line_type: DataLineType = DataLineType::Lyric;
        if line.contains("[ch]") {
                line_type = DataLineType::Chord;
        }
        let mut clean_line: String = String::from(line);
        for key in ["[ch]", "[/ch]", "[tab]", "[/tab]"] {
                clean_line = clean_line.replace(key, "")
        }
        if clean_line.contains("[") && clean_line.contains("]") {
                line_type = DataLineType::Section;
        }
        DataLine {line_type: line_type, text_data: clean_line}
}

fn get_type(html: &str) -> Result<CoralChordsDataType, CoralChordsError> {
        for item in HTML_BLACKLIST {
                if html.contains(item) {
                        return Err(CoralChordsError::InvalidPageType)
                }
        }
        if !html.contains(START_OF_CHORDS_DELIM) || !html.contains(END_OF_CHORDS_DELIM) {
                return Err(CoralChordsError::InvalidPageType)
        }
        let regex = Regex::new(TYPE_REGEX).unwrap();
        let captures = regex.captures(html).unwrap();
        
        match &captures[2] {
                "Chords" => Ok(CoralChordsDataType::Chords),
                "Tabs" => Ok(CoralChordsDataType::Tab),
                "Bass Tabs" => Ok(CoralChordsDataType::Bass),
                "Ukulele Chords" => Ok(CoralChordsDataType::Ukulele),
                "Drum Tabs" => Ok(CoralChordsDataType::Drums),
                _ => Err(CoralChordsError::UnknownType)
        }
}

fn try_to_fix_url(error: ReqError, url: &str) -> Result<String, ReqError> {
        match error {
                ReqError::BadUri(_e) => return get_raw_html(&("https://".to_owned() + url)),
                _ => return Err(ReqError::BadUri(String::from(url)))
        }
}

fn get_raw_html(url: &str) -> Result<String, ReqError> {
        let mut response =  get(url).call()?;
        let raw_html = response.body_mut().read_to_string()?;
        Ok(raw_html)
}

#[cfg(test)]
mod tests {
        use std::fmt::Result;

        use super::*;

        #[test]
        fn type_detection() {
                let type_detection_checks: Vec<(CoralChordsDataType, &str)> = vec![(CoralChordsDataType::Chords, "https://tabs.ultimate-guitar.com/tab/queen/dont-stop-me-now-chords-519549"),
                        (CoralChordsDataType::Chords, "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741"),
                        (CoralChordsDataType::Bass, "https://tabs.ultimate-guitar.com/tab/bloc-party/this-modern-love-bass-180218"),
                        (CoralChordsDataType::Tab, "https://tabs.ultimate-guitar.com/tab/led-zeppelin/stairway-to-heaven-tabs-9488"),
                        (CoralChordsDataType::Ukulele, "https://tabs.ultimate-guitar.com/tab/olli-schulz/wenn-es-gut-ist-ukulele-1381967"),
                        (CoralChordsDataType::Drums, "https://tabs.ultimate-guitar.com/tab/phil-collins/in-the-air-tonight-drums-880599")];
                for check in type_detection_checks {
                        println!("Testing url: {}", stringify!(get_type(&get_raw_html(check.1).unwrap()).unwrap()));
                        assert_eq!(get_type(&get_raw_html(check.1).unwrap()).unwrap(), check.0);
                }
        }

        #[test]
        fn validate_page_contents() {
                let valid_page_urls = vec!["https://tabs.ultimate-guitar.com/tab/queen/dont-stop-me-now-chords-519549",
                        "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741",
                        "https://tabs.ultimate-guitar.com/tab/led-zeppelin/stairway-to-heaven-tabs-9488",
                        "https://tabs.ultimate-guitar.com/tab/olli-schulz/wenn-es-gut-ist-ukulele-1381967",
                        "https://tabs.ultimate-guitar.com/tab/bloc-party/this-modern-love-bass-180218",
                        "https://tabs.ultimate-guitar.com/tab/phil-collins/in-the-air-tonight-drums-880599"];
                for valid_page_url in valid_page_urls {
                        println!("Testing valid url: {}", valid_page_url);
                        assert!(matches!(get_song_data_from_url(valid_page_url), CoralChordsData::Data(_)));
                }

                let invalid_page_urls = vec!["https://tabs.ultimate-guitar.com/tab/refused/i-wanna-watch-the-world-burn-guitar-pro-5868920", 
                        "https://tabs.ultimate-guitar.com/tab/refused/rather-be-dead-power-595658", 
                        "https://tabs.ultimate-guitar.com/tab/the-beatles/let-it-be-video-781202"];
                for invalid_page_url in invalid_page_urls {
                        println!("Testing invalid url: {}", invalid_page_url);
                        assert!(matches!(get_song_data_from_url(invalid_page_url), CoralChordsData::Error(CoralChordsError::InvalidPageType)));
                }
        }
}
