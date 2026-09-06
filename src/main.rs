use iced::widget::{Column, button, row, scrollable, text};
use iced::window::Position;
use iced::{Element, Length, Size, window};
use rodio::{Decoder, MixerDeviceSink, Player};
use std::fs;
use std::io;

struct RPlayer {
    list: Vec<String>,
    path_list: Vec<String>,
    selected_song: Option<usize>,
    _device_handle: MixerDeviceSink,
    audio_player: Player,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Play,
    SelectSong(usize),
}

impl Default for RPlayer {
    fn default() -> Self {
        let mut list = Vec::new();
        let mut path_list = Vec::new();
        if let Ok(dir) = fs::read_dir(r"D:\rust_projects\player\examples") {
            for entry in dir.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    list.push(file_name.to_string());
                }
                path_list.push(path.into_string().unwrap_or_default());
            }
        }

        let handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        let player = rodio::Player::connect_new(handle.mixer());

        Self {
            list,
            path_list,
            selected_song: None,
            _device_handle: handle,
            audio_player: player,
        }
    }
}

impl RPlayer {
    fn update(&mut self, message: Message) {
        match message {
            Message::SelectSong(index) => {
                self.selected_song = Some(index);
            }
            Message::Play => {
                let chosen_song_path = self.path_list[self.selected_song.unwrap()].clone();
                let reader = io::BufReader::new(fs::File::open(chosen_song_path).unwrap());
                let source = Decoder::new(reader).unwrap();
                self.audio_player.stop();
                self.audio_player.append(source);
                self.audio_player.play();
                println!("{}", self.path_list[self.selected_song.unwrap()]);
            }
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
    //player.try_seek(Duration::from_mins(1));
    iced::application(RPlayer::default, RPlayer::update, RPlayer::view)
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
