use nix::poll::{poll,PollFd,PollFlags};
use std::io;
use std::io::{Read,Write,ErrorKind};
use termios::*;
use std::os::fd::{AsFd,AsRawFd};
use std::cell::RefCell;
use std::sync::Mutex;

struct InputHistory {
	buffer: Vec<Vec<char>>,
	index: usize,
}

impl InputHistory {
	fn new() -> Self {
		InputHistory {
			buffer: vec![],
			index: 0,
		}
	}
}

pub struct ThreadedIO {
	io_lock: Mutex<()>,
	//input_buffer: Mutex<RefCell<Vec<char>>>,
	current_prompt_state: Mutex<RefCell<String>>,
	old_term_settings: Termios,
	interupt: Mutex<bool>,
	history: Mutex<InputHistory>,
	pub handle_history: bool,
	pub handle_signals: bool,
}

enum EscapeCode {
	UpArrow,
	DownArrow,
	LeftArrow,
	RightArrow,
	Unknown
}

impl ThreadedIO {
	pub fn new() -> Self {
		Self::builder()
			.handle_history(false)
			.handle_signals(false)
			.build()
	}
	pub fn builder() -> Self {
		ThreadedIO {
			io_lock: Mutex::new(()),
			//input_buffer: Mutex::new(RefCell::new(vec![])),
			current_prompt_state: Mutex::new(RefCell::new("".to_string())),
			old_term_settings: Termios::from_fd(io::stdin().as_raw_fd()).unwrap(),
			interupt: Mutex::new(false),
			history: Mutex::new(InputHistory::new()),
			handle_history: false,
			handle_signals: false,
		}
	}
	pub fn build(self) -> Self {
		//====== setup raw stdin ======
		let mut term = self.old_term_settings.clone();
		let mut lflags = ICANON | ECHO;
		if self.handle_signals { lflags |= ISIG }
		term.c_lflag &= !(lflags); //unbuffered no echo
		term.c_cc[VMIN] = 1; //get at least one byte before read returns
		term.c_cc[VTIME] = 0; //dont wait for bytes
		tcsetattr(io::stdin().as_raw_fd(),TCSANOW,&term).unwrap();
		//init history (used even if history is disabled)
		self.history_new_entry_empty();
		//return
		self
	}
	pub fn handle_history(mut self,handle_history_setting: bool) -> Self
	//the history mechanism of storing the buffer will still be used if disabled,
	//but no new history items will be created after hitting enter, effectively overwriting the 
	//previous prompt, and meaning history_next and history_prev will default to doing nothing
		{ self.handle_history = handle_history_setting; self }
	pub fn handle_signals(mut self, handle_signals_setting: bool) -> Self
		{ self.handle_signals = handle_signals_setting; self }
	pub fn println(&self,string: String) -> Result<(),std::io::Error>{
		let _io_guard = self.io_lock.lock();
		let current_prompt_state_binding = self.current_prompt_state.lock().unwrap();
		let current_prompt_state = current_prompt_state_binding.borrow();
		let mut stdout = io::stdout();
		//delete old prompt and insert line
		stdout.write_all(format!("\r\x1b[2K{}\n",string).as_bytes())?;
		//redisplay the prompt
		stdout.write_all(current_prompt_state.as_bytes())?;
		stdout.flush()?;
		Ok(())
	}
	pub fn input(&self,prompt: &str) -> Result<String,std::io::Error>{
		{//reset interupt
			*self.interupt.lock().unwrap() = false;
		}
		{//====== initialy display the prompt ======
			let _io_guard = self.io_lock.lock();
			let current_prompt_state_binding = self.current_prompt_state.lock().unwrap();
			let mut current_prompt_state = current_prompt_state_binding.borrow_mut();
			*current_prompt_state = prompt.to_string();
			let mut stdout = io::stdout();
			stdout.write_all(format!("\r\x1b[2K{}",current_prompt_state).as_bytes())?;
			stdout.flush()?;
		}
		//====== poll wrapper that allows interuption ======
		let wait_for_stdin = move |timeout|{
			let stdin = io::stdin();
			let mut pollfd = [PollFd::new(stdin.as_fd(),PollFlags::POLLIN)];
			//====== wait for data ======
			loop {
				if poll::<u16>(&mut pollfd,timeout)? >= 1 {break}
				if* self.interupt.lock().expect("Mutex poisoned: fatal") == true {return Err(io::Error::from(ErrorKind::Interrupted))}
			}
			io::Result::<()>::Ok(())
		};
		//====== get input bytes ======
		//grab definitions of special characters
		let sp = &self.old_term_settings.c_cc;
		//wait
		wait_for_stdin(50)?;
		for ch in io::stdin().bytes(){
			match ch?{
				10 => break,//enter
				127 => {self.history_current_pop(); ()}, //delete
				val if val == sp[VINTR] || val == sp[VQUIT] => return Err(ErrorKind::Interrupted.into()), //ctrl-c
				val if val == sp[VEOF] => break,//ctrl-d
				27 => { //escape codes
					match self.get_escape_sequence()?{
						EscapeCode::UpArrow => if self.handle_history {
							let _ = self.history_prev();
						},

						EscapeCode::DownArrow => if self.handle_history {
							let _ = self.history_next();
						},
						_ => (),
					};
				}
				ch => {
					if ch >= 32 && ch <= 126{
						self.history_current_push(char::from(ch));
					}else{
						self.println(format!("unknown char {}",ch))?;
					}
				},
			}
			{//====== display the prompt ======
				let _io_guard = self.io_lock.lock();
				let current_prompt_state_binding = self.current_prompt_state.lock().unwrap();
				let mut current_prompt_state = current_prompt_state_binding.borrow_mut();
				*current_prompt_state = prompt.to_string() + &self
					.history_get_current()
					.iter()
					.collect::<String>();
				let mut stdout = io::stdout();
				stdout.write_all(format!("\r\x1b[2K{}",current_prompt_state).as_bytes())?;
				stdout.flush()?;
			}
			//====== wait for data ======
			wait_for_stdin(50)?;
		}
		{//====== clear the input buffer ======
			let _io_guard = self.io_lock.lock();
			let current_prompt_state_binding = self.current_prompt_state.lock().unwrap();
			let mut current_prompt_state = current_prompt_state_binding.borrow_mut();
			*current_prompt_state = "".to_string();
		}
		let message = self.history_get_current().iter().collect();
		//or create a new empty entry and use that next if history is on
		if self.handle_history {self.history_new_entry_empty()}
		//simply reset the current history buffer to empty if history is off
		else {self.history_set_current(vec![])}
		Ok(message)
	}
	pub fn interupt_input(&self){
		let mut lock = self.interupt.lock().unwrap();
		*lock = true;
	}
	pub fn reset_term(&self){
		tcsetattr(io::stdin().as_raw_fd(),TCSANOW,&self.old_term_settings).unwrap();
	}
	fn get_escape_sequence(&self) -> io::Result<EscapeCode>{
		let getch = || io::stdin()
				.bytes()
				.next()
				.ok_or(ErrorKind::UnexpectedEof)?;
		Ok(match getch()? {
			0x9b => match getch()?{ //control sequence introducer
				_ => EscapeCode::Unknown,
			},
			0x5b => match getch()?{//arrow keys
				0x41 => EscapeCode::UpArrow,
				0x42 => EscapeCode::DownArrow,
				0x43 => EscapeCode::RightArrow,
				0x44 => EscapeCode::LeftArrow,
				_ => EscapeCode::Unknown,
			},
			_ => EscapeCode::Unknown,
		})
	}
	fn history_next(&self){
		let mut history = self.history.lock().unwrap();
		if history.index == history.buffer.len()-1 { //already the latest
		}else{
			history.index += 1;
			history.buffer[history.index].clone();
		}
		
	}
	fn history_prev(&self){
		let mut history = self.history.lock().unwrap();
		if history.index == 0 { //already the latest
		}else{
			history.index -= 1;
			history.buffer[history.index].clone();
		}
	}
	fn history_set_current(&self, new: Vec<char>){
		let mut history = self.history.lock().unwrap();
		let current_index = history.index;
		history.buffer[current_index] = new;
	}
	//also sets index to this new entry
	fn history_new_entry_empty(&self){
		let mut history = self.history.lock().unwrap();
		history.buffer.push(vec![]);
		history.index = history.buffer.len()-1;
	}
	fn history_current_push(&self, ch: char){
		let mut history = self.history.lock().unwrap();
		let current_index = history.index;
		history.buffer[current_index].push(ch);
	}
	fn history_current_pop(&self){
		let mut history = self.history.lock().unwrap();
		let current_index = history.index;
		let _ = history.buffer[current_index].pop();
	}
	fn history_get_current(&self) -> Vec<char>{
		let history = self.history.lock().unwrap();
		let current_index = history.index;
		history.buffer[current_index].clone()
	}
}
impl Drop for ThreadedIO{
	fn drop(&mut self){
		self.reset_term();
	}
}
