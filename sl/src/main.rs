use std::path::Path;
use std::ffi::OsString;
use std::env;

#[link(name = "term")]
unsafe extern "C" {
	fn get_term_width() -> i32;
}

fn main() {
	let dirs = env::args()
		.enumerate()
		.filter(|&(i,_)| i != 0) //skip argv[0]
		.map(|x| x.1)
		.collect::<Vec<_>>();
	for arg in dirs{
		let dir = Path::new(&arg);
		println!("{}",match dir.file_name().unwrap_or(&OsString::from("..")).to_str(){
			Some(name) => name,
			None => {eprintln!("Path has bad unicode data"); continue},
		});
		if dir.exists() {print_dir(dir,"".into())}
	}
}
fn print_dir(path: &Path, indent: String){
	let width = unsafe {get_term_width()};
	let items = match path.read_dir(){
		Ok(d) => d,
		Err(e) => {eprintln!("{:?} - {:?}",path.file_name().unwrap_or(&OsString::from("..")),e); return},
	}.into_iter().collect::<Vec<_>>();
	if items.len() > 0 {
		//===== iterate over items in dir ======
		for (i,item) in items.iter().enumerate() {
			match item {
				Ok(item) => {
					//====== step into item if it is a dir ======
					let item_path = item.path();
					let file_name = match item.file_name().into_string(){
						Ok(name) => name,
						Err(e) => {eprintln!("{:?}: Bad unicode data",e); String::new()},
					};
					//====== print the ones with ├ ======
					if i != items.len()-1 {print_clamped(format!("{}├─{}",&indent,file_name),width)}
					//====== the last one should use └ ======
					else {print_clamped(format!("{}└─{}",&indent,file_name),width)}
					if item_path.is_dir() {print_dir(&item_path.as_path(),indent.clone()+
						if i == items.len()-1 {"  "}
						else {"│ "}
					)}
				},
				Err(e) => eprintln!("{:?}",e),
			}
		}
	}
}
fn print_clamped(string: String, width: i32){
	println!("{}",string.chars().take(width as usize).collect::<String>())
}
