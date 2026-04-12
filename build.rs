fn main() {
    // Only build Slint UI on desktop targets; Android uses a headless stub.
    #[cfg(not(target_os = "android"))]
    slint_build::compile("ui/app.slint").expect("failed to compile Slint UI");
}
