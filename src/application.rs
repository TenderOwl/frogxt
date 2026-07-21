/* MIT License
 *
 * Copyright (c) 2025 Andrey Maksimov
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * SPDX-License-Identifier: MIT
 */

use adw::{prelude::*, subclass::prelude::*};
use ashpd::WindowIdentifier;
use gettextrs::gettext;
use gtk::glib::clone;
use gtk::{gdk, gio, glib};
use imageproc::image;

use crate::config::VERSION;
use crate::services::ocr::OcrEngine;
use crate::FrogWindow;

mod imp {
    use std::cell::RefCell;
    use std::sync::mpsc;

    use super::*;

    #[derive(Debug, Default)]
    pub struct FrogxtApplication {
        pub tray_rx: RefCell<Option<mpsc::Receiver<crate::tray::TrayMessage>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FrogxtApplication {
        const NAME: &'static str = "FrogxtApplication";
        type Type = super::FrogxtApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for FrogxtApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<control>q"]);
        }
    }

    impl ApplicationImpl for FrogxtApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = application.active_window().unwrap_or_else(|| {
                let window = FrogWindow::new(&*application);
                window.upcast()
            });

            // Ask the window manager/compositor to present the window
            window.present();
        }

        fn startup(&self) {
            self.parent_startup();

            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/com/tenderowl/frog/general.css");
            gtk::style_context_add_provider_for_display(
                &gdk::Display::default().expect("Could not connect to a display."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            init_tessdata()
                .map_err(|_| {
                    tracing::error!("Failed to initialize tessdata");
                    self.obj()
                        .show_toast("Failed to initalize language models.");
                })
                .ok();

            // Spawn tray icon
            let (tx, rx) = mpsc::channel();
            self.tray_rx.replace(Some(rx));
            crate::tray::spawn_tray(tx);

            // Poll tray messages
            let app = self.obj().clone();
            glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                if let Some(rx) = app.imp().tray_rx.borrow().as_ref() {
                    while let Ok(msg) = rx.try_recv() {
                        match msg {
                            crate::tray::TrayMessage::OpenWindow => {
                                if let Some(window) = app.active_window() {
                                    window.set_visible(true);
                                    window.present();
                                } else {
                                    let window = crate::window::FrogWindow::new(
                                        app.upcast_ref::<gtk::Application>(),
                                    );
                                    window.present();
                                }
                            }
                            crate::tray::TrayMessage::GrabScreenshot => {
                                app.take_screenshot();
                            }
                            crate::tray::TrayMessage::OpenFile => {
                                app.select_file();
                            }
                            crate::tray::TrayMessage::Quit => {
                                app.quit();
                            }
                        }
                    }
                }
                glib::ControlFlow::Continue
            });
        }
    }

    impl GtkApplicationImpl for FrogxtApplication {}
    impl AdwApplicationImpl for FrogxtApplication {}
}

glib::wrapper! {
    pub struct FrogxtApplication(ObjectSubclass<imp::FrogxtApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl FrogxtApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        tracing::debug!("Setting up application with id '{}'", application_id);
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/com/tenderowl/frog")
            .build()
    }

    fn setup_gactions(&self) {
        let prefs_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| app.show_preferences())
            .build();

        // Quit
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        // About
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        // GitHub Star
        let github_star_action = gio::ActionEntry::builder("github_star")
            .activate(move |app: &Self, _, _| app.github_star())
            .build();
        // Toast
        let toast_action = gio::ActionEntry::builder("toast")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(move |app: &Self, _, arg| {
                if let Some(message) = arg.expect("Could not get parameter.").get::<String>() {
                    app.show_toast(&message);
                }
            })
            .build();
        // Take Screenshot
        let screenshot_action = gio::ActionEntry::builder("screenshot")
            .activate(move |app: &Self, _, _| app.take_screenshot())
            .build();

        let open_image_action = gio::ActionEntry::builder("open-file")
            .activate(move |app: &Self, _, _| app.select_file())
            .build();

        let paste_from_clipboard_action = gio::ActionEntry::builder("paste-from-clipboard")
            .activate(move |app: &Self, _, _| app.paste_from_clipboard())
            .build();

        // Set keyboard shortcusts
        self.set_accels_for_action(
            format!("app.{}", prefs_action.name()).as_str(),
            &[&"<Primary>comma"],
        );
        self.set_accels_for_action(
            format!("app.{}", screenshot_action.name()).as_str(),
            &[&"<Primary>g"],
        );
        self.set_accels_for_action(
            format!("app.{}", open_image_action.name()).as_str(),
            &[&"<Primary>o"],
        );
        self.set_accels_for_action(
            format!("app.{}", paste_from_clipboard_action.name()).as_str(),
            &[&"<Primary>v"],
        );

        // Add actions
        self.add_action_entries([
            prefs_action,
            quit_action,
            about_action,
            toast_action,
            screenshot_action,
            open_image_action,
            paste_from_clipboard_action,
            github_star_action,
        ]);
    }

    fn show_preferences(&self) {
        let window = self.active_window().unwrap();
        let prefs = crate::preferences_dialog::PrerefencesDialog::new();
        prefs.present(Some(&window));
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("frog")
            .application_icon("com.tenderowl.frog")
            .developer_name("Andrey Maksimov")
            .version(VERSION)
            .developers(vec!["Andrey Maksimov"])
            // Translators: Replace "translator-credits" with your name/username, and optionally an email or URL.
            .translator_credits(&gettext("translator-credits"))
            .copyright("© 2025 Andrey Maksimov")
            .build();

        about.present(Some(&window));
    }

    fn github_star(&self) {
        let launcher = gtk::UriLauncher::new("https://github.com/tenderowl/frog");
        if let Some(window) = self.active_window() {
            launcher.launch(Some(&window), gio::Cancellable::NONE, |result| {
                if let Err(err) = result {
                    tracing::error!("failed to launch github star: {err}");
                }
            });
        }
    }

    fn show_toast(&self, message: &str) {
        if let Some(window) = self.active_window() {
            window.downcast::<FrogWindow>().unwrap().show_toast(message);
        }
    }

    fn take_screenshot(&self) {
        tracing::info!("begin taking screenshot");

        let window = match self.active_window() {
            Some(window) => window,
            None => return,
        };

        glib::spawn_future_local(clone!(
            #[weak]
            window,
            #[weak(rename_to=app)]
            self,
            async move {
                // Must get identifier while window is visible (Wayland needs the surface)
                let native = window.native().unwrap();
                let identifier = WindowIdentifier::from_native(&native)
                    .await
                    .map(|id| id.to_string())
                    .unwrap_or_default();

                // Now hide so it doesn't overlay the screenshot area
                window.set_visible(false);

                // On Wayland the compositor processes the unmap asynchronously,
                // so requesting the screenshot immediately can still capture the
                // window. Wait until the hide actually takes effect first.
                wait_until_window_hidden(&window).await;

                tracing::info!("send screenshot request");

                match crate::portal::take_screenshot(identifier).await {
                    Ok(uri) => {
                        let file = gio::File::for_uri(&uri);
                        let filepath = file.path().unwrap_or_default();
                        tracing::info!("Screenshot saved to {}", filepath.display());
                        app.extract_from_file(filepath.to_str().unwrap_or_default(), false);
                    }
                    Err(err) => {
                        tracing::error!("Failed to take a screenshot: {err}");
                        window.set_visible(true);
                    }
                }
            }
        ));
    }

    fn select_file(&self) {
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Images"));
        filter.add_mime_type("image/*");

        let dialog = gtk::FileDialog::builder()
            .title("Open image")
            .accept_label("Select")
            .default_filter(&filter)
            .build();

        let window = match self.active_window() {
            Some(window) => window,
            None => return,
        };

        dialog.open(
            Some(&window),
            gio::Cancellable::NONE,
            glib::clone!(
                #[weak(rename_to=app)]
                self,
                move |file| {
                    if let Ok(file) = file {
                        app.on_select_file(file);
                    }
                }
            ),
        );
    }

    fn on_select_file(&self, result: gio::File) {
        let filepath = result.path().expect("Failed to get file path");
        tracing::info!("File selected: {}", filepath.display());
        if let Some(window) = self.active_window() {
            window
                .downcast::<FrogWindow>()
                .expect("Failed to downcast to FrogWindow")
                .show_extracted_page();

            self.extract_from_file(filepath.to_str().unwrap_or_default(), false);
        }
    }

    fn paste_from_clipboard(&self) {
        let display = gdk::Display::default().expect("Failed to get default display");
        let clipboard = display.clipboard();

        let window = match self.active_window() {
            Some(window) => window,
            None => return,
        };

        clipboard.read_texture_async(
            gio::Cancellable::NONE,
            glib::clone!(
                #[weak]
                window,
                #[weak(rename_to=app)]
                self,
                move |texture| {
                    if let Ok(Some(texture)) = texture {
                        let frog_window = window
                            .downcast_ref::<FrogWindow>()
                            .expect("Failed to downcast to FrogWindow");

                        let path = std::env::temp_dir().join("frog_clipboard.png");
                        if let Err(e) = texture.save_to_png(&path) {
                            tracing::error!("Failed to save clipboard texture: {e}");
                            frog_window.show_toast("Failed to process clipboard image");
                            return;
                        }

                        frog_window.show_extracted_page();
                        app.extract_from_file(path.to_str().unwrap_or_default(), true);
                    }
                }
            ),
        );
    }

    pub fn extract_from_file(&self, path: &str, delete_after: bool) {
        let window = match self.active_window() {
            Some(window) => window.downcast::<FrogWindow>().unwrap(),
            None => return,
        };

        let filepath = path.to_string();
        let delete_after = delete_after;
        let tessdata_path = resolve_tessdata_path();
        tracing::info!("OCR: using tessdata from {tessdata_path}");

        window.show_spinner(true);
        window.show_extracted_page();
        window.set_visible(true);

        glib::spawn_future_local(clone!(
            #[weak]
            window,
            async move {
                let result = gio::spawn_blocking(move || -> Result<String, String> {
                    tracing::info!("OCR: opening image from {filepath}");
                    let img = image::open(&filepath).map_err(|e| {
                        tracing::error!("Failed to open image: {e}");
                        e.to_string()
                    })?;
                    tracing::info!("OCR: image loaded: {}x{}", img.width(), img.height());

                    // Try QR code detection first
                    let qr_result = try_decode_qr(&img);
                    if let Some(text) = qr_result {
                        tracing::info!("QR code detected: {}", text);
                        return Ok(text);
                    }
                    tracing::info!("No QR code found, falling back to OCR");

                    if delete_after {
                        if let Err(e) = std::fs::remove_file(&filepath) {
                            tracing::warn!("Failed to remove temp image {}: {e}", filepath);
                        } else {
                            tracing::info!("Removed temp image {}", filepath);
                        }
                    }

                    let (active_lang, extra_lang) = {
                        let settings = gio::Settings::new(crate::config::APP_ID);
                        let active = settings.string("active-language").to_string();
                        let extra = settings.string("extra-language").to_string();
                        (active, extra)
                    };

                    let ocr_lang = if extra_lang.is_empty() || extra_lang == active_lang {
                        active_lang.clone()
                    } else {
                        format!("{}+{}", active_lang, extra_lang)
                    };

                    let tessdata_file = std::path::Path::new(&tessdata_path)
                        .join(format!("{}.traineddata", &active_lang));
                    if !tessdata_file.exists() {
                        tracing::error!(
                            "OCR: tessdata file not found at {}",
                            tessdata_file.display()
                        );
                    }

                    let mut engine = OcrEngine::new(&tessdata_path, &ocr_lang).map_err(|e| {
                        tracing::error!("Failed to create OCR engine: {e}");
                        e.to_string()
                    })?;

                    let ocr_result = engine.recognize(&img).map_err(|e| {
                        tracing::error!("OCR recognition failed: {e}");
                        e.to_string()
                    })?;

                    tracing::info!(
                        "OCR complete: {} words, {:.1}% confidence, language: {}",
                        ocr_result.word_count,
                        ocr_result.confidence,
                        ocr_result.language
                    );

                    Ok(ocr_result.text)
                })
                .await;

                window.show_spinner(false);

                match result {
                    Ok(Ok(text)) => {
                        window.show_extracted_text(text);
                    }
                    Ok(Err(e)) => {
                        window.show_toast(&format!("OCR failed: {e}"));
                    }
                    Err(e) => {
                        tracing::error!("OCR task panicked: {:?}", e);
                        window.show_toast("Failed to extract text");
                    }
                }
            }
        ));
    }
}

fn try_decode_qr(img: &imageproc::image::DynamicImage) -> Option<String> {
    let luma = img.to_luma8();
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    let grids = prepared.detect_grids();
    for grid in grids {
        if let Ok((_meta, content)) = grid.decode() {
            return Some(content);
        }
    }
    None
}

async fn wait_until_window_hidden(window: &gtk::Window) {
    let start = std::time::Instant::now();
    while window.is_visible() && start.elapsed() < std::time::Duration::from_millis(500) {
        glib::timeout_future(std::time::Duration::from_millis(30)).await;
    }
    // Extra grace period for the compositor to drop the last frame.
    glib::timeout_future(std::time::Duration::from_millis(200)).await;
}

pub fn resolve_tessdata_path() -> String {
    // let home = std::env::var("HOME").unwrap_or_default();
    let candidates: Vec<std::path::PathBuf> = vec![
        std::path::PathBuf::from(format!(
            "{}/tessdata",
            std::env::var("XDG_DATA_HOME").unwrap_or_default()
        )),
        glib::user_data_dir().join("tessdata"),
        std::path::PathBuf::from("/app/share/appdata/tessdata"),
        std::path::PathBuf::from("data/tessdata"),
        std::path::PathBuf::from("/usr/share/tessdata"),
        std::path::PathBuf::from("/usr/share/tesseract-ocr/4.00/tessdata"),
        std::path::PathBuf::from("/usr/share/tesseract-ocr/tessdata"),
    ];

    for path in &candidates {
        if path.join("eng.traineddata").exists() {
            return path.to_str().unwrap_or("").to_string();
        }
    }

    // Fallback to user data dir
    glib::user_data_dir()
        .join("tessdata")
        .to_str()
        .unwrap_or("/app/share/appdata/tessdata")
        .to_string()
}

fn init_tessdata() -> Result<(), ()> {
    let tessdata_dir = glib::user_data_dir().join("tessdata");
    if !tessdata_dir.exists() {
        std::fs::create_dir_all(&tessdata_dir).map_err(|e| {
            tracing::error!("Failed to create tessdata directory: {}", e);
            ()
        })?;
    }

    let dest_path = tessdata_dir.join("eng.traineddata");
    if dest_path.exists() {
        return Ok(());
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let candidates: Vec<std::path::PathBuf> = vec![
        std::path::PathBuf::from(format!("{}/.tesseract-rs/tessdata/eng.traineddata", home)),
        std::path::PathBuf::from("/app/share/appdata/tessdata/eng.traineddata"),
        std::path::PathBuf::from("data/tessdata/eng.traineddata"),
        tessdata_dir.clone(),
        std::path::PathBuf::from("/usr/share/tessdata/eng.traineddata"),
        std::path::PathBuf::from("/usr/share/tesseract-ocr/4.00/tessdata/eng.traineddata"),
        std::path::PathBuf::from("/usr/share/tesseract-ocr/tessdata/eng.traineddata"),
    ];

    for source_path in &candidates {
        if source_path.exists() {
            tracing::info!("Copying tessdata from {}", source_path.display());
            std::fs::copy(source_path, &dest_path).map_err(|e| {
                tracing::error!("Failed to copy eng.traineddata: {}", e);
                ()
            })?;
            return Ok(());
        }
    }

    tracing::error!(
        "eng.traineddata not found in any candidate path. \
         Searched: {:?}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
    );
    Err(())
}
