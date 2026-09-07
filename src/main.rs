// Make songs play while chosen (now its only plays via Start/Stop button)
// Work on visuals
use iced::theme::Palette;
use iced::widget::{Column, button, column, row, scrollable, slider, text};
use iced::window::Position;
use iced::{Color, Element, Length, Size, Subscription, Theme, window};
use rodio::{Decoder, MixerDeviceSink, Player, Source};
use std::fs;
use std::time::Duration;

struct RPlayer {
    list: Vec<String>,
    path_list: Vec<String>,
    selected_song: Option<usize>,
    current_song: Option<usize>,
    _device_handle: MixerDeviceSink,
    audio_player: Player,
    is_playing: bool,
    current_position: f32,
    song_duration: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Play,
    SelectSong(usize),
    Pause,
    SeekChanged(f32),
    SeekReleased,
    Tick,
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
            current_song: None,
            _device_handle: handle,
            audio_player: player,
            is_playing: false,
            current_position: 0.0,
            song_duration: 180.0,
        }
    }
}

impl RPlayer {
    fn subscription(&self) -> Subscription<Message> {
        if self.is_playing {
            iced::time::every(Duration::from_millis(250)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn play_song(&mut self, index: usize) {
        let Some(path) = self.path_list.get(index) else {
            return;
        };
        let Ok(file) = std::fs::File::open(path) else {
            return;
        };
        let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(file)) else {
            return;
        };

        self.song_duration = source
            .total_duration()
            .map(|d| d.as_secs_f32())
            .unwrap_or(180.0);

        self.current_position = 0.0;
        self.audio_player.stop();
        self.audio_player.append(source);
        self.audio_player.play();

        self.selected_song = Some(index);
        self.current_song = Some(index);
        self.is_playing = true;
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SelectSong(index) => {
                self.selected_song = Some(index);
                self.play_song(index);
            }

            Message::Play => {
                if let Some(selected_index) = self.selected_song {
                    if self.current_song != Some(selected_index) {
                        self.play_song(selected_index);
                    } else {
                        self.audio_player.play();
                        self.is_playing = true;
                    }
                }
            }

            Message::Pause => {
                self.audio_player.pause();
                self.is_playing = false;
            }

            Message::SeekChanged(new_position) => {
                self.current_position = new_position;
            }

            Message::SeekReleased => {
                let target = Duration::from_secs_f32(self.current_position);
                if self.audio_player.try_seek(target).is_err()
                    && let Some(index) = self.selected_song
                    && let Some(path) = self.path_list.get(index)
                    && let Ok(file) = fs::File::open(path)
                    && let Ok(mut source) = Decoder::new(std::io::BufReader::new(file))
                {
                    let _ = source.try_seek(target);
                    self.audio_player.stop();
                    self.audio_player.append(source);

                    if self.is_playing {
                        self.audio_player.play();
                    } else {
                        self.audio_player.pause();
                    }
                }
            }

            Message::Tick => {
                if self.is_playing {
                    self.current_position += 0.25;
                    if self.current_position >= self.song_duration {
                        self.current_position = 0.0;
                        self.is_playing = false;
                        self.audio_player.stop();
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let play_button = if self.is_playing {
            button("Pause")
                .style(button::secondary)
                .on_press(Message::Pause)
        } else {
            button("Play")
                .style(button::primary)
                .on_press(Message::Play)
        };
        let seek_bar = slider(
            0.0..=self.song_duration,
            self.current_position,
            Message::SeekChanged,
        )
        .on_release(Message::SeekReleased)
        .step(0.5_f32);

        let time_label = text(format!(
            "{} / {}",
            format_time(self.current_position),
            format_time(self.song_duration)
        ))
        .size(13);

        let left_panel = column![seek_bar, time_label, play_button];

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
        row![left_panel, list_view].spacing(10).padding(20).into()
    }
}

fn format_time(seconds: f32) -> String {
    let secs = seconds as u32;
    let mins = secs / 60;
    let rem_secs = secs % 60;
    format!("{:02}:{:02}", mins, rem_secs)
}

fn custom_theme(_state: &RPlayer) -> Theme {
    let palette = Palette {
        background: Color::from_rgb8(30, 30, 46),
        text: Color::from_rgb8(205, 214, 244),
        primary: Color::from_rgb8(203, 166, 247),
        success: Color::from_rgb8(166, 227, 161),
        warning: Color::from_rgb8(243, 139, 168),
        danger: Color::from_rgb8(243, 139, 168),
    };
    Theme::custom("Purple".to_string(), palette)
}

fn main() -> iced::Result {
    //player.try_seek(Duration::from_mins(1));
    iced::application(RPlayer::default, RPlayer::update, RPlayer::view)
        .title("Rust Music Player")
        .theme(custom_theme)
        .subscription(RPlayer::subscription)
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
