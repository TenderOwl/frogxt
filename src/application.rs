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
use ashpd::{desktop::screenshot, WindowIdentifier};
use gettextrs::gettext;
use gtk::glib::clone;
use gtk::{gdk, gio, glib};

use crate::config::VERSION;
use crate::FrogWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct FrogxtApplication {}

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
        // Quit
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        // About
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
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
            quit_action,
            about_action,
            toast_action,
            screenshot_action,
            open_image_action,
            paste_from_clipboard_action,
        ]);
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

    fn show_toast(&self, message: &str) {
        if let Some(window) = self.active_window() {
            window.downcast::<FrogWindow>().unwrap().show_toast(message);
        }
    }

    fn take_screenshot(&self) {
        tracing::info!("begin taking screenshot");

        if let Some(window) = self.active_window() {
            // Implement screenshot functionality here by utilizing Portals and ASHPD
            glib::spawn_future_local(clone!(
                #[weak]
                window,
                async move {
                    // let root = window.native().unwrap();
                    // let identifier = WindowIdentifier::from_native(&root).await;
                    // let path = std::env::temp_dir().join("frog_screenshot.png");
                    tracing::info!("send screenshot request");

                    match screenshot::ScreenshotRequest::default()
                        // .identifier(identifier)
                        .interactive(true)
                        .modal(false)
                        .send()
                        .await
                        .and_then(|r| r.response())
                    {
                        Ok(response) => {
                            let file = gio::File::for_uri(response.uri().as_str());
                            tracing::info!(
                                "Screenshot saved to {}",
                                file.path().unwrap_or_default().display()
                            );

                            window
                                .downcast_ref::<FrogWindow>()
                                .expect("Failed to downcast to FrogWindow")
                                .show_extracted_page();
                        }
                        Err(err) => {
                            tracing::error!("Failed to take a screenshot {err}");
                        }
                    }

                    tracing::info!("Window show");
                    window.set_visible(true);
                }
            ));

            glib::idle_add_local_full(
                glib::Priority::HIGH,
                clone!(
                    #[weak]
                    window,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        window.set_visible(false);
                        tracing::info!("Window hide");
                        glib::ControlFlow::Break
                    }
                ),
            );
            tracing::info!("end taking screenshot");
            while glib::MainContext::default().pending() {
                glib::MainContext::default().iteration(false);
            }
        }
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
                move |texture| {
                    tracing::info!("Texture read from clipboard: {:?}", texture);
                    if let Ok(texture) = texture {
                        window
                            .downcast_ref::<FrogWindow>()
                            .expect("Failed to downcast to FrogWindow")
                            .begin_extracting_texture(texture);
                    }
                }
            ),
        );
    }
}
