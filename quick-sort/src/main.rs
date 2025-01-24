fn main() {
	let data = vec![2,7,4,6,1,4,3,9,7,5];
	let sorted_data = quick_sort(data);
	println!("{:?}",sorted_data)
}
fn quick_sort(data_vec: Vec<isize>) -> Vec<isize> {
	if data_vec.len() <= 1 {
		return data_vec;
	}
	let mut data_vec_clone = data_vec.clone();
	let pivot: isize = data_vec_clone.pop().expect("vector empty");
	let mut lower_half: Vec<isize> = vec![];
	let mut upper_half: Vec<isize> = vec![];
	for data in data_vec_clone {
		if data < pivot {
			lower_half.push(data)
		} else{
			upper_half.push(data)
		}
	}
	println!("{:?} {} {:?}",lower_half,pivot,upper_half);
	let mut sorted_data = quick_sort(lower_half.clone());
	sorted_data.push(pivot);
	sorted_data.append(&mut quick_sort(upper_half.clone()));
	return sorted_data;
}
