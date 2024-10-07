use headless_chrome::protocol::cdp::types::Event;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::Browser;

use std::sync::Arc;
use tracing::info;

const APP: &str = env!("CARGO_BIN_EXE_testing");

#[tokio::test]
async fn real() {
    tracing_subscriber::fmt::init();

    let mut app = std::process::Command::new(APP)
        .spawn()
        .expect("Failed to start server");
    info!("started app");

    let browser = Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    let addr = std::net::SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
        4044,
    );
    let url = format!("http://{addr}");
    tab.navigate_to(&url).unwrap();
    tab.wait_until_navigated().unwrap();
    tab.enable_log().unwrap();
    tab.add_event_listener(Arc::new(move |event: &Event| match event {
        Event::LogEntryAdded(log_event) => {
            info!("log_event: {log_event:?}");
        }
        _ => {}
    }))
    .unwrap();

    tab.wait_for_element("#email").unwrap().click().unwrap();
    info!("clicked email");
    tab.type_str("email").unwrap();
    info!("typed email");
    tab.wait_for_element("#password").unwrap().click().unwrap();
    info!("clicked password");
    tab.type_str("password").unwrap();
    info!("typed password");
    tab.wait_for_element("#register-btn")
        .unwrap()
        .click()
        .unwrap();
    info!("clicked register");

    std::thread::sleep(std::time::Duration::from_secs(10));

    app.kill().unwrap();
    app.wait().unwrap();
}
