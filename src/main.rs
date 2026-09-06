use iced::widget::{Column, button, row, scrollable, text};
use iced::window::Position;
use iced::{Element, Length, Size, window};
use std::fs;

struct Player {
    list: Vec<String>,
    selected_song: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Play,
    SelectSong(usize),
}

impl Default for Player {
    fn default() -> Self {
        let mut list = Vec::new();
        if let Ok(dir) = fs::read_dir(r"D:\rust_projects\player\examples") {
            for entry in dir.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    list.push(file_name.to_string());
                }
            }
        }

        Self {
            list,
            selected_song: None,
        }
    }
}

impl Player {
    fn update(&mut self, message: Message) {
        match message {
            Message::SelectSong(index) => {
                self.selected_song = Some(index);
            }
            Message::Play => todo!(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let play_button = button("Play").on_press(Message::Play);
        let elements: Vec<Element<Message>> = self
            .list
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = self.selected_song == Some(index);
                let item_button = button(text(item).size(15))
                    .width(Length::Fill)
                    .style(if is_selected {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .on_press(Message::SelectSong(index));
                row![item_button].spacing(8).into()
            })
            .collect();
        let list_view = scrollable(Column::with_children(elements).spacing(6)).height(Length::Fill);
        //let list_column = Column::with_children(elements);
        row![play_button, list_view].spacing(10).padding(20).into()
    }
}

fn main() -> iced::Result {
    //let mut input = String::new();
    //io::stdin().read_line(&mut input)?;
    //let dir = fs::read_dir(input.trim().trim_matches('"'))?;
    //for entry in dir {
    //    let entry = entry?;
    //    let path = entry.path();
    //    println!("{:?}", path.as_path());
    //}
    //let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    //let player = rodio::Player::connect_new(&handle.mixer());
    //
    //let reader = io::BufReader::new(
    //    fs::File::open("examples/Виктор Цой - Звезда по имени Солнце.mp3").unwrap(),
    //);
    //let source = Decoder::new(reader).unwrap();
    //
    //player.append(source);
    //player.try_seek(Duration::from_mins(1));
    //
    //player.sleep_until_end();
    iced::application(Player::default, Player::update, Player::view)
        .title("Rust Music Player")
        .window(window::Settings {
            size: Size::new(400.0, 600.0),
            fullscreen: false,
            position: Position::Centered,
            resizable: false,
            closeable: true,
            minimizable: true,
            decorations: true,
            transparent: false,
            blur: false,
            exit_on_close_request: true,
            ..Default::default()
        })
        .run()
}
