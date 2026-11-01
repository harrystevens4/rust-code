use iced::widget::{
    column,text,row,table,grid,
    container,text_input,
};
use iced::Length;
use iced::color;
use iced::Element;

#[derive(Default)]
struct FileBrowser {
    path_input_text: String,
}
#[derive(Clone)]
enum FileBrowserMessage {
    PathInputChanged(String),
}

fn main() {
    println!("Hello, world!");
    iced::run(FileBrowser::update,FileBrowser::view);
}

impl FileBrowser{
    pub fn update(&mut self, message: FileBrowserMessage) {
        use FileBrowserMessage::*;
        match message {
            PathInputChanged(new_path) => self.path_input_text = new_path,
        }
    }
    pub fn view(&self) -> impl Into<Element<'_,FileBrowserMessage>> {
        //====== design each of the sections ======
        //path bar at the top
        let file_list = [text("file1"),text("file2"),text("file3")];
        let path_bar = container(
            text_input("path",&self.path_input_text)
                .on_input(FileBrowserMessage::PathInputChanged)
        ).padding(10).center_x(Length::Fill).style(|theme| {
            container::rounded_box(theme)
                .background(color!(0x3618af))
        });
        //icon grid displaying the files
        let icon_grid = grid(file_list.into_iter().map(Element::from));
        //quick access locations down the left
        let quick_access_locations = container(column![
            text("Pictures"),text("Videos"),text("Desktop")
        ]).padding(10).center_y(Length::Fill).style(container::rounded_box);
        //====== the actuall layout ======
        row![
            quick_access_locations,
            column![
                path_bar,
                icon_grid,
            ],
        ]
    }
}
