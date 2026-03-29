use hello_android::MyApp;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "hello_android",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
