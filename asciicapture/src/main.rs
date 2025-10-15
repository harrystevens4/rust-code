use portal_screencast::*;
use std::io::Error;
use pipewire::{main_loop::*,properties::*};
fn main() -> Result<(), Box<dyn std::error::Error>>{
	//request screen cast
	let mut screen_cast = ScreenCast::new()?;
	screen_cast.set_source_types(SourceType::MONITOR);
	let screen_cast = screen_cast.start(None)?; //i looooove raii
	let pipewire_node = screen_cast
		.streams()
		.next()
		.ok_or(Error::other("No streams provided"))?
		.pipewire_node();
	//setup pipewire
	let properties = properties!{ //i just bought a property in egypt
		"PW_KEY_TARGET_OBJECT" => pipewire_node.to_string(),
	};
	let mainloop = MainLoopBox::new(Some(properties.as_ref()))?;
	mainloop.run();
	Ok(())
}
