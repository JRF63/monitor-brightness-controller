fn main() {
    println!("cargo:rustc-link-arg-bins=resources.res");
    println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg-bins=/MANIFESTINPUT:application.manifest");
}