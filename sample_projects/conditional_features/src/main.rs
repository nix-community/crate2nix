fn main() {
	#[cfg(feature = "hello")]
	println!("Hello, {}!", optional::maybe_foo());

	#[cfg(all(not(feature = "hello"), feature = "foo"))]
	println!("Bye, foo!");

	#[cfg(all(not(feature = "hello"), not(feature = "foo")))]
	println!("Bye, not foo!");
}
