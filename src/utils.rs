#[macro_export]
macro_rules! say {
	($who:literal, $($arg:tt)*) => {{
		print!("[{}]: ", $who);
		println!($($arg)*);
	}};
	($who:ident, $($arg:tt)*) => {{
		print!("[{}]: ", $who);
		println!($($arg)*);
	}};
}

#[macro_export]
macro_rules! sprintln {
	($($arg:tt)*) => {
		$crate::say!("system", $($arg)*)
	};
}
