use headless_chrome::{Browser, LaunchOptions};
fn main() {
    Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .build()
            .unwrap(),
    )
    .expect("TODO: panic message");
}
