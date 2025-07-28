use nix::poll::{poll,PollFd,PollFlags};
use std::io;
use std::io::{Read,Write,ErrorKind};
use termios::*;
use std::os::fd::{AsFd,AsRawFd};
use std::cell::RefCell;
use std::sync::Mutex;

pub struct ThreadedIO {
	io_lock: Mutex<()>,
	input_buffer: Mutex<RefCell<Vec<char>>>,
	current_prompt_state: Mutex<RefCell<String>>,
	old_term_settings: Termios,
	interupt: Mutex<bool>,
}

impl ThreadedIO {
	pub fn new() -> ThreadedIO{
		let instance = ThreadedIO {
			io_lock: Mutex::new(()),
			input_buffer: Mutex::new(RefCell::new(vec![])),
			current_prompt_state: Mutex::new(RefCell::new("".to_string())),
			old_term_settings: Termios::from_fd(io::stdin().as_raw_fd()).unwrap(),
			interupt: Mutex::new(false),
		};
		//====== setup raw stdin ======
		let mut term = instance.old_term_settings.clone();
		term.c_lflag &= !(ICANON | ECHO | ISIG); //unbuffered no echo
		term.c_cc[VMIN] = 1; //get at least one byte before read returns
		term.c_cc[VTIME] = 0; //dont wait for bytes
		tcsetattr(io::stdin().as_raw_fd(),TCSANOW,&term).unwrap();
		//return
		instance
	}
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

		let input_buffer_binding = self.input_buffer.lock().unwrap();
		let mut input_buffer = input_buffer_binding.borrow_mut();
		{//====== initialy display the prompt ======
			let _io_guard = self.io_lock.lock();
			let current_prompt_state_binding = self.current_prompt_state.lock().unwrap();
			let mut current_prompt_state = current_prompt_state_binding.borrow_mut();
			*current_prompt_state = prompt.to_string() + &input_buffer.iter().collect::<String>();
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
				127 => {input_buffer.pop(); ()}, //delete
				val if val == sp[VINTR] || val == sp[VQUIT] => return Err(ErrorKind::Interrupted.into()), //ctrl-c
				val if val == sp[VEOF] => break,//ctrl-d
				ch => {
					if ch >= 32 && ch <= 126{
						input_buffer.push(char::from(ch));
					}else{
						//self.println(format!("unknown char {}",ch))?;
					}
				},
			}
			{//====== display the prompt ======
				let _io_guard = self.io_lock.lock();
				let current_prompt_state_binding = self.current_prompt_state.lock().unwrap();
				let mut current_prompt_state = current_prompt_state_binding.borrow_mut();
				*current_prompt_state = prompt.to_string() + &input_buffer.iter().collect::<String>();
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
		let message = input_buffer.iter().collect();
		input_buffer.truncate(0);
		Ok(message)
	}
	pub fn interupt_input(&self){
		let mut lock = self.interupt.lock().unwrap();
		*lock = true;
	}
	pub fn reset_term(&self){
		tcsetattr(io::stdin().as_raw_fd(),TCSANOW,&self.old_term_settings).unwrap();
	}
}
impl Drop for ThreadedIO{
	fn drop(&mut self){
		self.reset_term();
	}
}
