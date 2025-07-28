use tio::ThreadedIO;
use std::process::{Command,Stdio,ExitStatus,ExitCode};
use std::thread;
use std::sync::{Arc,Mutex};
use std::env::args;
use std::io::{Read,Write,Result,Error};
use std::time::Duration;

//======================== structs ======================
struct SyncedHalt {
	halt: Mutex<bool>
}
impl SyncedHalt {
	fn new() -> Self {
		let instance = SyncedHalt {
			halt: Mutex::new(false),
		};
		instance
	}
	fn halt(&self){
		*self.halt.lock().unwrap() = true;
	}
	fn halted(&self) -> bool {
		let res = *self.halt.lock().unwrap();
		res
	}
	//returns a guard, that when dropped will set halt
	fn guard(&self) -> SyncedHaltGuard<'_> {
		SyncedHaltGuard {
			halt: &self.halt,
		}
	}
}
struct SyncedHaltGuard<'a>{
	halt: &'a Mutex<bool>
}
impl Drop for SyncedHalt {
	fn drop(&mut self){
		*self.halt.lock().unwrap() = true;
	}
}

//====================== functions =======================
fn main() -> Result<ExitCode>{
	let synced_halt = Arc::new(SyncedHalt::new());
	let threaded_io = Arc::new(ThreadedIO::new());
	//====== spawn process ======
	let mut args_iter = args();
	//skip our argv[0]
	let _ = args_iter.next();
	let mut process = Command::new(args_iter.next().ok_or(Error::other("No arguments provided"))?)
		.args(args_iter)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.spawn()?;
	//====== input thread ======
	let input_thread_handle = {
		let mut stdin = process.stdin.take().unwrap();
		let io = threaded_io.clone();
		let halt = synced_halt.clone();
		thread::spawn(move ||{
			//will halt when dropped
			let _halt_guard = halt.guard();
			loop {
				let input = io.input(">>>")?;
				stdin.write_all(input.as_bytes())?;
				if halt.halted() {
					return Ok::<(),Error>(())
				}
			}
		})
	};
	//====== output thread ======
	let output_thread_handle = {
		let mut stdout = process.stdout.take().unwrap();
		let io = threaded_io.clone();
		let halt = synced_halt.clone();
		thread::spawn(move ||{
			//will halt when dropped
			let _halt_guard = halt.guard();
			loop {
				let output = read_line(&mut stdout)?;
				io.println(output)?;
				if halt.halted() {
					io.interupt_input();
					return Ok::<(),Error>(())
				}
			}
		})
	};
	//====== wait for child process to exit ======
	let exit_status = process.wait_with_output()?.status;
	synced_halt.halt();
	threaded_io.interupt_input();
	let _ = input_thread_handle.join();
	let _ = output_thread_handle.join();
	Ok(ExitCode::from(exit_status.code().unwrap_or(0) as u8))
}

fn read_line<T: Read>(stream: &mut T) -> Result<String> {
	//read one byte untill we hit a newline
	let mut buffer = String::new();
	loop {
		//get byte
		let mut byte = [0; 1];
		//break if eof
		if stream.read(&mut byte)? == 0 {break};
		let ch = char::from(byte[0]);
		//break when end of transmition or newline sent
		if ch == '\n' || ch == '\x04' {break}
		//add byte to buffer
		buffer.push(ch);
	}
	Ok(buffer)
}
