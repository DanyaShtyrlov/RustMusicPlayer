// Work on visuals
// Make drop down list over the layout, so it  won't move it
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use iced::widget::{
    Column, button, column, container, image, mouse_area, row, scrollable, slider, text, text_input,
};
use iced::window::Position;
use iced::{
    Alignment, Background, Border, Color, Element, Length, Renderer, Size, Subscription, Task,
    Theme, border, window,
};
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use rodio::{Decoder, MixerDeviceSink, Player, Source};
use serde_json::Value;
use std::fs;
use std::time::Duration;

struct Palette;

impl Palette {
    //Window layout colors
    const WINDOW_BG: Color = Color::from_rgb8(81, 45, 32);
    const TEXT: Color = Color::from_rgb8(181, 136, 94);
    const WINDOW_BORDER: Color = Color::from_rgb8(115, 65, 32);

    //Button colors
    const COMMON_BUTTON: Color = Color::from_rgb8(181, 136, 94);
    const BUTTON_TEXT: Color = Color::from_rgb8(81, 45, 32);
    const WARNING_BUTTON: Color = Color::from_rgb8(195, 118, 40);
    const DANGER_BUTTON: Color = Color::from_rgb8(111, 36, 31);
    const SUCCESS_BUTTON: Color = Color::from_rgb8(139, 140, 82);

    //Slider colors
    const SLIDER_BG: Color = Color::from_rgb8(181, 136, 94);
    const HANDLE_BG: Color = Color::from_rgb8(115, 65, 32);
    const HANDLE_BORDER: Color = Color::from_rgb8(224, 196, 159);
    const SLIDER_BORDER: Color = Color::from_rgb8(224, 196, 159);

    //Container  colors
    const CONTAINER_BG: Color = Color::from_rgb8(115, 65, 32);
    const CONTAINER_BORDER: Color = Color::from_rgb8(224, 196, 159);

    //Text input colors
    const SEARCH_BG: Color = Color::from_rgb8(115, 65, 32);
    const SEARCH_SELECTION: Color = Color::from_rgb8(155, 90, 50);
    const SEARCH_BORDER_FOCUSED: Color = Color::from_rgb8(224, 196, 159);

    //Style constants
    const BORDER_RADIUS: f32 = 10.0;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Player,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Local,
    Internet,
}

#[derive(Debug, Clone)]
pub struct OnlineSong {
    title: String,
    id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiProvider {
    Audius,
    ArchiveOrg,
}

impl std::fmt::Display for ApiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiProvider::Audius => write!(f, "Audius API (Open REST API)"),
            ApiProvider::ArchiveOrg => write!(f, "Archive.org API (Open Archive)"),
        }
    }
}

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
    search_mode: SearchMode,
    search_results: Vec<OnlineSong>,
    is_searching: bool,
    is_downloading: bool,
    active_tab: ActiveTab,
    api_provider: ApiProvider,
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
    SetSearchMode,
    SubmitSearch,
    SearchResultsReceived(Result<Vec<OnlineSong>, String>),
    SelectOnlineSong(OnlineSong),
    DownloadFinished(Result<String, String>),
    SwitchTab(ActiveTab),
    SelectApiProvider(ApiProvider),
}

impl Default for RPlayer {
    fn default() -> Self {
        let mut list = Vec::new();
        let mut path_list = Vec::new();
        if let Ok(dir) = fs::read_dir("./songs") {
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
            search_mode: SearchMode::Local,
            search_results: Vec::new(),
            is_searching: false,
            is_downloading: false,
            active_tab: ActiveTab::Player,
            api_provider: ApiProvider::Audius,
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

    fn reload_songs(&mut self) {
        self.list.clear();
        self.path_list.clear();
        if let Ok(dir) = fs::read_dir("./songs") {
            for entry in dir.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    self.list.push(file_name.to_string());
                }
                self.path_list.push(path.into_string().unwrap_or_default());
            }
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

    fn view_settings(&self) -> Element<'_, Message> {
        let title = text("App settings").size(18).style(|_| text::Style {
            color: Some(Palette::TEXT),
        });

        let api_label = text("Search and download sources:")
            .size(14)
            .style(|_| text::Style {
                color: Some(Palette::TEXT),
            });

        let providers = [ApiProvider::Audius, ApiProvider::ArchiveOrg];

        let provider_buttons: Vec<Element<Message>> = providers
            .into_iter()
            .map(|provider| {
                let is_selected = self.api_provider == provider;
                button(text(provider.to_string()).size(13))
                    .width(Length::Fill)
                    .padding(10)
                    .style(move |theme: &Theme, status| {
                        let mut style = button::secondary(theme, status);
                        style.text_color = Palette::BUTTON_TEXT;
                        style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                        if is_selected {
                            style.background = Some(Background::Color(Palette::SUCCESS_BUTTON));
                        } else {
                            style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                        }
                        style
                    })
                    .on_press(Message::SelectApiProvider(provider))
                    .into()
            })
            .collect();

        let back_button = button(text("<- Return bact to player").size(13))
            .padding(10)
            .style(|theme: &Theme, status| {
                let mut style = button::primary(theme, status);
                style.background = Some(Background::Color(Palette::WARNING_BUTTON));
                style.text_color = Palette::BUTTON_TEXT;
                style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                style
            })
            .on_press(Message::SwitchTab(ActiveTab::Player));

        let settings_box = column![
            title,
            api_label,
            Column::with_children(provider_buttons).spacing(8),
            back_button
        ]
        .spacing(16)
        .padding(20);

        container(settings_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.active_tab = tab;
                Task::none()
            }

            Message::SelectApiProvider(provider) => {
                self.api_provider = provider;
                Task::none()
            }

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

            Message::SetSearchMode => {
                self.search_mode = match self.search_mode {
                    SearchMode::Local => SearchMode::Internet,
                    SearchMode::Internet => SearchMode::Local,
                };
                Task::none()
            }

            Message::SubmitSearch => {
                if self.search_mode == SearchMode::Internet
                    && !self.search_query.trim().is_empty()
                    && !self.is_searching
                {
                    self.is_searching = true;
                    self.search_results.clear();
                    let query = self.search_query.clone();
                    let provider = self.api_provider;
                    Task::perform(
                        search_music_by_provider(query, provider),
                        Message::SearchResultsReceived,
                    )
                } else {
                    Task::none()
                }
            }

            Message::SearchResultsReceived(result) => {
                self.is_searching = false;
                match result {
                    Ok(results) => {
                        self.search_results = results;
                    }
                    Err(err) => {
                        eprintln!("Ошибка поиска: {}", err);
                        self.search_results.clear();
                    }
                }
                Task::none()
            }

            Message::SelectOnlineSong(selected_song) => {
                self.is_downloading = true;
                self.search_results.clear();
                let provider = self.api_provider;
                Task::perform(
                    download_audio_by_provider(selected_song.id, selected_song.title, provider),
                    Message::DownloadFinished,
                )
            }

            Message::DownloadFinished(result) => {
                self.is_downloading = false;
                match result {
                    Ok(title) => {
                        println!("Успешно загружено: {}", title);
                        self.search_query.clear();
                        self.reload_songs();
                    }
                    Err(err) => {
                        eprintln!("Ошибка скачивания: {}", err);
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let settings_icon = if self.active_tab == ActiveTab::Settings {
            "✕"
        } else {
            "☰"
        };
        let target_tab = if self.active_tab == ActiveTab::Settings {
            ActiveTab::Player
        } else {
            ActiveTab::Settings
        };

        let settings_button = button(text(settings_icon).size(12))
            .padding([4, 8])
            .style(|theme: &Theme, status| {
                let mut style = button::secondary(theme, status);
                style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                style.text_color = Palette::BUTTON_TEXT;
                style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                style
            })
            .on_press(Message::SwitchTab(target_tab));

        let minimize_button = button(text("—").size(12))
            .padding([4, 8])
            .style(|theme: &Theme, status| {
                let mut style = button::secondary(theme, status);
                style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                style.text_color = Palette::BUTTON_TEXT;
                style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                style
            })
            .on_press(Message::MinimizeWindow);

        let maximize_button = button(text("☐").size(12))
            .padding([4, 8])
            .style(|theme: &Theme, status| {
                let mut style = button::warning(theme, status);
                style.background = Some(Background::Color(Palette::WARNING_BUTTON));
                style.text_color = Palette::BUTTON_TEXT;
                style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                style
            })
            .on_press(Message::MaximizeWindow);

        let close_button = button(text("✕").size(12))
            .padding([4, 8])
            .style(|theme: &Theme, status| {
                let mut style = button::danger(theme, status);
                style.background = Some(Background::Color(Palette::DANGER_BUTTON));
                style.text_color = Palette::BUTTON_TEXT;
                style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                style
            })
            .on_press(Message::CloseWindow);

        let title_drag_area = mouse_area(
            container(
                text("Rust Music Player")
                    .style(|theme: &Theme| {
                        let mut style = text::default(theme);
                        style.color = Some(Palette::TEXT);
                        style
                    })
                    .size(13),
            )
            .width(Length::Fill)
            .padding(6),
        )
        .on_press(Message::DragWindow);

        let title_bar = container(
            row![
                title_drag_area,
                settings_button,
                minimize_button,
                maximize_button,
                close_button
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding(8);

        let mode_label = match self.search_mode {
            SearchMode::Local => "Local",
            SearchMode::Internet => "Internet",
        };

        let set_mode_button: iced::widget::Button<'_, _, iced::Theme, Renderer> =
            button(text(mode_label).size(12))
                .padding(10)
                .style(move |theme: &Theme, status| {
                    let mut style = button::secondary(theme, status);
                    style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                    match self.search_mode {
                        SearchMode::Local => {
                            style.background = Some(Background::Color(Palette::SUCCESS_BUTTON));
                        }
                        SearchMode::Internet => {
                            style.background = Some(Background::Color(Palette::WARNING_BUTTON));
                        }
                    }
                    style.text_color = Palette::BUTTON_TEXT;
                    style
                })
                .on_press(Message::SetSearchMode);

        let dropdown_panel: Element<'_, Message> = if !self.search_results.is_empty() {
            let items: Vec<Element<Message>> = self
                .search_results
                .iter()
                .cloned()
                .map(|item| {
                    let label = item.title.clone();
                    button(text(label).size(12))
                        .width(Length::Fill)
                        .padding(6)
                        .style(|theme: &Theme, status| {
                            let mut style = button::secondary(theme, status);
                            style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                            style.text_color = Palette::BUTTON_TEXT;
                            style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                            style
                        })
                        .on_press(Message::SelectOnlineSong(item))
                        .into()
                })
                .collect();

            container(Column::with_children(items).spacing(4))
                .padding(6)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Palette::CONTAINER_BG)),
                    border: Border {
                        color: Palette::CONTAINER_BORDER,
                        width: 1.0,
                        radius: border::Radius::from(Palette::BORDER_RADIUS),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            Column::new().into()
        };

        let placeholder_text = if self.is_searching {
            "Net  search..."
        } else if self.is_downloading {
            "Downloading song..."
        } else {
            "Song search..."
        };

        let search_bar: Element<'_, Message> = container(
            column![
                row![
                    text_input(placeholder_text, &self.search_query)
                        .style(|theme: &Theme, status| {
                            let mut style = text_input::default(theme, status);
                            style.background = Background::Color(Palette::SEARCH_BG);
                            style.placeholder = Palette::TEXT;
                            style.value = Palette::TEXT;
                            style.selection = Palette::SEARCH_SELECTION;
                            if matches!(status, text_input::Status::Focused { .. }) {
                                style.border.color = Palette::SEARCH_BORDER_FOCUSED;
                                style.border.width = 2.0;
                            }
                            style
                        })
                        .on_input(Message::SearchInputChanged)
                        .on_submit(Message::SubmitSearch)
                        .padding(8)
                        .size(14),
                    set_mode_button
                ]
                .spacing(6)
                .align_y(Alignment::Center),
                dropdown_panel
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .padding(8)
        .into();

        let play_button = if self.is_playing {
            button("Pause")
                .style(|theme: &Theme, status| {
                    let mut style = button::secondary(theme, status);
                    style.background = Some(Background::Color(Palette::WARNING_BUTTON));
                    style.text_color = Palette::BUTTON_TEXT;
                    style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                    style
                })
                .on_press(Message::Pause)
        } else {
            button("Play")
                .style(|theme: &Theme, status| {
                    let mut style = button::primary(theme, status);
                    style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                    style.text_color = Palette::BUTTON_TEXT;
                    style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                    style
                })
                .on_press(Message::Play)
        };

        let previous_song_button = button("<")
            .style(|theme: &Theme, status| {
                let mut style = button::secondary(theme, status);
                style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                style.text_color = Palette::BUTTON_TEXT;
                style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                style
            })
            .on_press(Message::PrevSong);

        let next_song_button = button(">")
            .style(|theme: &Theme, status| {
                let mut style = button::secondary(theme, status);
                style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                style.text_color = Palette::BUTTON_TEXT;
                style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                style
            })
            .on_press(Message::NextSong);

        let seek_bar = slider(
            0.0..=self.song_duration,
            self.current_position,
            Message::SeekChanged,
        )
        .style(|theme: &Theme, status| {
            let mut style = slider::default(theme, status);
            style.rail = slider::Rail {
                backgrounds: (
                    Background::Color(Palette::SLIDER_BG),
                    Background::Color(Palette::SLIDER_BG),
                ),
                border: Border {
                    color: Palette::SLIDER_BORDER,
                    width: 1.0,
                    radius: border::Radius::from(Palette::BORDER_RADIUS),
                },
                width: 4.0,
            };
            style.handle = slider::Handle {
                shape: slider::HandleShape::Circle { radius: 8.0 },
                background: Background::Color(Palette::HANDLE_BG),
                border_width: 2.0,
                border_color: Palette::HANDLE_BORDER,
            };
            style
        })
        .width(Length::Fill)
        .on_release(Message::SeekReleased)
        .step(0.5_f32);

        let time_label = text(format!(
            "{} / {}",
            format_time(self.current_position),
            format_time(self.song_duration)
        ))
        .style(|theme: &Theme| {
            let mut style = text::default(theme);
            style.color = Some(Palette::TEXT);
            style
        })
        .size(14);

        let control_elements =
            container(row![previous_song_button, play_button, next_song_button].spacing(10))
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center);

        let cover = container(image(self.cover_handle.clone()))
            .style(|_theme| container::Style {
                text_color: Some(Palette::TEXT),
                background: Some(Background::Color(Palette::CONTAINER_BG)),
                border: Border {
                    color: Palette::CONTAINER_BORDER,
                    width: 1.0,
                    radius: border::Radius::from(Palette::BORDER_RADIUS),
                },
                ..Default::default()
            })
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        let left_panel = column![cover, seek_bar, time_label, control_elements];

        let query_lower = self.search_query.to_lowercase();

        let elements: Vec<Element<Message>> = self
            .list
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if self.search_mode == SearchMode::Local && !query_lower.is_empty() {
                    item.to_lowercase().contains(&query_lower)
                } else {
                    true
                }
            })
            .map(|(index, item)| {
                let is_selected = self.selected_song == Some(index);
                let item_button = button(text(item).size(12))
                    .width(Length::Fill)
                    .clip(true)
                    .style(move |theme: &Theme, status| {
                        let mut style = button::secondary(theme, status);
                        style.text_color = Palette::BUTTON_TEXT;
                        style.border.radius = iced::border::radius(Palette::BORDER_RADIUS);
                        if is_selected {
                            style.background = Some(Background::Color(Palette::SUCCESS_BUTTON));
                        } else {
                            style.background = Some(Background::Color(Palette::COMMON_BUTTON));
                        }
                        style
                    })
                    .on_press(Message::SelectSong(index));
                row![item_button].spacing(4).padding(4).into()
            })
            .collect();

        let song_list =
            container(scrollable(Column::with_children(elements).spacing(6)).height(Length::Fill))
                .style(|_theme| container::Style {
                    text_color: Some(Palette::TEXT),
                    background: Some(Background::Color(Palette::CONTAINER_BG)),
                    border: Border {
                        color: Palette::CONTAINER_BORDER,
                        width: 1.0,
                        radius: border::Radius::from(Palette::BORDER_RADIUS),
                    },
                    ..Default::default()
                });

        let content: Element<'_, Message> = match self.active_tab {
            ActiveTab::Player => {
                let main_content = row![left_panel, song_list].spacing(10).padding(20);
                Column::new().push(search_bar).push(main_content).into()
            }
            ActiveTab::Settings => self.view_settings(),
        };

        let window_layout = Column::new().push(title_bar).push(content);

        container(window_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(1)
            .style(|_theme| container::Style {
                background: Some(Background::Color(Palette::WINDOW_BG)),
                border: Border {
                    color: Palette::WINDOW_BORDER,
                    width: 2.0,
                    radius: border::Radius::from(0.0),
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

fn extract_cover(song_path: &str) -> Option<image::Handle> {
    let tagged_file = Probe::open(song_path).ok()?.read().ok()?;
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())?;
    let picture = tag.pictures().first()?;

    Some(image::Handle::from_bytes(picture.data().to_vec()))
}

async fn search_audius(query: String) -> Result<Vec<OnlineSong>, String> {
    let client = reqwest::Client::builder()
        .user_agent("RustPlayer/1.0")
        .build()
        .map_err(|e| format!("Ошибка HTTP клиента: {}", e))?;

    let url = format!(
        "https://discoveryprovider.audius.co/v1/tracks/search?query={}&app_name=RustPlayer",
        urlencoding::encode(&query)
    );

    let response: Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Ошибка сети: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Ошибка парсинга ответа: {}", e))?;

    let items = response["data"]
        .as_array()
        .ok_or_else(|| "Ничего не найдено".to_string())?;

    let results: Vec<_> = items
        .iter()
        .filter_map(|track| {
            let id = track["id"].as_str()?.to_string();
            let title = track["title"].as_str()?.to_string();
            let artist = track["user"]["name"].as_str().unwrap_or("Unknown");

            Some(OnlineSong {
                title: format!("{} - {}", artist, title),
                id,
            })
        })
        .take(5)
        .collect();

    if results.is_empty() {
        Err("По вашему запросу ничего не найдено".to_string())
    } else {
        Ok(results)
    }
}

async fn search_archive(query: String) -> Result<Vec<OnlineSong>, String> {
    let client = reqwest::Client::builder()
        .user_agent("RustPlayer/1.0")
        .build()
        .map_err(|e| format!("Ошибка HTTP клиента: {}", e))?;

    let url = format!(
        "https://archive.org/advancedsearch.php?q=title:({})%20AND%20mediatype:(audio)&fl[]=identifier,title,creator&rows=5&page=1&output=json",
        urlencoding::encode(&query)
    );

    let response: Value = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Ошибка сети Archive.org: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Ошибка парсинга JSON Archive.org: {}", e))?;

    let docs = response["response"]["docs"]
        .as_array()
        .ok_or_else(|| "Ничего не найдено".to_string())?;

    let songs: Vec<OnlineSong> = docs
        .iter()
        .filter_map(|doc| {
            let id = doc["identifier"].as_str()?.to_string();
            let title = doc["title"].as_str()?.to_string();
            let creator = doc["creator"].as_str().unwrap_or("Archive Audio");

            Some(OnlineSong {
                title: format!("{} - {}", creator, title),
                id,
            })
        })
        .collect();

    if songs.is_empty() {
        Err("В архиве ничего не найдено".to_string())
    } else {
        Ok(songs)
    }
}

async fn search_music_by_provider(
    query: String,
    provider: ApiProvider,
) -> Result<Vec<OnlineSong>, String> {
    match provider {
        ApiProvider::Audius => search_audius(query).await,
        ApiProvider::ArchiveOrg => search_archive(query).await,
    }
}

async fn download_audius_stream(id: String, title: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("RustPlayer/1.0")
        .build()
        .map_err(|e| format!("Ошибка HTTP клиента: {}", e))?;

    let stream_url = format!(
        "https://discoveryprovider.audius.co/v1/tracks/{}/stream?app_name=RustPlayer",
        id
    );

    let mut clean_title = sanitize_filename::sanitize(&title);
    if clean_title.trim().is_empty() {
        clean_title = format!("track_{}", id);
    }

    let _ = std::fs::create_dir_all("./songs");
    let file_path = format!("./songs/{}.mp3", clean_title);

    let audio_bytes = client
        .get(&stream_url)
        .send()
        .await
        .map_err(|e| format!("Ошибка скачивания файла: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Ошибка чтения байтов: {}", e))?;

    tokio::fs::write(&file_path, &audio_bytes)
        .await
        .map_err(|e| format!("Ошибка сохранения на диск: {}", e))?;

    Ok(clean_title)
}

async fn download_archive(id: String, title: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("RustPlayer/1.0")
        .build()
        .map_err(|e| format!("Ошибка HTTP клиента: {}", e))?;

    let metadata_url = format!("https://archive.org/metadata/{}", id);
    let metadata: Value = client
        .get(&metadata_url)
        .send()
        .await
        .map_err(|e| format!("Ошибка получения метаданных: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Ошибка парсинга метаданных: {}", e))?;

    let files = metadata["files"]
        .as_array()
        .ok_or_else(|| "Файлы в релизе не найдены".to_string())?;

    let target_file = files
        .iter()
        .find(|f| {
            let name = f["name"].as_str().unwrap_or("");
            name.to_lowercase().ends_with(".mp3")
        })
        .ok_or_else(|| "В данном релизе не найден файл в формате MP3".to_string())?;

    let file_name = target_file["name"]
        .as_str()
        .ok_or_else(|| "Имя файла отсутствует".to_string())?;

    let download_url = format!(
        "https://archive.org/download/{}/{}",
        id,
        urlencoding::encode(file_name)
    );

    let mut clean_title = sanitize_filename::sanitize(&title);
    if clean_title.trim().is_empty() {
        clean_title = format!("archive_{}", id);
    }

    let _ = std::fs::create_dir_all("./songs");
    let file_path = format!("./songs/{}.mp3", clean_title);

    let audio_bytes = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Ошибка скачивания файла: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Ошибка чтения байтов: {}", e))?;

    tokio::fs::write(&file_path, &audio_bytes)
        .await
        .map_err(|e| format!("Ошибка сохранения на диск: {}", e))?;

    Ok(clean_title)
}

async fn download_audio_by_provider(
    id: String,
    title: String,
    provider: ApiProvider,
) -> Result<String, String> {
    match provider {
        ApiProvider::Audius => download_audius_stream(id, title).await,
        ApiProvider::ArchiveOrg => download_archive(id, title).await,
    }
}

fn main() -> iced::Result {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let icon = window::icon::from_file_data(icon_bytes, None).unwrap();
    iced::application(RPlayer::default, RPlayer::update, RPlayer::view)
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
