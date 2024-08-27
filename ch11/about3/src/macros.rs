#[allow(unused_macros)]
macro_rules! hiword {
	($param:expr) => { (($param >> 16) & 0xFFFF) as u16};
}

#[allow(unused_macros)]
macro_rules! loword {
	($param:expr) => {($param & 0xFFFF) as u16};
}

#[allow(unused_macros)]
macro_rules! rgb {
	($r:expr, $g:expr, $b:expr) => {
		(($r & 0xFF) | ($g << 8) & 0xFF00 | ($b << 16) & 0xFF0000) as u32
	};
}

#[allow(unused_macros)]
macro_rules! rvalue {
	($colorref:expr) => {
		($colorref & 0xFF) as u8
	};
}

#[allow(unused_macros)]
macro_rules! gvalue {
	($colorref:expr) => {
		(($colorref >> 8) & 0xFF) as u8
	};
}

#[allow(unused_macros)]
macro_rules! bvalue {
	($colorref:expr) => {
		(($colorref >> 16) & 0xFF) as u8
	};
}

#[allow(unused_macros)]
macro_rules! makelong {
	($loword:expr, $hiword:expr) => { ($hiword << 16 & 0x7FFF0000) | ($loword & 0xFFFF) };
}
