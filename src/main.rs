// Work on visuals
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use iced::theme::Palette;
use iced::widget::{
    Column, button, column, container, image, mouse_area, row, scrollable, slider, text, text_input,
};
use iced::window::Position;
use iced::{
    Alignment, Background, Border, Color, Element, Length, Size, Subscription, Task, Theme, border,
    window,
};
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
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
    cover_handle: image::Handle,
    search_query: String,
}

#[derive(Debug, Clone)]
enum Message {
    MinimizeWindow,
    MaximizeWindow,
    CloseWindow,
    DragWindow,
    Play,
    SelectSong(usize),
    Pause,
    SeekChanged(f32),
    SeekReleased,
    Tick,
    NextSong,
    PrevSong,
    SearchInputChanged(String),
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

        let cover_handle =
            image::Handle::from_bytes(include_bytes!("../assets/dummy.png").as_slice());

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
            cover_handle,
            search_query: String::new(),
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

        self.cover_handle = extract_cover(path).unwrap_or_else(|| {
            image::Handle::from_bytes(&include_bytes!("../assets/dummy.png")[..])
        });
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MinimizeWindow => window::latest().then(|id| {
                if let Some(id) = id {
                    window::minimize(id, true)
                } else {
                    Task::none()
                }
            }),
            Message::MaximizeWindow => window::latest().then(|id| {
                if let Some(id) = id {
                    window::toggle_maximize(id)
                } else {
                    Task::none()
                }
            }),

            Message::CloseWindow => window::latest().then(|id| {
                if let Some(id) = id {
                    window::close(id)
                } else {
                    Task::none()
                }
            }),

            Message::DragWindow => window::latest().then(|id| {
                if let Some(id) = id {
                    window::drag(id)
                } else {
                    Task::none()
                }
            }),

            Message::SelectSong(index) => {
                self.selected_song = Some(index);
                self.play_song(index);
                Task::none()
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
                Task::none()
            }

            Message::Pause => {
                self.audio_player.pause();
                self.is_playing = false;
                Task::none()
            }

            Message::SeekChanged(new_position) => {
                self.current_position = new_position;
                Task::none()
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
                Task::none()
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
                Task::none()
            }

            Message::NextSong => {
                if !self.path_list.is_empty() {
                    let current_idx = self.current_song.unwrap_or(0);
                    let next_idx = (current_idx + 1) % self.path_list.len();
                    self.play_song(next_idx);
                }
                Task::none()
            }

            Message::PrevSong => {
                if !self.path_list.is_empty() {
                    let current_idx = self.current_song.unwrap_or(0);
                    let prev_idx = if current_idx == 0 {
                        self.path_list.len() - 1
                    } else {
                        current_idx - 1
                    };
                    self.play_song(prev_idx);
                }
                Task::none()
            }

            Message::SearchInputChanged(query) => {
                self.search_query = query;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let minimize_button = button(text("—").size(12))
            .padding([4, 8])
            .style(|theme: &Theme, status| {
                let mut style = button::secondary(theme, status);
                style.border.radius = iced::border::radius(10.0);
                style
            })
            .on_press(Message::MinimizeWindow);

        let maximize_button = button(text("☐").size(12))
            .padding([4, 8])
            .style(|theme: &Theme, status| {
                let mut style = button::warning(theme, status);
                style.border.radius = iced::border::radius(10.0);
                style
            })
            .on_press(Message::MaximizeWindow);

        let close_button = button(text("✕").size(12))
            .padding([4, 8])
            .style(|theme: &Theme, status| {
                let mut style = button::danger(theme, status);
                style.border.radius = iced::border::radius(10.0);
                style
            })
            .on_press(Message::CloseWindow);

        let title_drag_area = mouse_area(
            container(text("Rust Music Player").size(13))
                .width(Length::Fill)
                .padding(6),
        )
        .on_press(Message::DragWindow);

        let title_bar = container(
            row![
                title_drag_area,
                minimize_button,
                maximize_button,
                close_button
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(8);

        let search_bar: Element<'_, Message> = container(
            text_input("Song search...", &self.search_query)
                .on_input(Message::SearchInputChanged)
                .padding(8)
                .size(14),
        )
        .style(container::secondary)
        .width(Length::Fill)
        .into();

        let play_button = if self.is_playing {
            button("Pause")
                .style(|theme: &Theme, status| {
                    let mut style = button::secondary(theme, status);
                    style.border.radius = iced::border::radius(50.0);
                    style
                })
                .on_press(Message::Pause)
        } else {
            button("Play")
                .style(|theme: &Theme, status| {
                    let mut style = button::primary(theme, status);
                    style.border.radius = iced::border::radius(50.0);
                    style
                })
                .on_press(Message::Play)
        };

        let previous_song_button = button("<")
            .style(|theme: &Theme, status| {
                let mut style = button::secondary(theme, status);
                style.border.radius = iced::border::radius(50.0);
                style
            })
            .on_press(Message::PrevSong);

        let next_song_button = button(">")
            .style(|theme: &Theme, status| {
                let mut style = button::secondary(theme, status);
                style.border.radius = iced::border::radius(50.0);
                style
            })
            .on_press(Message::NextSong);

        let seek_bar = slider(
            0.0..=self.song_duration,
            self.current_position,
            Message::SeekChanged,
        )
        .width(Length::Fill)
        .on_release(Message::SeekReleased)
        .step(0.5_f32);

        let time_label = text(format!(
            "{} / {}",
            format_time(self.current_position),
            format_time(self.song_duration)
        ))
        .size(14);

        let control_elements =
            container(row![previous_song_button, play_button, next_song_button].spacing(10))
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center);

        let cover = container(image(self.cover_handle.clone()))
            .style(container::primary)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        let left_panel = column![cover, seek_bar, time_label, control_elements];

        let elements: Vec<Element<Message>> = self
            .list
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = self.selected_song == Some(index);
                let item_button = button(text(item).size(12))
                    .width(Length::Fill)
                    .style(if is_selected {
                        button::success
                    } else {
                        button::secondary
                    })
                    .on_press(Message::SelectSong(index));
                row![item_button].spacing(4).padding(4).into()
            })
            .collect();

        let song_list =
            container(scrollable(Column::with_children(elements).spacing(6)).height(Length::Fill))
                .style(container::primary);

        let main_content = row![left_panel, song_list].spacing(10).padding(20);

        let window_layout = Column::new()
            .push(title_bar)
            .push(search_bar)
            .push(main_content);

        container(window_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(1)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgb8(81, 45, 32))),
                border: Border {
                    color: Color::from_rgb8(115, 65, 32),
                    width: 2.0,
                    radius: border::Radius::from(10.0),
                },
                ..Default::default()
            })
            .into()
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
        background: Color::from_rgb8(81, 45, 32),
        text: Color::from_rgb8(181, 136, 94),
        primary: Color::from_rgb8(115, 65, 32),
        success: Color::from_rgb8(139, 140, 82),
        warning: Color::from_rgb8(195, 118, 40),
        danger: Color::from_rgb8(111, 36, 31),
    };
    Theme::custom("ArinasCoffee".to_string(), palette)
}

fn extract_cover(song_path: &str) -> Option<image::Handle> {
    let tagged_file = Probe::open(song_path).ok()?.read().ok()?;
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())?;
    let picture = tag.pictures().first()?;

    Some(image::Handle::from_bytes(picture.data().to_vec()))
}

fn main() -> iced::Result {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon = window::icon::from_file_data(icon_bytes, None).unwrap();
    iced::application(RPlayer::default, RPlayer::update, RPlayer::view)
        .title("Rust Music Player")
        .theme(custom_theme)
        .subscription(RPlayer::subscription)
        .window(window::Settings {
            icon: Some(icon),
            size: Size::new(400.0, 600.0),
            fullscreen: false,
            position: Position::Centered,
            resizable: false,
            closeable: true,
            minimizable: true,
            decorations: false,
            transparent: true,
            blur: false,
            exit_on_close_request: true,
            ..Default::default()
        })
        .run()
}
