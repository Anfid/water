use iced::{Application, Clipboard, Command, Element, Text};

pub struct Ui;

#[derive(Debug)]
pub enum Message {
    Loaded,
}

impl Application for Ui {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Ui, Command<Message>) {
        (Ui, Command::none())
    }

    fn title(&self) -> String {
        String::from("Water")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Text::new("Hello World").into()
    }
}
