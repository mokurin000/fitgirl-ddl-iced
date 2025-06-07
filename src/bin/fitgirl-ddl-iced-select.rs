#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::BTreeMap, path::PathBuf};

use ahash::AHashMap;
use fitgirl_ddl_lib::extract::DDL;
use iced::widget::{self, Column};
use itertools::Itertools;

fn main() -> Result<(), iced::Error> {
    let path_part = std::env::args()
        .nth(1)
        .expect("expected first argument: path part of the game");
    let ddls = std::env::args()
        .nth(2)
        .expect("expected second argument: json of DDLs");
    let ddls: Vec<DDL> = serde_json::from_str(&ddls).expect("invalid json format!");
    // run the app from main function
    iced::application("fitgirl-ddl", State::update, State::view)
        .centered()
        .window_size(iced::Size::new(500., 500.))
        .run_with(|| (State::new(path_part, ddls), iced::Task::none()))
}

struct State {
    groups: AHashMap<String, Vec<DDL>>,
    path_part: String,
    groups_checked: BTreeMap<String, bool>,
}

#[derive(Debug, Clone)]
enum Message {
    Check(String),
    Uncheck(String),
    Export,
}

impl State {
    fn new(path_part: String, mut ddls: Vec<DDL>) -> Self {
        let mut groups: AHashMap<String, Vec<DDL>> = AHashMap::new();
        ddls.sort_by(|a, b| a.filename.cmp(&b.filename));

        for ddl in ddls {
            let group_name = ddl
                .filename
                .split_once(".part")
                .map(|(s, _)| s)
                .unwrap_or(&ddl.filename);
            groups.entry(group_name.to_string()).or_default().push(ddl);
        }

        let groups_sorted = groups
            .keys()
            .sorted()
            .cloned()
            .map(|s| {
                (
                    s.clone(),
                    !["selective", "optional"]
                        .iter()
                        .any(|keyword| s.contains(keyword)),
                )
            })
            .collect();

        Self {
            path_part,
            groups,
            groups_checked: groups_sorted,
        }
    }

    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::Check(name) => {
                self.groups_checked.insert(name, true);
            }
            Message::Uncheck(name) => {
                self.groups_checked.insert(name, false);
            }
            Message::Export => {
                let path = PathBuf::from(&self.path_part).with_extension("txt");
                let contents = self
                    .groups_checked
                    .iter()
                    .filter(|(_, c)| **c)
                    .map(|(name, _)| &self.groups[name])
                    .flatten()
                    .map(
                        |DDL {
                             filename,
                             direct_link,
                         }| {
                            format!("{direct_link}\n    out={filename}\n    continue=true")
                        },
                    )
                    .join("\n");
                return iced::Task::future(async move { tokio::fs::write(path, contents).await })
                    .discard();
            }
        }

        iced::Task::none()
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let mut column = Column::new();

        for (name, checked) in self.groups_checked.clone().into_iter() {
            let mut cbox = widget::checkbox(&name, checked);

            if !name.contains("fitgirl-repacks.site") {
                cbox = cbox.on_toggle(move |toggle| {
                    let action = if toggle {
                        Message::Check
                    } else {
                        Message::Uncheck
                    };
                    action(name.clone())
                });
            }

            column = column.push(cbox);
        }

        column = column.push(widget::button("Export").on_press(Message::Export));

        widget::container(column)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}
