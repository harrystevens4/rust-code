mod icalendar;
use std::env;
use std::io;
use std::error::Error;
use icalendar::CombinedCalendar;
use ratatui::{
	DefaultTerminal,
	Frame,
	crossterm,
	crossterm::event::{KeyCode,KeyEventKind,Event},
	style::{Stylize,Style},
	buffer::Buffer,
	layout::{Rect,Constraint,Direction,Layout,Offset},
	widgets::{Block, Paragraph, Widget, Shadow, Borders},
	text::{Line, Text},
	symbols::{border},
};
use std::default::Default;

static STYLE_SELECTED_TEXT: Style = Style::new().on_red();

struct Application {
	exit: bool,
	selected_window: usize,
	calendar_view_size: usize, //1 - 1 day, 7 - week
}

fn main() -> Result<(),()>{
	//====== grab the calendars ======
	let Some(calendar_url) = env::args().nth(1)
	else {
		eprintln!("please provide ics url as first argument");
		return Err(());
	};
	println!("fetching calendars...");
	let calendar_raw = match reqwest::blocking::get(calendar_url).map(|c| c.text()).flatten(){
		Ok(c) => c,
		Err(e) => {
			eprintln!("Error fetching calendar: {e}");
			return Err(());
		}
	};
	println!("loading calendars...");
	let calendar = CombinedCalendar::load_from_strings(vec![
		("calendar1",&calendar_raw)
	]);
	//println!("{calendar:#?}");
	//====== ratatui ======
	let mut application = Application::default();
	if let Err(e) = ratatui::run(move |terminal| application.tui_loop(terminal)){
		eprintln!("Error in tui loop: {e}");
		return Err(());
	}
	Ok(())
}

impl Default for Application {
	fn default() -> Application {
		Application {
			exit: false,
			selected_window: 0, //date
			calendar_view_size: 3,
		}
	}
}
impl Application {
	pub fn tui_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<(),Box<dyn Error>>{
		while self.exit == false {
			terminal.draw(|frame| self.draw(frame))?;
			self.handle_events()?;
		}
		Ok(())
	}
	fn exit(&mut self){
		self.exit = true;
	}
	fn handle_events(&mut self) -> io::Result<()>{
		match crossterm::event::read()?{
			Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
				match key_event.code {
					KeyCode::Char('q') => self.exit(),
					KeyCode::Char('+') => {
						if self.calendar_view_size <= 7 {self.calendar_view_size += 1}
					},
					KeyCode::Char('-') => {
						if self.calendar_view_size > 1 {self.calendar_view_size -= 1}
					},
					KeyCode::Tab => self.selected_window = (self.selected_window + 1) % 2,
					_ => (),
				}
			},
			_ => (),
		}
		Ok(())
	}
	fn draw(&self, frame: &mut Frame){
		frame.render_widget(self,frame.area());
	}
}
impl Widget for &Application {
	fn render(self, area: Rect, buf: &mut Buffer){
		//====== main layouts ======
		let [top_rect,bottom_rect] = area.layout(&Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Min(3),
				Constraint::Percentage(100),
			])
		);
		let [event_info_rect,calendar_display_rect] = bottom_rect.layout(&Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![
				Constraint::Min(14),
				Constraint::Percentage(100),
			])
		);
		//====== date selector ======
		let date_selector_block = Block::bordered()
			.title(Line::styled(" Date ",
				if self.selected_window == 0 {STYLE_SELECTED_TEXT}
				else {Style::new()}
			).centered())
			.border_set(border::THICK);
		let date_selector_block_rect = date_selector_block.inner(top_rect);
		date_selector_block.render(top_rect,buf);
		//text layout in block
		let [view_type,relative_day,date_selected,weekday] = date_selector_block_rect.layout(&Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![
				Constraint::Fill(1),
				Constraint::Fill(1),
				Constraint::Fill(1),
				Constraint::Fill(1),
			])
		);
		//text
		Line::from("Month View")
			.centered()
			.render(view_type,buf);
		Line::from("Today")
			.centered()
			.render(relative_day,buf);
		Line::from("14/9/26")
			.centered()
			.render(date_selected,buf);
		Line::from("Monday")
			.centered()
			.render(weekday,buf);
		//====== event info ======
		Block::bordered()
			.title(Line::from(" Event Info ").centered())
			.border_set(border::THICK)
			.render(event_info_rect,buf);
		//====== calendar display ======
		//block outline
		let calendar_display_block = Block::bordered()
			.title_bottom(Line::from("+/- to change view").centered())
			.title(Line::styled(" Calendar ",
				if self.selected_window == 1 {STYLE_SELECTED_TEXT}
				else {Style::new()}
			).centered())
			.border_set(border::THICK);
		let calendar_display_block_rect = calendar_display_block.inner(calendar_display_rect);
		calendar_display_block.render(calendar_display_rect,buf);
		//calendar day layouts
		let calendar_day_layout = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Fill(1); self.calendar_view_size])
			.split(calendar_display_block_rect);
		for i in 0..(self.calendar_view_size){
			Block::new()
				.borders(Borders::LEFT)
				.title_top(i.to_string())
				.render(calendar_day_layout[i],buf);
		}
	}
}
