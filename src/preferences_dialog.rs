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

use adw::prelude::PreferencesDialogExt;
use adw::subclass::prelude::*;
// use gio::Settings;
use gtk::glib;
use gtk::prelude::*;

use std::cell::OnceCell;

use crate::preferences_dialog::general_page::PreferencesGeneralPage;
use crate::preferences_dialog::languages_page::PreferencesLanguagesPage;

mod general_page;
mod languages_page;

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/preferences-dialog.ui")]
    pub struct PrerefencesDialog {
        // #[template_child]
        pub general_page: OnceCell<general_page::PreferencesGeneralPage>,
        // #[template_child]
        pub languages_page: OnceCell<languages_page::PreferencesLanguagesPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PrerefencesDialog {
        const NAME: &'static str = "PrerefencesDialog";
        type Type = super::PrerefencesDialog;
        type ParentType = adw::PreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PrerefencesDialog {
        fn constructed(&self) {
            self.parent_constructed();

            // Load latest window state
            let obj = self.obj();

            obj.build_ui();

            obj.setup_settings();
        }
    }
    impl WidgetImpl for PrerefencesDialog {}
    impl AdwDialogImpl for PrerefencesDialog {}
    impl PreferencesDialogImpl for PrerefencesDialog {}
}

glib::wrapper! {
    pub struct PrerefencesDialog(ObjectSubclass<imp::PrerefencesDialog>)
        @extends gtk::Widget, adw::Dialog, adw::PreferencesDialog,
        @implements gtk::ShortcutManager, gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl PrerefencesDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn build_ui(&self) {
        self.add(&PreferencesGeneralPage::new());
        self.add(&PreferencesLanguagesPage::new());
    }

    fn setup_settings(&self) {
        // Load latest window state
        tracing::info!("Loading current preferences");
    }
}
