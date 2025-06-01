mod ui;
use ui::{Message, State};
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
                        let _ = fitgirl_ddl_lib::init_nyquest().await;
                    },
                    |_| Message::InitDone,
                ),
            )
        })
}
