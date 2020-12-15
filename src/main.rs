use iced::{button, executor, Align, Button, Column, Element, Sandbox, Settings, Text, Application, Subscription, Command};

mod websocket;
//mod foo;

pub fn main() -> iced::Result {
    Counter::run(Settings::default())
}

#[derive(Debug)]
enum State {
    Ready,
    Connected,
    Disconnected,
}

#[derive( Debug)]
struct Counter {
    value: i32,
    state: State,
    increment_button: button::State,
    decrement_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    IncrementPressed,
    DecrementPressed,
    Websocket(websocket::Progress),
}

impl Application for Counter {
    type Message = Message;
    type Flags = ();
    type Executor = executor::Default;


    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Counter {
            value: 0,
            state: State::Ready,
            increment_button: Default::default(),
            decrement_button: Default::default()
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        println!("got message {:?}", message);
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
            _ => ()
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        println!("called subscription {:?}", self);
        match self.state {
            State::Ready =>
                websocket::file("http://127.0.0.1:8000").map(Message::Websocket),
            _ => Subscription::none()
        }
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(
                Button::new(&mut self.increment_button, Text::new("Increment"))
                    .on_press(Message::IncrementPressed),
            )
            .push(Text::new(self.value.to_string()).size(50))
            .push(
                Button::new(&mut self.decrement_button, Text::new("Decrement"))
                    .on_press(Message::DecrementPressed),
            )
            .into()
    }
}
