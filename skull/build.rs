fn main() {
    if rustc_version::Channel::Nightly == rustc_version::version_meta().unwrap().channel {
        println!("cargo:rustc-cfg=nightly");
    }
}
