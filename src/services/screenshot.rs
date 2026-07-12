use ashpd::desktop::screenshot::Screenshot;
use ashpd::WindowIdentifier;
use gtk4::prelude::*;

pub async fn take_screenshot(window: &gtk4::ApplicationWindow) -> Result<String, ScreenshotError> {
    let proxy = Screenshot::new().await?;

    let window_id =
        WindowIdentifier::from_native(&window.native().expect("No native window")).await;

    let response = proxy
        .screenshot(
            &window_id, false, // interactive
            false, // modal
        )
        .await?;

    Ok(response.uri().to_string())
}

pub async fn pick_color(window: &gtk4::ApplicationWindow) -> Result<(), ScreenshotError> {
    // Можно использовать Color picker portal для доп. функциональности
    todo!()
}
