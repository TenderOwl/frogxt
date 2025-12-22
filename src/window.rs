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
use gtk::{gdk, prelude::*};
use gtk::{gio, glib};

use crate::config::APP_ID;

mod imp {
    use std::cell::OnceCell;

    use crate::{extracted_page::ExtractedPage, welcome_page::WelcomePage};

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
        }
    }
    impl WidgetImpl for FrogWindow {}
    impl WindowImpl for FrogWindow {
        fn close_request(&self) -> glib::Propagation {
            // Save window size
            self.obj()
                .save_window_size()
                .expect("Failed to save window state");
            // Allow to invoke other event handlers
            glib::Propagation::Proceed
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

    pub fn begin_extracting_texture(&self, _texture: Option<gdk::Texture>) {
        // self.imp().extracted_texture = Some(texture);
        self.show_extracted_page();
    }

    pub fn show_extracted_text(&self, text: String) {
        self.imp().extracted_page.set_text(text);
        self.show_extracted_page();
    }
}
