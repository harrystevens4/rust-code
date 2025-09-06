//use cc;
fn main(){
	println!("cargo::rerun-if-changed=src/term.c");
	cc::Build::new().file("src/term.c").compile("term")
}
