#[derive(Debug,Clone)]
pub struct Image<'a> {
	width: usize,
	height: usize,
	pixel_array: &'a [u8],
}

impl<'a> Image<'a> {
	pub fn new(width: usize, height: usize, pixel_array: &'a mut [u8]) -> Self {
		println!("{}",pixel_array.len());
		Self {
			width, height, pixel_array
		}
	}
	pub fn scale(&mut self, sf: f32){
		if sf > 1.0 { panic!("cannot enlarge image") }
	}
	pub fn as_ascii(&self) -> String {
		//alloc now to save fragmented allocs
		let mut output = String::with_capacity(self.width*self.height);
		for i in 0..(self.width*self.height){
			//BGRx 
			//has a range of 0.0-1.0
			let brightness = (
				self.pixel_array[(i*4)+0] as usize
				+ self.pixel_array[(i*4)+1] as usize
				+ self.pixel_array[(i*4)+2] as usize
			) as f32 / (u8::MAX as usize*3_usize) as f32;
			const CHAR_BRIGHTNESS: [char; 66] = ['@','B','%','8','&','W','M','#','*','o','a','h','k','b','d','p','q','w','m','Z','O','0','L','C','J','U','Y','X','z','c','v','u','n','x','r','j','f','t','/','\\','|','(',')','1','{','}','?','-','_','+','~','<','>','i','!','l','I',';',':',',','\"','^','`',',','\'','.'];
			let position = ((CHAR_BRIGHTNESS.len()-1_usize) as f32 * brightness).round() as usize;
			output.push(CHAR_BRIGHTNESS[position]);
			//line break at the end of each row of pixels
			if i % self.width == 0 { output.push('\n') }
		}
		output
	}
}
