pub fn main() {
    // Required for LLVM 17. LLVM 18 doesn't need this.
    println!("cargo:rustc-link-arg=-export-dynamic");
}
