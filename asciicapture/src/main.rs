use portal_screencast::*;
use std::os::fd::{OwnedFd,FromRawFd};
use std::io::Error;
use libspa::{pod::*,utils::*,param::*};
use pipewire::{main_loop::*,properties::*,stream::*,context::*};

mod images;
use images::Image;

//this is what we pass to the stream callback
struct UserData {
	format: video::VideoInfoRaw, //for format negotiation
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
	//====== request screen cast ======
	let mut screen_cast = ScreenCast::new()?;
	screen_cast.set_source_types(SourceType::MONITOR);
	let screen_cast = screen_cast.start(None)?; //i looooove raii
	let pipewire_node = screen_cast
		.streams()
		.next()
		.ok_or(Error::other("No streams provided"))?
		.pipewire_node();
	//====== grab the terminal size ======
	let (term_width,term_height) = term_size::dimensions().unwrap_or((500,500));
	//====== setup pipewire ======
	let pipewire_fd = unsafe { OwnedFd::from_raw_fd(screen_cast.pipewire_fd()) };
	let properties = PropertiesBox::new(); //i just bought a property in egypt
	let mainloop = MainLoopBox::new(Some(properties.as_ref()))?;
	let context = ContextBox::new(&mainloop.loop_(),None)?;
	let core = context.connect_fd(pipewire_fd,None)?;
	//====== connect to the stream ======
	let user_data = UserData {
		format: Default::default(),
	};
	let stream_properties = PropertiesBox::new();
	let stream = StreamBox::new(&core,"screen-capture",stream_properties)?;
	//====== format information ======
	let obj = Object {
		type_: SpaTypes::ObjectParamFormat.as_raw(),
		id: ParamType::EnumFormat.as_raw(),
		properties: vec![
			property!(
				libspa::param::format::FormatProperties::MediaType,
				Id,
				libspa::param::format::MediaType::Video
			),
			property!(
				libspa::param::format::FormatProperties::MediaSubtype,
				Id,
				libspa::param::format::MediaSubtype::Raw
			),
			//property!(
			//	libspa::param::format::FormatProperties::VideoFormat,
			//	Id,
			//	//i guess this is the only accepted format???
			//	libspa::param::video::VideoFormat::BGRx
			//),
		],
	};
	let values: Vec<u8> = serialize::PodSerializer::serialize(
	 	std::io::Cursor::new(Vec::new()),
	 	&Value::Object(obj),
	).unwrap().0.into_inner();
	let mut params = [Pod::from_bytes(&values).unwrap()];
	//====== setup stream callbacks ======
	//when this object is dropped so are the callbacks so keep it in scope
	let _listener = stream
		.add_local_listener_with_user_data(user_data)
		//format negotiation
		.param_changed(|_, user_data, _id, param| {
			//none is to clear the format
			if param.is_none() { return }
			let param = param.unwrap();
			// --- --- --- https://gitlab.freedesktop.org/pipewire/pipewire-rs/-/raw/main/pipewire/examples/audio-capture.rs?ref_type=heads --- --- ---
			let (media_type, media_subtype) = match format_utils::parse_format(param) {
            	Ok(v) => v,
            	Err(_) => return,
            };
			println!("param changed to (media type, media subtype): {:?} {:?}",media_type,media_subtype);
            //only accept raw video
			use libspa::param::format::*;
            if media_type != MediaType::Video || media_subtype != MediaSubtype::Raw {
            	return;
            }
			//initialise our user_data's format with the format pipewire gave us
			//this fills in the width and height for us too
            user_data
            	.format
            	.parse(param)
            	.expect("Failed to parse new video format");
			// --- --- --- end --- --- ---
			println!("{:?}",user_data.format);
		})
		//new data just dropped
		.process(move |stream, user_data| match stream.dequeue_buffer(){
			None => println!("stream queue empty"),
			Some(mut buffer) => {
				let available_data = buffer.datas_mut(); //&mut [Data]
				if available_data.is_empty() { return }
				//in theory there should only be _one peice_ of data
				if let Some(pixel_array) = available_data[0].data() {
					//====== resize and output the image ======
					let width = user_data.format.size().width as usize;
					let height = user_data.format.size().height as usize;
					let mut image = Image::new(
						width,
						height,
						pixel_array
					);
					//to fit terminal
					image.scale(
						(term_width-1) as f32 / width as f32,
						(term_height-1) as f32 / height as f32
					);
					//clear screen then print
					print!("\x1b[3J{}",image.as_ascii());
				}
			}
		})
		.register()?;
	//====== connect to the stream ======
	stream.connect(
		Direction::Input,
		Some(pipewire_node),
		StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS | StreamFlags::AUTOCONNECT,
		&mut params
	)?;
	println!("connected");
	println!("Note: BGRx is the only supported video format");
	//====== continuously run the mainloop ======
	mainloop.run();
	Ok(())
}
