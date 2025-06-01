use std::{env::current_exe, fmt::Write, path::PathBuf, sync::Arc, u32};

use fitgirl_ddl_lib::{
    errors::{ExtractError, ScrapeError},
    extract::{DDL, extract_ddl},
    scrape::GameInfo,
};
use iced::{
    Task,
    widget::{
        self,
        text_editor::{Action, Content},
    },
};
use tokio::sync::Semaphore;

pub struct State {
    editor_content: Content,
    init_done: bool,
    current_pos: u32,
    max_cap: u32,
    path_part: String,
    results: Vec<Result<DDL, ExtractError>>,
    downloading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    InitDone,
    Scrape,
    Scraped(Result<GameInfo, ScrapeError>),
    Edit(Action),
    ExtractedSingle {
        direct_link: Result<DDL, ExtractError>,
    },
    Extracted {
        path_part: String,
        direct_links: Vec<Result<DDL, ExtractError>>,
    },
}

// Implement our Counter
impl State {
    pub fn new() -> Self {
        // initialize the counter struct
        // with count value as 0.
        Self {
            init_done: false,
            editor_content: Content::new(),
            current_pos: 0,
            max_cap: f32::MAX as u32,
            path_part: String::new(),
            results: vec![],
            downloading: false,
        }
    }

    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        // handle emitted messages
        match message {
            Message::InitDone => self.init_done = true,
            Message::Scrape => {
                return if !self.init_done {
                    Task::none()
                } else {
                    self.downloading = true;

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

                    self.path_part = path_part;
                    self.max_cap = ddls.fuckingfast_links.len() as _;
                    self.current_pos = 0;

                    let sem = Arc::new(Semaphore::new(2));

                    return Task::batch(ddls.fuckingfast_links.into_iter().map(|url| {
                        let sem = sem.clone();
                        Task::perform(
                            async move {
                                let _sem = sem.acquire().await;
                                extract_ddl(url).await
                            },
                            |direct_link| Message::ExtractedSingle { direct_link },
                        )
                    }));
                }
                Err(_e) => {
                    self.downloading = false;
                }
            },
            Message::Edit(action) => {
                self.editor_content.perform(action);
            }
            Message::ExtractedSingle { direct_link } => {
                self.results.push(direct_link);
                self.current_pos += 1;

                if self.results.len() == self.max_cap as usize {
                    let mut direct_links = Vec::new();
                    direct_links.append(&mut self.results);
                    let path_part = self.path_part.clone();

                    self.downloading = false;
                    return Task::done(Message::Extracted {
                        path_part,
                        direct_links,
                    });
                }
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
                    let path_part = path_part.clone();
                    let path_part2 = path_part.clone();
                    let _dl = direct_links.clone();
                    tokio::task::spawn_blocking(|| {
                        let ddls = _dl.into_iter().flatten().collect::<Vec<_>>();
                        let Ok(ddls) = serde_json::to_string(&ddls) else {
                            return;
                        };

                        let mut cmd = PathBuf::from("fitgirl-ddl-iced-select");

                        for path in [
                            PathBuf::from(
                                #[cfg(windows)]
                                "./fitgirl-ddl-iced-select.exe",
                                #[cfg(not(windows))]
                                "./fitgirl-ddl-iced-select",
                            ),
                            current_exe().unwrap_or_default().join(
                                #[cfg(windows)]
                                "fitgirl-ddl-iced-select.exe",
                                #[cfg(not(windows))]
                                "fitgirl-ddl-iced-select",
                            ),
                        ] {
                            if path.exists() {
                                cmd = path;
                                break;
                            }
                        }

                        let _ = tokio::process::Command::new(cmd)
                            .arg(path_part2)
                            .arg(ddls)
                            .spawn();
                    });

                    if !message.is_empty() {
                        rfd::AsyncMessageDialog::new()
                            .set_title(&path_part)
                            .set_level(rfd::MessageLevel::Error)
                            .set_description(message)
                            .show()
                            .await;
                    }
                })
                .discard();
            }
        }

        Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        let button = if self.downloading {
            widget::button("scrape")
        } else {
            widget::button("scrape").on_press(Message::Scrape)
        };

        let row = widget::row![
            widget::text_editor(&self.editor_content)
                .placeholder("https://fitgirl-repacks.site/xxx-xxxxxx-xxxxxx/")
                .on_action(Message::Edit),
            button
        ];
        let col = widget::column![
            row,
            widget::vertical_space().height(5.0),
            widget::progress_bar(0.0..=self.max_cap as f32, self.current_pos as f32)
        ];
        widget::container(col)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}
