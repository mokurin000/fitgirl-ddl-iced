use std::fmt::Write;

use fitgirl_ddl_lib::{
    errors::{ExtractError, ScrapeError},
    extract::{DDL, extract_ddl},
    scrape::GameInfo,
};
use futures_util::StreamExt as _;
use iced::{
    Size, Task,
    widget::{
        self,
        text_editor::{Action, Content},
    },
};
use itertools::Itertools as _;

struct Counter {
    editor_content: Content,
    init_done: bool,
}

#[derive(Debug, Clone)]
enum Message {
    InitDone,
    Scrape,
    Scraped(Result<GameInfo, ScrapeError>),
    Edit(Action),
    Extracted {
        path_part: String,
        direct_links: Vec<Result<DDL, ExtractError>>,
    },
}

// Implement our Counter
impl Counter {
    fn new() -> Self {
        // initialize the counter struct
        // with count value as 0.
        Self {
            init_done: false,
            editor_content: Content::new(),
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        // handle emitted messages
        match message {
            Message::InitDone => self.init_done = true,
            Message::Scrape => {
                return if !self.init_done {
                    Task::none()
                } else {
                    let content = self.editor_content.text();
                    Task::perform(
                        async move {
                            let results =
                                fitgirl_ddl_lib::scrape::scrape_game(content.trim()).await;
                            results
                        },
                        |results| Message::Scraped(results),
                    )
                };
            }
            Message::Scraped(results) => match results {
                Ok(ddls) => {
                    let path_part = ddls.path_part;
                    return Task::perform(
                        async move {
                            (
                                path_part,
                                futures_util::stream::iter(ddls.fuckingfast_links)
                                    .map(async |link| extract_ddl(link).await)
                                    .buffer_unordered(3)
                                    .collect::<Vec<_>>()
                                    .into_future()
                                    .await,
                            )
                        },
                        |(path_part, direct_links)| Message::Extracted {
                            path_part,
                            direct_links,
                        },
                    );
                }
                Err(_e) => {}
            },
            Message::Edit(action) => {
                self.editor_content.perform(action);
            }
            Message::Extracted {
                path_part,
                direct_links,
            } => {
                let mut message = String::new();

                for error in direct_links.iter().filter_map(|e| e.as_ref().err()) {
                    let _ = message.write_fmt(format_args!("error: {error:?}\n"));
                }

                return Task::future(async move {
                    let savefile = async {
                        if let Some(file) = rfd::AsyncFileDialog::new()
                            .add_filter("aria2 input file", &["txt"])
                            .set_title(format!("Save aria2 input for {path_part}"))
                            .save_file()
                            .await
                        {
                            let aria2_input = direct_links
                                .iter()
                                .flatten()
                                .map(
                                    |DDL {
                                         filename,
                                         direct_link,
                                     }| {
                                        format!(
                                            "{direct_link}\n    continue=true\n    out={filename}"
                                        )
                                    },
                                )
                                .join("\n");
                            file.write(aria2_input.as_bytes()).await;
                        }
                    };

                    if !message.is_empty() {
                        let msg = rfd::AsyncMessageDialog::new()
                            .set_title(&path_part)
                            .set_level(rfd::MessageLevel::Error)
                            .set_description(message)
                            .show();

                        futures_util::join!(msg, savefile);
                    } else {
                        savefile.await;
                    }
                })
                .discard();
            }
        }

        Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        // create the View Logic (UI)
        let row = widget::row![
            widget::text_editor(&self.editor_content)
                .placeholder("https://fitgirl-repacks.site/xxx-xxxxxx-xxxxxx/")
                .on_action(Message::Edit),
            widget::button("scrape").on_press(Message::Scrape)
        ];
        widget::container(row)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

fn main() -> Result<(), iced::Error> {
    nyquest_preset::register();

    // run the app from main function
    iced::application("fitgirl-ddl", Counter::update, Counter::view)
        .centered()
        .window_size(Size::new(800., 50.))
        .run_with(|| {
            (
                Counter::new(),
                iced::Task::perform(
                    async {
                        let _ = fitgirl_ddl_lib::init_nyquest().await;
                    },
                    |_| Message::InitDone,
                ),
            )
        })
}
