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
use iced::color;
use iced::widget::{column, rich_text, row, scrollable, span, text, Column, Space};
use iced::Alignment::Center;

pub fn build_welcome_page<'a>(application_state: &ApplicationState) -> Column<'a, Message> {
        column![scrollable(column![
                column![
                        text(format!("Welcome to Coral-Chords v{}!", env!("CARGO_PKG_VERSION")))
                                .size(25),
                                Space::new(0, 20),
                        row![
                                text("Thank you for trying it out!"),
                        ],
                        Space::new(0, 10),
                        row![
                                text("For usage and configuration instructions, take a look at "),
                                rich_text([span(
                                        "the GitHub repository.")
                                        .link(Message::OpenReadme)
                                        .underline(true),
                                ]),
                        ],
                        row![
                                text("If you experience any problems or have a suggestion about the application, please submit an issue on the GitHub page above."),
                        ],
                        row![
                                text!("And if you like this program, maybe think about "),
                                rich_text([span(
                                        "leaving a tip.")
                                        .link(Message::OpenCoffeePage)
                                        .underline(true),
                                ]),
                        ],
                        Space::new(0, 30),
                        row![
                                rich_text([span(
                                        "Created with ")
                                                .color(color!(0x696969))
                                                .size(12),
                                ]),
                                rich_text([span(
                                        "❤️")
                                                .color(color!(0x933030))
                                                .size(12),
                                ]),
                                rich_text([span(
                                        " by Linus Tibert (Lich-Corals)")
                                                .color(color!(0x696969))
                                                .size(12),
                                ]),
                        ],
                        Space::new(0, 5),
                        rich_text([span(
                                format!("Coral-Chords v{}  Copyright (C) 2025  Linus Tibert", env!("CARGO_PKG_VERSION")))
                                        .color(color!(0x696969))
                                        .size(11),
                        ]),
                        rich_text([span(
                                "GNU Affero General Public Licence v3")
                                        .color(color!(0x696969))
                                        .link(Message::OpenLicence)
                                        .size(11)
                                        .underline(true),
                        ]),
                        text("")
                                .size(12),
                        text("")
                                .size(12)
                ].align_x(Center),
        ].padding(10)
                .spacing(20)
                .align_x(Center)
                .width(application_state.size.width - 2.0 * application_state.main_padding))]
}
