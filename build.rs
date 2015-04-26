fn main()
{
	println!("cargo:rustc-link-search=native=./SharedSupport");
	println!("cargo:rustc-link-lib=dylib=alut.0");
}