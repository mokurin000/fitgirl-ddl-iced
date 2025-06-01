use iced::{Task, widget};

struct Counter {
    // This will be our state of the counter app
    // a.k.a the current count value
    count: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    // Emitted when the increment ("+") button is pressed
    IncrementCount,
    // Emitted when decrement ("-") button is pressed
    DecrementCount,
    None,
}

// Implement our Counter
impl Counter {
    fn new() -> Self {
        // initialize the counter struct
        // with count value as 0.
        Self { count: 0 }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        // handle emitted messages
        match message {
            Message::IncrementCount => self.count += 1,
            Message::DecrementCount => self.count -= 1,
            _ => {
                return Task::none();
            }
        }
        iced::Task::perform(
            async move {
                println!("hello inside tokio!");
            },
            |_| Message::None,
        )
    }

    fn view(&self) -> iced::Element<'_, Message> {
        // create the View Logic (UI)
        let row = widget::row![
            widget::button("-").on_press(Message::DecrementCount),
            widget::text(self.count).width(20.0),
            widget::button("+").on_press(Message::IncrementCount)
        ];
        widget::container(row)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

fn main() -> Result<(), iced::Error> {
    // run the app from main function
    iced::application("Counter Example", Counter::update, Counter::view)
        .run_with(|| (Counter::new(), iced::Task::none()))
}
