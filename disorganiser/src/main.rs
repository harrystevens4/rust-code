use iced::widget::{
    column,text,row,table,grid,
    container,text_input,button,
};
use iced::Length;
use iced::color;
use iced::Element;
use std::path::{Path,PathBuf};

#[derive(Default)]
struct FileBrowser {
    path_input_text: String,
}
#[derive(Clone)]
enum FileBrowserMessage {
    PathInputChanged(String),
    ChangeLocation(PathBuf),
}

fn main() {
    println!("Hello, world!");
    iced::run(FileBrowser::update,FileBrowser::view);
}

impl FileBrowser {
    pub fn update(&mut self, message: FileBrowserMessage) {
        use FileBrowserMessage::*;
        match message {
            PathInputChanged(new_path) => self.path_input_text = new_path,
            ChangeLocation(new_path) => self.change_location(new_path),
        }
    }
    pub fn view(&self) -> impl Into<Element<'_,FileBrowserMessage>> {
        use FileBrowserMessage::*;
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
            button("Pictures")
                .on_press(ChangeLocation(PathBuf::from("/tmp"))),
            button("Videos")
                .on_press(ChangeLocation(PathBuf::from("/tmp"))),
            button("Desktop")
                .on_press(ChangeLocation(PathBuf::from("/tmp"))),
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
    pub fn update_path_input(&mut self) {
    }
    pub fn change_location(&mut self, new_location: impl AsRef<Path>) {
        self.path_input_text = new_location
            .as_ref()
            .to_string_lossy()
            .to_owned()
            .to_string();
        self.update_path_input();
    }
}
