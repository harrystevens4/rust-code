mod icalendar;
use std::env;
use std::io;
use std::error::Error;
use std::cmp::{min,max};
use icalendar::CombinedCalendar;
use ratatui::{
	DefaultTerminal,
	Frame,
	crossterm,
	crossterm::event::{KeyCode,KeyEventKind,Event},
	style::{Stylize,Style},
	buffer::Buffer,
	layout::{Rect,Constraint,Direction,Layout,Offset},
	widgets::{Block, Paragraph, Widget, Shadow, Borders, List, ListState, StatefulWidget},
	text::{Line, Text},
	symbols::{border},
};
use std::default::Default;
use chrono::{Local,NaiveDate,Datelike,Days,Months};

static STYLE_SELECTED_TEXT: Style = Style::new().on_red();
static DATE_FORMAT_STRING: &str = "%d/%m/%Y";

struct Application {
	exit: bool,
	selected_window: usize,
	calendar_view_size: usize, //1 - 1 day, 7 - week
	current_day_list_state: ListState,
	selected_date: NaiveDate,
	calendar_scroll_offset: usize, //how far to the right the selected day is
}
enum DaysOrMonths {
	Days(Days),
	Months(Months),
}

impl From<Days> for DaysOrMonths {
	fn from(days: Days) -> DaysOrMonths {
		DaysOrMonths::Days(days)
	}
}
impl From<Months> for DaysOrMonths {
	fn from(months: Months) -> DaysOrMonths {
		DaysOrMonths::Months(months)
	}
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
		let application = Application {
			exit: false,
			selected_window: 0, //date
			calendar_view_size: 3,
			current_day_list_state: ListState::default().with_selected(Some(0)),
			selected_date: Local::now().date_naive(),
			calendar_scroll_offset: 0,
		};
		application
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
					KeyCode::Char('=') => self.increase_calendar_view_size(1),
					KeyCode::Char('-') => self.decrease_calendar_view_size(1),
					KeyCode::Char('n') => self.set_selected_date(Local::now()),
					KeyCode::Right => 
						if self.selected_window == 0 {self.increase_selected_date_by(Days::new(1))}
						else if self.selected_window == 1 {self.scroll_calendar_right()}
					KeyCode::Left => 
						if self.selected_window == 0 {self.decrease_selected_date_by(Days::new(1))}
						else if self.selected_window == 1 {self.scroll_calendar_left()}
					KeyCode::Tab => self.selected_window = (self.selected_window + 1) % 2,
					_ => (),
				}
			},
			_ => (),
		}
		Ok(())
	}
	fn scroll_calendar_right(&mut self){
		if self.calendar_scroll_offset < self.calendar_view_size-1 {
			self.calendar_scroll_offset += 1;
		}else {
			self.increase_selected_date_by(Days::new(1));
		}
	}
	fn scroll_calendar_left(&mut self){
		if self.calendar_scroll_offset > 0 {
			self.calendar_scroll_offset -= 1;
		}else {
			self.decrease_selected_date_by(Days::new(1));
		}
	}
	fn set_selected_date(&mut self, new_date: impl Datelike){
		self.selected_date = NaiveDate::from_ymd(
			new_date.year(),
			new_date.month(),
			new_date.day()
		);
	}
	fn increase_calendar_view_size(&mut self, amount: usize){
		self.set_calendar_view_size(self.calendar_view_size+1)
	}
	fn decrease_calendar_view_size(&mut self, amount: usize){
		self.set_calendar_view_size(self.calendar_view_size-1)
	}
	fn set_calendar_view_size(&mut self, new_size: usize){
		//min and max bounds
		if new_size < 1 || new_size > 7 {return}
		self.calendar_view_size = new_size;
		//make sure the selected date is still visible
		self.calendar_scroll_offset = min(
			self.calendar_scroll_offset as isize,
			1-(new_size) as isize
		) as usize;
	}
	fn increase_selected_date_by(&mut self, time: impl Into<DaysOrMonths>){
		match time.into() {
			DaysOrMonths::Days(days) => self.selected_date = self.selected_date + days,
			DaysOrMonths::Months(months) => self.selected_date = self.selected_date + months,
		}
	}
	fn decrease_selected_date_by(&mut self, time: impl Into<DaysOrMonths>){
		match time.into() {
			DaysOrMonths::Days(days) => self.selected_date = self.selected_date - days,
			DaysOrMonths::Months(months) => self.selected_date = self.selected_date - months,
		}
	}
	fn draw(&mut self, frame: &mut Frame){
		frame.render_widget(self,frame.area());
	}
}
impl Widget for &mut Application {
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
			.title_bottom(Line::from(" n to switch to today ").centered())
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
		Line::from(self.selected_date.format(DATE_FORMAT_STRING).to_string())
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
			.title_bottom(Line::from(" =/- to change view ").centered())
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
			//seperates individual days
			let day_block = Block::new()
				.borders(Borders::LEFT)
				.title_top(Line::from(
					(self.selected_date + Days::new(i as u64))
					.format(DATE_FORMAT_STRING)
					.to_string()
				).centered());
			let item_list = List::new(["test event 1","test event 2","test event 3"])
				.block(day_block)
				.highlight_style(STYLE_SELECTED_TEXT);
			//only the selected date gets the selected ListState
			let mut list_state = if i == self.calendar_scroll_offset {
				self.current_day_list_state
			}else {
				ListState::default()
			};
			StatefulWidget::render(item_list,calendar_day_layout[i],buf,&mut list_state);
		}
	}
}
