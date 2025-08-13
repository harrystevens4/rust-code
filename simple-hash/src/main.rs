use std::io;
use std::cmp;
use std::io::Read;
use std::env;

fn main() {
	let inputs = {
		let args = env::args().collect::<Vec<_>>();
		if args.len() > 1 {
			(&args[1..]).to_owned()
		}else {
			let mut input = String::new();
			let _ = io::stdin().read_to_string(&mut input);
			vec![input]
		}
	};
	inputs.into_iter().for_each(
		|i| println!("{}",hash(i.to_string())
			.iter()
			.map(|x| format!("{:x}",x))
			.collect::<String>()
		)
	)
}

fn rotate(input: &mut String,count: isize){
	if input.len() == 0 {return}
	let bound = 
		if count < 0 {
			input.len() - ((-count as usize) % input.len()) as usize
		}else {
			count as usize
		};
	input.extend_from_within(..(bound % input.len()));
	let _ = input.drain(..(bound % input.len()));
}

fn as_chunks<'a>(string: &'a String,chunk_size: usize) -> Vec<&'a str>{
	let mut chunks: Vec<&str> = vec![];
	for i in 0..(string.len().div_ceil(chunk_size)){
		let start = i * chunk_size;
		let end = cmp::min(start + chunk_size,string.len());
		chunks.push(&string[start..end]);
	}
	chunks
}

fn hash(input: String) -> [u8; 8] {
	let mut chunks: Vec<[u8; 8]> = vec![];
	let mut result = [0_u8; 8];
	//split into [u8; 8] chunks
	for (i,chunk) in as_chunks(&input,8).into_iter().enumerate(){
		let mut buffer = chunk.to_string();
		rotate(&mut buffer,i as isize);
		let str_chunk = <String as AsRef<[u8]>>::as_ref(&buffer);
		let mut processed_chunk = [0; 8];
		processed_chunk[..str_chunk.len()].copy_from_slice(str_chunk);
		chunks.push(processed_chunk);
	}
	//flatten all chunks by performing some random operations i made up and an xor
	for chunk in chunks {
		let new_result: Vec<u8> = chunk.iter().zip(result).map(|(x,y)| x ^ y ^ x.wrapping_pow(y.wrapping_add(2) as u32)).collect();
		result.copy_from_slice(&new_result[..]);
	}
	//more magic operations with the final array
	let mut magic_val = 42;
	for i in 0..8 {
		println!("{}",magic_val);
		magic_val = result[i].wrapping_add(magic_val).wrapping_mul(magic_val.wrapping_add(13));
		result[i] = magic_val;
	}
	result
}
