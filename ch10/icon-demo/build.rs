use windres::Build;

fn main() {
	std::env::set_var("INCLUDE", 
		// "C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\MSVC\\14.39.33519\\include;\
		// C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\MSVC\\14.39.33519\\ATLMFC\\include;\
		// C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Auxiliary\\VS\\include;\
		// C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\ucrt;\
		// C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\um;\
		// C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\shared;\
		// C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\winrt;\
		// C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\cppwinrt;\
		// C:\\Program Files (x86)\\Windows Kits\\NETFXSDK\\4.8\\include\\um"
		"C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\um;\
		C:\\Program Files (x86)\\Windows Kits\\10\\include\\10.0.22621.0\\shared;"

	);
	Build::new().compile("icondemo.rc").unwrap();
}