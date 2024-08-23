use windres::Build;

fn main() {
	std::env::set_var("INCLUDE", 
		r"C:\Program Files (x86)\Windows Kits\10\include\10.0.22621.0\um;\
		C:\Program Files (x86)\Windows Kits\10\include\10.0.22621.0\shared;"
	);
	Build::new().compile("menudemo.rc").unwrap();
}