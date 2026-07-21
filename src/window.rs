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

use adw::subclass::prelude::*;
use gio::Settings;
use gtk::gdk;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::config::APP_ID;
use crate::language_manager::LanguageManager;

mod imp {
    use std::cell::OnceCell;

    use crate::{
        extracted_page::ExtractedPage,
        language_popover::LanguagePopover,
        welcome_page::WelcomePage,
    };

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/window.ui")]
    pub struct FrogWindow {
        pub settings: OnceCell<Settings>,
        // Template widgets
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,

        #[template_child]
        pub split_view: TemplateChild<adw::NavigationSplitView>,

        #[template_child]
        pub welcome_page: TemplateChild<WelcomePage>,

        #[template_child]
        pub extracted_page: TemplateChild<ExtractedPage>,

        #[template_child]
        pub spinner: TemplateChild<adw::Spinner>,

        #[template_child]
        pub lang_combo: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub language_popover: TemplateChild<LanguagePopover>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FrogWindow {
        const NAME: &'static str = "FrogWindow";
        type Type = super::FrogWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FrogWindow {
        fn constructed(&self) {
            self.parent_constructed();
            // Load latest window state
            let obj = self.obj();
            obj.setup_settings();
            obj.load_window_size();
            obj.setup_dnd();
            obj.setup_language_popover();
        }
    }
    impl WidgetImpl for FrogWindow {}
    impl WindowImpl for FrogWindow {
        fn close_request(&self) -> glib::Propagation {
            // Save window size
            self.obj()
                .save_window_size()
                .expect("Failed to save window state");
            // Hide instead of closing so the app stays alive in the tray
            self.obj().set_visible(false);
            glib::Propagation::Stop
        }
    }
    impl ApplicationWindowImpl for FrogWindow {}
    impl AdwApplicationWindowImpl for FrogWindow {}
}

glib::wrapper! {
    pub struct FrogWindow(ObjectSubclass<imp::FrogWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gtk::ShortcutManager, gtk::ConstraintTarget, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::Root, gtk::Native;
}

impl FrogWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    pub fn show_toast(&self, message: &str) {
        let toast = adw::Toast::new(message);
        self.imp().toast_overlay.add_toast(toast);
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        // Get the size of the window
        let size = self.default_size();

        // Set the window state in `settings`
        self.settings().set_int("window-width", size.0)?;
        self.settings().set_int("window-height", size.1)?;
        self.settings()
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        // Get the window state from `settings`
        let width = self.settings().int("window-width");
        let height = self.settings().int("window-height");
        let is_maximized = self.settings().boolean("is-maximized");

        // Set the size of the window
        self.set_default_size(width, height);

        // If the window was maximized when it was closed, maximize it again
        if is_maximized {
            self.maximize();
        }
    }

    pub fn show_extracted_page(&self) {
        self.imp().split_view.set_show_content(true);
    }

    pub fn show_welcome_page(&self) {
        self.imp().split_view.set_show_content(false);
    }

    pub fn show_spinner(&self, visible: bool) {
        self.imp().spinner.set_visible(visible);
    }

    pub fn show_extracted_text(&self, text: String) {
        self.imp().extracted_page.set_text(text);
        self.show_extracted_page();
        self.handle_extracted_urls();
    }

    fn handle_extracted_urls(&self) {
        let imp = self.imp();
        let urls = imp.extracted_page.urls();

        if urls.is_empty() {
            return;
        }

        let settings = self.settings();
        let autolinks = settings.boolean("autolinks");

        if autolinks {
            // Auto-open the first URL
            if let Some(url) = urls.first() {
                tracing::info!("Auto-opening URL: {url}");
                let launcher = gtk::UriLauncher::new(url);
                let window_weak = self.downgrade();
                launcher.launch(
                    Some(self),
                    gtk::gio::Cancellable::NONE,
                    move |result| {
                        if let Err(e) = result {
                            tracing::error!("Failed to open URL: {e}");
                            if let Some(w) = window_weak.upgrade() {
                                w.show_toast("Failed to open URL");
                            }
                        } else if let Some(w) = window_weak.upgrade() {
                            w.show_toast("QR-code URL opened");
                        }
                    },
                );
            }
        } else {
            // Show toast with "Open" button
            let toast = adw::Toast::new("QR-code contains URL.");
            toast.set_button_label(Some("Open"));
            toast.set_priority(adw::ToastPriority::High);

            if let Some(url) = urls.first() {
                let url = url.clone();
                let win = self.clone();
                toast.connect_button_clicked(move |_toast| {
                    let launcher = gtk::UriLauncher::new(&url);
                    launcher.launch(Some(&win), gtk::gio::Cancellable::NONE, |result| {
                        if let Err(e) = result {
                            tracing::error!("Failed to open URL: {e}");
                        }
                    });
                });
            }

            imp.toast_overlay.add_toast(toast);
        }
    }

    fn setup_dnd(&self) {
        let drop_target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);

        let win = self.downgrade();
        drop_target.connect_drop(move |_, value, _, _| {
            let Ok(file_list) = value.get::<gdk::FileList>() else {
                return true;
            };
            let files = file_list.files();
            let Some(file) = files.first() else {
                return true;
            };
            let Some(path) = file.path() else {
                return true;
            };
            let Some(win) = win.upgrade() else {
                return true;
            };
            if let Some(app) = win
                .application()
                .and_then(|a| a.downcast::<crate::FrogxtApplication>().ok())
            {
                app.extract_from_file(path.to_str().unwrap_or_default(), false);
            }
            true
        });

        self.add_controller(drop_target);
    }

    fn setup_language_popover(&self) {
        let imp = self.imp();

        // Set initial active language from settings
        let settings = self.settings();
        let active_code = settings.string("active-language").to_string();

        let lm = LanguageManager::instance();
        if let Some(item) = lm.get_language_item(&active_code) {
            lm.set_active_language(&item);
            imp.language_popover.set_active_language_code(&active_code);
            imp.lang_combo.set_label(&item.title());
        } else if let Some(item) = lm.get_language_item("eng") {
            lm.set_active_language(&item);
            imp.language_popover.set_active_language_code("eng");
            imp.lang_combo.set_label(&item.title());
        }

        // Connect to language-changed signal
        let win = self.downgrade();
        imp.language_popover
            .connect_language_changed(move |_popover, item| {
                if let Some(win) = win.upgrade() {
                    win.imp().lang_combo.set_label(&item.title());

                    // Persist the selection
                    if let Some(settings) = win.imp().settings.get() {
                        let _ = settings.set_string("active-language", &item.code());
                    }
                }
            });
    }
}
