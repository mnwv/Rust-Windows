#[allow(unused_macros)]
macro_rules! hiword {
	($param:expr) => { (($param >> 16) & 0xFFFF) as i32};
}

#[allow(unused_macros)]
macro_rules! loword {
	($param:expr) => {$param as i32};
}
