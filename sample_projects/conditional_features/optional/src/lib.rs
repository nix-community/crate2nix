#[cfg(not(feature = "allow-build"))]
compile_error!("allow-build feature is not enabled, refusing to build");

pub fn maybe_foo() -> &'static str {
	#[cfg(feature = "foo")]
	{ "foo" }

	#[cfg(not(feature = "foo"))]
	{ "not foo" }
}
