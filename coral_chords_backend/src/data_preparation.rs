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

use crate::types_and_constants::*;
use ureq::{get, Error as ReqError};
use html_escape::{decode_html_entities};
use regex::Regex;

pub fn get_song_data_from_url(url: &str) -> Result<SongData, CoralChordsError> {
        let raw_html: String;
        match get_raw_html(url) {
                Ok(s) => raw_html = s,
                Err(e) => match try_to_fix_url(e, url) {
                        Ok(s) => raw_html = s,
                        Err(e) => return Err(CoralChordsError::ReqError(e.to_string())),
                },
        }
        match get_type(&raw_html) {
                Ok(d) => {
                        extratc_data(&raw_html, d)
                },
                Err(e) => Err(e)
        }
}

fn unescape_string(string: &str) -> String{
        decode_html_entities(string).to_string().replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\r", "\r")
                .replace("\\n", "\n")
}

fn extratc_data(raw_html: &str, data_type: CoralChordsDataType) -> Result<SongData, CoralChordsError> {
        let string_parts: Vec<&str> = raw_html.split(END_OF_CHORDS_DELIM).collect();
        let raw_data: &str = string_parts[0].split(START_OF_CHORDS_DELIM).collect::<Vec<&str>>()[1];
        let formatted_string_lines = unescape_string(raw_data);
        match data_type {
                CoralChordsDataType::Error(e) => return Err(e),
                CoralChordsDataType::Drums => {
                        let clean_lines: Vec<DataLine> = clean_and_evaluate(formatted_string_lines.lines());
                        return Ok(SongData { data_type: data_type, 
                                lines: clean_lines, 
                                metadata: SongMetadata::default(), 
                                basic_data: BasicSongData::default() });
                }
                _ => (),
        }

        let mut clean_lines: Vec<DataLine> = clean_and_evaluate(formatted_string_lines.lines());

        let regex = Regex::new(DETAIL_REGEX).unwrap();
        let captures = regex.captures(raw_html);
        let song_metadata: SongMetadata;
        if captures.is_some() {
                let captures = captures.unwrap();
                println!("Capo: {}, Tonality: {}, Tuning Name: {}, Tuning: {}", &captures[1], &captures[2], &captures[3], &captures[4]);
                song_metadata = SongMetadata { capo: captures[1].to_string(), 
                        tonality: captures[2].to_string(), 
                        tuning_name: captures[3].to_string(), 
                        tuning: captures[4].to_string() };
                for i in 1..5 {
                        if !captures[i].is_empty() {
                                match i {
                                        1 => clean_lines.push(DataLine { line_type: DataLineType::Capo, text_data: String::from(&captures[i]) }),
                                        2 => clean_lines.push(DataLine { line_type: DataLineType::Tonality, text_data: String::from(&captures[i]) }),
                                        3 => clean_lines.push(DataLine { line_type: DataLineType::TuningName, text_data: String::from(&captures[i]) }),
                                        4 => clean_lines.push(DataLine { line_type: DataLineType::Tuning, text_data: String::from(&captures[i]) }),
                                        _ => (),
                                }
                        }
                }
        } else {
                song_metadata = SongMetadata::default();
        }
        return Ok(SongData { data_type: data_type, lines: clean_lines, metadata: song_metadata, basic_data: BasicSongData::default()})
}

fn clean_and_evaluate(lines: std::str::Lines<'_>) -> Vec<DataLine> {
        let mut clean_lines: Vec<DataLine> = Vec::new();
        for line in lines {
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
                clean_lines.push(DataLine {line_type: line_type, text_data: clean_line});
        }
        clean_lines
        
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
        use super::*;

        #[test]
        fn type_detection() {
                let type_detection_checks: Vec<(CoralChordsDataType, &str)> = vec![(CoralChordsDataType::Chords, "https://tabs.ultimate-guitar.com/tab/queen/dont-stop-me-now-chords-519549"),
                        (CoralChordsDataType::Chords, "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741"),
                        (CoralChordsDataType::Bass, "https://tabs.ultimate-guitar.com/tab/bloc-party/this-modern-love-bass-180218"),
                        (CoralChordsDataType::Tab, "https://tabs.ultimate-guitar.com/tab/led-zeppelin/stairway-to-heaven-tabs-9488"),
                        (CoralChordsDataType::Ukulele, "https://tabs.ultimate-guitar.com/tab/olli-schulz/wenn-es-gut-ist-ukulele-1381967"),
                        (CoralChordsDataType::Drums, "https://tabs.ultimate-guitar.com/tab/phil-collins/in-the-air-tonight-drums-880599"),
                        (CoralChordsDataType::Bass, "https://tabs.ultimate-guitar.com/tab/pink-floyd/empty-spaces-bass-147995")];
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
                        "https://tabs.ultimate-guitar.com/tab/phil-collins/in-the-air-tonight-drums-880599",
                        "https://tabs.ultimate-guitar.com/tab/blink-182/feeling-this-bass-104175",
                        "https://tabs.ultimate-guitar.com/tab/pink-floyd/empty-spaces-bass-147995"];
                for valid_page_url in valid_page_urls {
                        println!("Testing valid url: {}", valid_page_url);
                        assert!(!matches!(get_song_data_from_url(valid_page_url), Err(CoralChordsError::InvalidPageType)));
                }

                let invalid_page_urls = vec!["https://tabs.ultimate-guitar.com/tab/refused/i-wanna-watch-the-world-burn-guitar-pro-5868920", 
                        "https://tabs.ultimate-guitar.com/tab/refused/rather-be-dead-power-595658", 
                        "https://tabs.ultimate-guitar.com/tab/the-beatles/let-it-be-video-781202",
                        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=RDdQw4w9WgXcQ&start_radio=1"];
                for invalid_page_url in invalid_page_urls {
                        println!("Testing invalid url: {}", invalid_page_url);
                        assert!(matches!(get_song_data_from_url(invalid_page_url), Err(CoralChordsError::InvalidPageType)));
                }
        }

        #[test]
        fn get_meta_data() {
                let url_meta_data_sets: Vec<(SongMetadata, &str)> = vec![(SongMetadata { capo: String::from("3"), 
                                tonality: String::from(""), 
                                tuning_name: String::from("G C E A"), 
                                tuning: String::from("G C E A") }, "https://tabs.ultimate-guitar.com/tab/olli-schulz/wenn-es-gut-ist-ukulele-1381967"),
                        (SongMetadata::default(), "https://tabs.ultimate-guitar.com/tab/pink-floyd/empty-spaces-bass-147995"),
                        (SongMetadata::default(), "https://tabs.ultimate-guitar.com/tab/phil-collins/in-the-air-tonight-drums-880599"),
                        (SongMetadata { capo: String::from("1"), 
                                tonality: String::from(""), 
                                tuning_name: String::from("Standard"), 
                                tuning: String::from("E A D G B E") }, "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741"),
                        (SongMetadata { capo: String::from(""), 
                                tonality: String::from("F"), 
                                tuning_name: String::from("Standard"), 
                                tuning: String::from("E A D G B E") }, "https://tabs.ultimate-guitar.com/tab/queen/dont-stop-me-now-chords-519549"),];
                for url_meta_data_set in url_meta_data_sets {
                        println!("Testing url: {}", stringify!(get_type(&get_raw_html(url_meta_data_set.1).unwrap()).unwrap()));
                        match extratc_data(&get_raw_html(url_meta_data_set.1).unwrap(), CoralChordsDataType::Chords) {
                                Ok(d) => assert_eq!(d.metadata, url_meta_data_set.0),
                                Err(e) => panic!("Something went wrong!... [insert useful error message here]"),
                        }
                }
        }
}
