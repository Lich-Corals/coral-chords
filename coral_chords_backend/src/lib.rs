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

const END_OF_CHORDS_DELIM: &str = "&quot;,&quot;revision_id&quot;:";
const START_OF_CHORDS_DELIM: &str = "&quot;:{&quot;wiki_tab&quot;:{&quot;content&quot;:&quot;";

const HTML_BLACKLIST: [&str; 1] = ["&quot;type&quot;:&quot;Video&quot;"];

pub enum CoralError {
        InvalidPageType,
        ReqError(String),
}

pub enum CoralChordsData {
        Chords(String),
        Tab(String),
        Error(CoralError),
}

pub fn get_song_data_from_url(url: &str) -> CoralChordsData {
        let raw_html: String;
        match get_raw_html(url) {
                Ok(s) => raw_html = s,
                Err(e) => match try_to_fix_url(e, url) {
                        Ok(s) => raw_html = s,
                        Err(e) => return CoralChordsData::Error(CoralError::ReqError(e.to_string())),
                },
        }
        match get_type(&raw_html) {
                Ok(d) => {
                        extratc_data(&raw_html, d)
                },
                Err(e) => CoralChordsData::Error(e)
        }
}

fn unescape_string(string: &str) -> String{
        decode_html_entities(string).to_string().replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\r", "\r")
                .replace("\\n", "\n")
}

fn extratc_data(raw_html: &str, data_type: CoralChordsData) -> CoralChordsData {
        let string_parts: Vec<&str> = raw_html.split(END_OF_CHORDS_DELIM).collect();
        let raw_data: &str = string_parts[0].split(START_OF_CHORDS_DELIM).collect::<Vec<&str>>()[1];
        let formatted_string: String = unescape_string(raw_data);
        CoralChordsData::Chords(formatted_string)
}

fn get_type(html: &str) -> Result<CoralChordsData, CoralError> {
        for item in HTML_BLACKLIST {
                if html.contains(item) {
                        return Err(CoralError::InvalidPageType)
                }
        }
        if !html.contains(START_OF_CHORDS_DELIM) || !html.contains(END_OF_CHORDS_DELIM) {
                return Err(CoralError::InvalidPageType)
        }
        Ok(CoralChordsData::Chords(String::default()))
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
        fn get_valid_page_song_data() {
                let valid_page_urls = vec!["https://tabs.ultimate-guitar.com/tab/queen/dont-stop-me-now-chords-519549",
                        "https://tabs.ultimate-guitar.com/tab/rick-astley/never-gonna-give-you-up-chords-521741",
                        "https://tabs.ultimate-guitar.com/tab/led-zeppelin/stairway-to-heaven-tabs-9488",
                        "https://tabs.ultimate-guitar.com/tab/olli-schulz/wenn-es-gut-ist-ukulele-1381967",
                        "https://tabs.ultimate-guitar.com/tab/bloc-party/this-modern-love-bass-180218",
                        "https://tabs.ultimate-guitar.com/tab/phil-collins/in-the-air-tonight-drums-880599"];
                for valid_page_url in valid_page_urls {
                        println!("Testing valid url: {}", valid_page_url);
                        assert!(matches!(get_song_data_from_url(valid_page_url), CoralChordsData::Chords(_)));
                }
        }

        #[test]
        fn get_invalid_page_song_data() {
                let invalid_page_urls = vec!["https://tabs.ultimate-guitar.com/tab/refused/i-wanna-watch-the-world-burn-guitar-pro-5868920", 
                        "https://tabs.ultimate-guitar.com/tab/refused/rather-be-dead-power-595658", 
                        "https://tabs.ultimate-guitar.com/tab/the-beatles/let-it-be-video-781202"];
                for invalid_page_url in invalid_page_urls {
                        println!("Testing invalid url: {}", invalid_page_url);
                        assert!(matches!(get_song_data_from_url(invalid_page_url), CoralChordsData::Error(CoralError::InvalidPageType)));
                }
        }
}
