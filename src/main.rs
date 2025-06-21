#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use fitgirl_ddl_lib::set_fg_cookies;
use itertools::Itertools as _;

use crate::model::Cookie;
use crate::ui::{Message, State};

mod model;
mod ui;

fn main() -> Result<(), iced::Error> {
    nyquest_preset::register();

    // run the app from main function
    iced::application("fitgirl-ddl", State::update, State::view)
        .centered()
        .window_size(iced::Size::new(800., 65.))
        .run_with(|| {
            (
                State::new(),
                iced::Task::perform(
                    async {
                        let Ok(Ok(cookies)) = tokio::fs::read("cookies.json")
                            .await
                            .map(|bytes| serde_json::from_slice::<Vec<Cookie>>(&bytes))
                        else {
                            return;
                        };

                        let _ = set_fg_cookies(
                            cookies
                                .iter()
                                .map(|Cookie { name, value }| format!("{name}={value}"))
                                .join("; "),
                        );
                    },
                    |_| Message::InitDone,
                ),
            )
        })
}
