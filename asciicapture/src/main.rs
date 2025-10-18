use portal_screencast::*;
use std::os::fd::{OwnedFd,FromRawFd};
use std::io::Error;
use libspa::{pod::*,utils::*,param::*};
use pipewire::{main_loop::*,properties::*,stream::*,context::*};

//using this to pass all our variables around callbacks and things
struct UserData {
	format: video::VideoInfoRaw, //for format negotiation
	cursor_move: bool, //if the cursor for the stream position has moved
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
	//====== setup pipewire ======
	let pipewire_fd = unsafe { OwnedFd::from_raw_fd(screen_cast.pipewire_fd()) };
	let properties = PropertiesBox::new(); //i just bought a property in egypt
	let mainloop = MainLoopBox::new(Some(properties.as_ref()))?;
	let context = ContextBox::new(&mainloop.loop_(),None)?;
	let core = context.connect_fd(pipewire_fd,None)?;
	//====== connect to the stream? idk what im doing ======
	let mut user_data = UserData {
		format: Default::default(),
		cursor_move: false,
	};
	let stream_properties = PropertiesBox::new();
	let stream = StreamBox::new(&core,"screen-capture",stream_properties)?;
	//let mut supported_formats = object!{
	//	libspa::utils::SpaTypes::ObjectParamFormat,
	//	libspa::param::ParamType::EnumFormat,
	//	property!(
	//		libspa::param::format::FormatProperties::MediaType,
	//		Id,
	//		libspa::param::format::MediaType::Video
	//	),
	//	property!(
	//		libspa::param::format::FormatProperties::MediaSubtype,
	//		Id,
	//		libspa::param::format::MediaSubtype::Raw
	//	),
	//};
	//====== properties??? ======
	let mut video_info = video::VideoInfoRaw::new();
	video_info.set_format(video::VideoFormat::RGB);
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
			//	libspa::param::video::VideoFormat::RGB
			//),
		],
	};
	let values: Vec<u8> = serialize::PodSerializer::serialize(
	 	std::io::Cursor::new(Vec::new()),
	 	&Value::Object(obj),
	)
	.unwrap()
	.0
	.into_inner();
	let mut params = [Pod::from_bytes(&values).unwrap()];
	//====== setup stream callbacks ======
	let listener = stream
		.add_local_listener_with_user_data(user_data)
		//format negotiation
		.param_changed(|_, user_data, id, param| {
			//none is to clear the format
			if param.is_none() { return }
			let param = param.unwrap();
			// --- --- --- https://gitlab.freedesktop.org/pipewire/pipewire-rs/-/raw/main/pipewire/examples/audio-capture.rs?ref_type=heads --- --- ---
			let (media_type, media_subtype) = match format_utils::parse_format(param) {
            	Ok(v) => v,
            	Err(_) => return,
            };
			println!("param changed to (media type, media subtype): {:?} {:?}",media_type,media_subtype);
            // only accept raw audio
			use libspa::param::format::*;
            if media_type != MediaType::Video || media_subtype != MediaSubtype::Raw {
            	return;
            }
            // call a helper function to parse the format for us.
            user_data
            	.format
            	.parse(param)
            	.expect("Failed to parse param changed to AudioInfoRaw");
			// --- --- --- end --- --- ---
			println!("{:?}",user_data.format);
		})
		//new data just dropped
		.process(|stream, user_data| match stream.dequeue_buffer(){
			None => println!("stream queue empty"),
			Some(mut buffer) => {
				println!("data :)");
			}
		})
		.register()?;
	//====== finaly ======
	stream.connect(
		Direction::Input,
		Some(pipewire_node),
		StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS | StreamFlags::AUTOCONNECT,
		&mut params
	)?;
	println!("connected");
	//====== continuously run the mainloop ======
	mainloop.run();
	Ok(())
}
