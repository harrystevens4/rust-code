#[derive(Debug,Clone)]
pub struct Image<'a> {
	width: usize,
	height: usize,
	pixel_array: &'a [u8],
}

impl<'a> Image<'a> {
	pub fn new(width: usize, height: usize, pixel_array: &'a mut [u8]) -> Self {
		Self {
			width, height, pixel_array
		}
	}
	pub fn scale(&mut self, sf: f32){
		if sf > 1.0 { panic!("cannot enlarge image") }
	}
	pub fn as_ascii(&self) -> String {
		//alloc now to save fragmented allocs
		let output = String::with_capacity(self.width*self.height);
		for i in 0..(self.width*self.height){
		}
		output
	}
}
