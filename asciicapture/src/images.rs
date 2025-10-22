#[derive(Debug,Clone)]
pub struct Image {
	width: usize,
	height: usize,
	pixel_array: Vec<u8>,
}

impl Image {
	pub fn new(width: usize, height: usize, pixel_array: &[u8]) -> Self {
		//println!("{}",pixel_array.len());
		let mut array = Vec::with_capacity(width*height*4);
		array.extend(pixel_array);
		Self {
			width,
			height,
			pixel_array: array,
		}
	}
	pub fn scale(&mut self, sf_width: f32, sf_height: f32) {
		if sf_width > 1.0 || sf_height > 1.0 { panic!("cannot enlarge image") }
		let new_width = (self.width as f32) * sf_width;
		let new_height = (self.height as f32) * sf_height;
		let mut new_pixel_array: Vec<u8> = vec![0; new_width as usize * new_height as usize * 4];
		//====== sample each new pixel from where its equivalent would be in the old array ======
		for y in 0..(new_height as usize) {
			for x in 0..(new_width as usize) {
				let old_x = (x as f32 / sf_width).round() as usize;
				let old_y = (y as f32 / sf_height).round() as usize;
				let old_i = old_x + (old_y * self.width); //index into old array
				let i = x + (y * new_width as usize); //index into new array
				//sample where our pixel would have been
				new_pixel_array[(i*4)+0] = self.pixel_array[(old_i*4)+0]; //G
				new_pixel_array[(i*4)+1] = self.pixel_array[(old_i*4)+1]; //B
				new_pixel_array[(i*4)+2] = self.pixel_array[(old_i*4)+2]; //R
			}
		}
		self.width = new_width as usize; self.height = new_height as usize;
		self.pixel_array = new_pixel_array;
	}
	pub fn as_ascii(&self) -> String {
		//alloc now to save fragmented allocs
		let mut output = String::with_capacity(self.width*self.height);
		for i in 0..(self.width*self.height){
			//BGRx 
			let b = self.pixel_array[(i*4)+0] as usize;
			let g = self.pixel_array[(i*4)+1] as usize;
			let r = self.pixel_array[(i*4)+2] as usize;
			//has a range of 0.0-1.0
			let brightness = (
				r+g+b
			) as f32 / (u8::MAX as usize*3_usize) as f32;
			//67 xD
			const CHAR_BRIGHTNESS: [char; 67] = ['@','B','%','8','&','W','M','#','*','o','a','h','k','b','d','p','q','w','m','Z','O','0','L','C','J','U','Y','X','z','c','v','u','n','x','r','j','f','t','/','\\','|','(',')','1','{','}','?','-','_','+','~','<','>','i','!','l','I',';',':',',','\"','^','`',',','\'','.',' '];
			let position = ((CHAR_BRIGHTNESS.len()-1_usize) as f32 * (1.0-brightness)).round() as usize;
			let colour_sequence = format!("\x1b[38;2;{r};{g};{b}m");
			output.push_str(&colour_sequence);
			output.push(CHAR_BRIGHTNESS[position]);
			//line break at the end of each row of pixels
			if i % self.width == 0 { output.push('\n') }
		}
		//end colour
		output.push_str("\x1b[0m");
		output
	}
}
