use iced::{Element};
use iced::widget::{Column, text};
use std::fs;

struct Player {
    list: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Default for Player {
    fn default() -> Self {
        let mut list = Vec::new();
        list.clear();
        if let Ok(dir) = fs::read_dir(r"D:\rust_projects\player\examples") {
            for entry in dir.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    list.push(file_name.to_string());
                }
            }
        }

        println!("{:?}", list);
        Self { list }
    }
}

impl Player {
    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        let elements: Vec<Element<Message>> = self
            .list
            .iter()
            .map(|item| text(item).size(20).into())
            .collect();

        // 3. Собираем вектор виджетов в контейнер Column
        Column::with_children(elements)
            .spacing(10)
            .padding(20)
            .into()
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
    iced::application(Player::default, Player::update, Player::view).run()
}
