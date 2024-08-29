use windres::Build;

fn main() {
	std::env::set_var("INCLUDE",
		concat!(
		r"C:\Program Files (x86)\Windows Kits\10\include\10.0.22621.0\um;",
		r"C:\Program Files (x86)\Windows Kits\10\include\10.0.22621.0\shared;")
	);
	let key = "INCLUDE";
	match std::env::var(key) {
		Ok(val) => println!("{key}: {val:?}"),
		Err(e) => println!("couldn't interpret {key}: {e}"),
	}
	Build::new().compile("hexcalc.rc").unwrap();
}