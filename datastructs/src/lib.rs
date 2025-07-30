//======================= binary search tree =======================
mod bst {
	use std::boxed::Box;

	pub struct Bst<T: std::cmp::PartialOrd>{
		root: Option<Box<BstNode<T>>>,
	}
	struct BstNode<T: std::cmp::PartialOrd>{
		left: Option<Box<BstNode<T>>>,
		right: Option<Box<BstNode<T>>>,
		data: T,
	}
	impl<T: std::cmp::PartialOrd> Bst<T>{
		pub fn new() -> Self {
			Self {
				root: None,
			}
		}
		pub fn add(&mut self, data: T){
			if self.root.is_none(){
				self.root = Some(Box::new(BstNode {
					left: None,
					right: None,
					data: data
				}))
			}else{
				Self::recursive_add(&mut self.root.as_mut().unwrap(),
					Box::new(BstNode {
						left: None,
						right: None,
						data: data
					}));
			}
		}
		fn recursive_add(node: &mut Box<BstNode<T>>, new_node: Box<BstNode<T>>){
			if new_node.data <= node.data {
				//add to the left
				match &mut node.left {
					Some(n) => Self::recursive_add(n,new_node),
					None => node.left = Some(new_node),
				}
			}else{
				//add to the right
				match &mut node.right {
					Some(n) => Self::recursive_add(n,new_node),
					None => node.right = Some(new_node),
				}
			}
		}
		pub fn as_vec(&self) -> Vec<&T>{
			 fn traverse<'a, T: std::cmp::PartialOrd>(bst_node: &'a Option<Box<BstNode<T>>>,output: &mut Vec<&'a T>){ //inorder
				if let Some(node) = bst_node {
					traverse(&node.left,output);
					output.push(&node.data);
					traverse(&node.right,output);
				}
			}
			let mut result = vec![];
			traverse(&self.root,&mut result);
			result
		}
	}
}
//======================= drop guard =======================
mod drop_guard {
	pub struct DropGuard<T: Fn()> {
		callback: T,
	}
	impl<T: Fn()> DropGuard<T> {
		fn drop(callback: T) -> Self {
			Self {
				callback: callback,
			}
		}
	}
	impl<T: Fn()> Drop for DropGuard<T> {
		fn drop(&mut self){
			(self.callback)();
		}
	}
}
//======================= tests =======================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bst_add_items(){
	    let mut bst = bst::Bst::new();
	    let mut test_data = vec![89_i32,34,56,3,78,67,90,200,100];
	    for val in test_data.clone() {
		    bst.add(val)
	    }
	    test_data.sort();
	    assert_eq!(
	    	test_data,
		bst.as_vec().into_iter().cloned().collect::<Vec<i32>>()
	);
    }
}
