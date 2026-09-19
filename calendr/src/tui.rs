use std::cmp::{min,max};
use ratatui::{
	DefaultTerminal,
	Frame,
	crossterm,
	crossterm::event::{KeyCode,KeyEventKind,Event},
	style::{Stylize,Style},
	buffer::Buffer,
	layout::{Rect,Constraint,Direction,Layout,Offset},
	widgets::{Block, Paragraph, Widget, Shadow, Borders, List, ListState, StatefulWidget, Wrap},
	text::{Line, Text},
	symbols::{border},
};
use std::default::Default;
use chrono::{Local,NaiveDate,Datelike,Days,Months,TimeDelta,NaiveTime,NaiveDateTime};
use crate::{DATE_FORMAT_STRING,TIME_FORMAT_STRING,STYLE_SELECTED_TEXT,STYLE_HIGHLIGHTED_TEXT};
use std::error::Error;
use std::ops::Sub;
use crate::icalendar::{CombinedCalendar,CalendarEvent};
use std::io;

pub struct Application {
	exit: bool,
	selected_window: SelectedWindow, //0 for calendar 1 for event info
	calendar_view_size: usize, //1 - 1 day, 7 - week
	selected_event: Option<usize>,
	selected_date: NaiveDate,
	calendar_scroll_offset: usize, //how far to the right the selected day is
	calendar: CombinedCalendar,
}
enum DaysOrMonths {
	Days(Days),
	Months(Months),
}
#[derive(PartialEq)]
enum SelectedWindow {
	CalendarDisplay,
	EventInfo,
}

//lowk did not need to be a trait but i wanted to try out making my own
trait DateOffsetString<U: Datelike + Copy>: Datelike + Copy {
	fn string_offset(self, other: U) -> String
	where Self: Sub<U, Output = TimeDelta> {
		let diff = -(self - other);
		match diff.num_days() {
			..=-2 => {
				if diff.num_weeks() == 0 && other.weekday().num_days_from_monday() < self.weekday().num_days_from_monday(){
					String::from("This Week")
				}else if diff.num_weeks() == -1 && other.weekday().num_days_from_monday() <= self.weekday().num_days_from_monday(){
					String::from("Last Week")
				}else if diff.num_weeks() == 0 && other.weekday().num_days_from_monday() > self.weekday().num_days_from_monday(){
					String::from("Last Week")
				}else if other.month() == self.month() && other.year() == self.year(){
					String::from("This Month")
				}else if other.month()+1 == self.month() && other.year() == self.year(){
					String::from("Last Month")
				}else if other.year() == self.year(){
					String::from("This Year")
				}else if other.year()+1 == self.year() {
					String::from("Last Year")
				}else {
					format!("{} Years ago",self.year() - other.year())
				}
			},
			-1 => String::from("Yesterday"),
			0 => String::from("Today"),
			1 => String::from("Tomorrow"),
			2.. => {
				if diff.num_weeks() == 0 && other.weekday().num_days_from_monday() < self.weekday().num_days_from_monday(){
					String::from("Next Week")
				}else if diff.num_weeks() == 0 {
					String::from("This Week")
				}else if diff.num_weeks() == 1 && other.weekday().num_days_from_monday() >= self.weekday().num_days_from_monday(){
					String::from("Next Week")
				}else if other.month() == self.month() && other.year() == self.year(){
					String::from("This Month")
				}else if other.year() == self.year(){
					String::from("This Year")
				}else if other.year() == self.year()+1 {
					String::from("Next Year")
				}else {
					format!("{} Years away",other.year() - self.year())
				}
			}
		}
	}
}
trait IsToday {
	fn is_today(&self) -> bool;
}

impl<T: Datelike + Copy> DateOffsetString<T> for NaiveDate {}
impl IsToday for NaiveDate {
	fn is_today(&self) -> bool {
		let now = Local::now().date_naive();
		(self.day(),self.month(),self.year()) == (now.day(),now.month(),now.year())
	}
}
impl SelectedWindow {
	pub fn next(&mut self){
		use SelectedWindow::*;
		*self = match self {
			CalendarDisplay => EventInfo,
			EventInfo => CalendarDisplay,
		}
	}
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

impl Application {
	//============ general calendar control functions ============
	pub fn new(calendar: CombinedCalendar) -> Application {
		let application = Application {
			exit: false,
			selected_window: SelectedWindow::CalendarDisplay,
			calendar_view_size: 3,
			//current_day_list_state: ListState::default().with_selected(Some(0)),
			selected_event: Some(0),
			selected_date: Local::now().date_naive(),
			calendar_scroll_offset: 0,
			calendar: calendar
		};
		application
	}
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
					KeyCode::Char('[') => self.decrease_selected_date_by(Months::new(1)),
					KeyCode::Char(']') => self.increase_selected_date_by(Months::new(1)),
					KeyCode::Down =>
						if self.selected_window == SelectedWindow::CalendarDisplay {
							self.select_next_event();
						}else {
						},
					KeyCode::Up =>
						if self.selected_window == SelectedWindow::CalendarDisplay {
							self.select_prev_event()
						}else {
						},
					KeyCode::Right => self.scroll_calendar_right(),
					KeyCode::Left => self.scroll_calendar_left(),
					KeyCode::Tab => self.selected_window.next(),
					_ => (),
				}
			},
			_ => (),
		}
		Ok(())
	}
	fn select_next_event(&mut self){
		let event_count = self.calendar
			.get_events_for_date(self.selected_date)
			.len();
		if event_count > 0 {
			self.selected_event = self.selected_event.map(|i| min(i+1,event_count-1));
		}
	}
	fn select_prev_event(&mut self){
		self.selected_event = self.selected_event.map(|i| if i > 0 {i-1} else {i});
	}
	fn scroll_calendar_right(&mut self){
		if self.calendar_scroll_offset < self.calendar_view_size-1 {
			self.calendar_scroll_offset += 1;
		}
		self.increase_selected_date_by(Days::new(1));
	}
	fn scroll_calendar_left(&mut self){
		if self.calendar_scroll_offset > 0 {
			self.calendar_scroll_offset -= 1;
		}
		self.decrease_selected_date_by(Days::new(1));
	}
	fn set_selected_date(&mut self, new_date: impl Datelike){
		self.selected_date = NaiveDate::from_ymd(
			new_date.year(),
			new_date.month(),
			new_date.day()
		);
		self.selected_event = Some(0);
	}
	fn increase_calendar_view_size(&mut self, amount: usize){
		self.set_calendar_view_size(self.calendar_view_size+amount)
	}
	fn decrease_calendar_view_size(&mut self, amount: usize){
		self.set_calendar_view_size(self.calendar_view_size-amount)
	}
	fn set_calendar_view_size(&mut self, new_size: usize){
		//min and max bounds
		if new_size < 1 || new_size > 7 {return}
		self.calendar_view_size = new_size;
		//make sure the selected date is still visible
		self.calendar_scroll_offset = min(
			self.calendar_scroll_offset as isize,
			(new_size-1) as isize
		) as usize;
	}
	fn increase_selected_date_by(&mut self, time: impl Into<DaysOrMonths>){
		match time.into() {
			DaysOrMonths::Days(days) => self.set_selected_date(self.selected_date + days),
			DaysOrMonths::Months(months) => self.set_selected_date(self.selected_date + months),
		}
	}
	fn decrease_selected_date_by(&mut self, time: impl Into<DaysOrMonths>){
		match time.into() {
			DaysOrMonths::Days(days) => self.set_selected_date(self.selected_date - days),
			DaysOrMonths::Months(months) => self.set_selected_date(self.selected_date - months),
		}
	}
	//============ rendering functions ============
	fn format_event_timespan(event: &CalendarEvent) -> String {
		if event.is_all_day(){
			format!("All day")
		}else {
			format!("{} - {}",
				event.start_time()
					.map(|t| t.format(TIME_FORMAT_STRING).to_string())
					.unwrap_or(String::from("")),
				event.end_time()
					.map(|t| t.format(TIME_FORMAT_STRING).to_string())
					.unwrap_or(String::from(""))
			)
		}
	}
	fn draw(&mut self, frame: &mut Frame){
		frame.render_widget(self,frame.area());
	}
	fn render_date_selector(&mut self, area: Rect, buf: &mut Buffer){
		let date_selector_block = Block::bordered()
			.title(Line::from(" Date ").centered())
			.title_bottom(Line::from(" n to switch to today ━━━ [ and ] to switch months").centered())
			.border_set(border::THICK);
		let date_selector_block_rect = date_selector_block.inner(area);
		date_selector_block.render(area,buf);
		//text layout in block
		let [view_type,date_selected,relative_day] = date_selector_block_rect.layout(&Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![
				Constraint::Fill(1),
				Constraint::Fill(1),
				Constraint::Fill(1),
			])
		);
		//text
		Line::from(format!("{} Day{}",self.calendar_view_size,
				if self.calendar_view_size == 1 {""}
				else {"s"} //days plural if more than one
			))
			.centered()
			.render(view_type,buf);
		Line::from(self.selected_date.format(DATE_FORMAT_STRING).to_string())
			.centered()
			.render(date_selected,buf);
		Line::from(Local::now().date_naive().string_offset(self.selected_date))
			.centered()
			.render(relative_day,buf);
	}
	fn render_event_info(&mut self, area: Rect, buf: &mut Buffer){
		//build the main block
		let event_info_block = Block::bordered()
			.title(Line::styled(" Event Info ",
				if self.selected_window == SelectedWindow::EventInfo {STYLE_SELECTED_TEXT}
				else {Style::new()}
			).centered())
			.border_set(border::THICK);
		//grab all the info
		let events = self.calendar.get_events_for_date(self.selected_date);
		//do nothing if there isnt a selected event
		if let Some(Some(selected_event)) = self.selected_event.map(|e| events.get(e)){
			//prepare lines to go into the paragraph
			let event_info = vec![
				Some(Line::from(selected_event.title()).centered()),
				Some(Line::from("")),
				selected_event.description().map(Line::from),
				selected_event.location().map(|l| Line::from(format!("Location: {l}"))),
				Some(Line::from(Self::format_event_timespan(&selected_event))),
				Some(Line::from(format!("Calendar: {}",selected_event.parent_calendar_name()))),
			].into_iter().filter_map(|i| i).collect::<Vec<_>>();
			Paragraph::new(event_info)
				.block(event_info_block)
				.wrap(Wrap { trim: false })
				.render(area,buf);
		}else {
			event_info_block.render(area,buf);
		}
	}
	fn render_calendar_display(&mut self, area: Rect, buf: &mut Buffer){
		//block outline
		let calendar_display_block = Block::bordered()
			.title_bottom(Line::from(" =/- to change view ").centered())
			.title(Line::styled(" Calendar ",
				if self.selected_window == SelectedWindow::CalendarDisplay {STYLE_SELECTED_TEXT}
				else {Style::new()}
			).centered())
			.border_set(border::THICK);
		let calendar_display_block_rect = calendar_display_block.inner(area);
		calendar_display_block.render(area,buf);
		//calendar day layouts
		let calendar_day_layout = Layout::default()
			.direction(Direction::Horizontal)
			.constraints(vec![Constraint::Fill(1); self.calendar_view_size])
			.split(calendar_display_block_rect);
		for i in 0..(self.calendar_view_size){
			//the current day we are on in this iteration
			let this_day_selected = i == self.calendar_scroll_offset;
			//the date for the current day we are rendering
			let date = self.selected_date - Days::new(self.calendar_scroll_offset as u64) + Days::new(i as u64);
			//seperates individual days
			let day_block = Block::new()
				.borders(Borders::LEFT)
				.title_top(Line::from(date.format(DATE_FORMAT_STRING).to_string())
					.centered()
					.style(
						//highlight red if it is selected
						if this_day_selected {STYLE_SELECTED_TEXT} 
						else {Style::default()}
						.patch(
							//underline if it is todays date
							if date.is_today() {STYLE_HIGHLIGHTED_TEXT}
							else {Style::default()}
						)
					)
				);
			let list_item_width = calendar_day_layout[i].width as isize;
			let events = self.calendar
				.get_events_for_date(date);
			//for each hour fetch events
			let event_titles: Vec<_> = events
				.iter()
				.map(|event| vec![
					Line::from(format!("--- {} {}",
						Self::format_event_timespan(&event),
						"-".repeat(max(list_item_width-12,0) as usize)
					)),
					Line::from(event.title()),
					Line::from("")
				])
				.flatten() //will flatten [["00:00","event1",""],["01:00","event2",""]]
				.collect();
			let item_list = List::new(event_titles)
				.block(day_block)
				.highlight_style(STYLE_SELECTED_TEXT);
			//only the selected date gets the selected ListState
			let mut list_state = if this_day_selected && events.len() > 0 {
				ListState::default().with_selected(self.selected_event.map(|n| n*3 + 1))
			}else {
				ListState::default()
			};
			StatefulWidget::render(item_list,calendar_day_layout[i],buf,&mut list_state);
		}
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
				Constraint::Min(16),
				Constraint::Percentage(75),
			])
		);
		//====== date selector ======
		self.render_date_selector(top_rect,buf);
		//====== event info ======
		self.render_event_info(event_info_rect,buf);
		//====== calendar display ======
		self.render_calendar_display(calendar_display_rect,buf);
	}
}
