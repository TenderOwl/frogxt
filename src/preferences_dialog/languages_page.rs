use std::cell::RefCell;
use std::collections::HashMap;

use adw::prelude::ActionRowExt;
use adw::prelude::ExpanderRowExt;
use adw::prelude::PreferencesPageExt;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::prelude::*;
use gtk::{gio, CompositeTemplate};

use crate::language_manager::LanguageManager;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/preferences-languages.ui")]
    pub struct PreferencesLanguagesPage {
        #[template_child]
        pub installed_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub available_expander_row: TemplateChild<adw::ExpanderRow>,

        pub installed_rows: RefCell<HashMap<String, adw::ActionRow>>,
        pub available_rows: RefCell<HashMap<String, adw::ActionRow>>,
        pub download_buttons: RefCell<HashMap<String, gtk::Button>>,
        pub download_spinners: RefCell<HashMap<String, gtk::Spinner>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesLanguagesPage {
        const NAME: &'static str = "PreferencesLanguagesPage";
        type Type = super::PreferencesLanguagesPage;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesLanguagesPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup();
        }
    }
    impl WidgetImpl for PreferencesLanguagesPage {}
    impl PreferencesPageImpl for PreferencesLanguagesPage {}
}

glib::wrapper! {
    pub struct PreferencesLanguagesPage(ObjectSubclass<imp::PreferencesLanguagesPage>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PreferencesLanguagesPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup(&self) {
        self.connect_signals();
        self.load_languages();
        self.check_connection();
    }

    fn connect_signals(&self) {
        let lm = LanguageManager::instance();

        let obj_weak = self.downgrade();
        lm.connect_downloaded(move |_lm, _code| {
            if let Some(obj) = obj_weak.upgrade() {
                obj.load_languages();
            }
        });

        let obj_weak = self.downgrade();
        lm.connect_removed(move |_lm, _code| {
            if let Some(obj) = obj_weak.upgrade() {
                obj.load_languages();
            }
        });
    }

    fn load_languages(&self) {
        let imp = self.imp();

        for (_, row) in imp.installed_rows.borrow().iter() {
            imp.installed_expander_row.remove(row);
        }
        imp.installed_rows.borrow_mut().clear();

        for (_, row) in imp.available_rows.borrow().iter() {
            imp.available_expander_row.remove(row);
        }
        imp.available_rows.borrow_mut().clear();
        imp.download_buttons.borrow_mut().clear();
        imp.download_spinners.borrow_mut().clear();

        let lm = LanguageManager::instance();
        let downloaded = lm.get_downloaded_codes(false);

        for code in lm.get_available_codes() {
            let Some(item) = lm.get_language_item(&code) else {
                continue;
            };
            let is_downloaded = downloaded.contains(&code);
            let is_loading = lm.is_loading(&code);

            let row = adw::ActionRow::builder()
                .title(item.title())
                .subtitle(&code)
                .build();

            if is_downloaded {
                if code == "eng" {
                    row.set_subtitle("Default language");
                } else {
                    let remove_btn = gtk::Button::builder()
                        .icon_name("user-trash-symbolic")
                        .valign(gtk::Align::Center)
                        .css_classes(["flat", "error"])
                        .tooltip_text("Remove")
                        .build();

                    let code_owned = code.clone();
                    let obj_weak = self.downgrade();
                    remove_btn.connect_clicked(move |_btn| {
                        if let Some(obj) = obj_weak.upgrade() {
                            obj.remove_language(&code_owned);
                        }
                    });

                    row.add_suffix(&remove_btn);
                }
                imp.installed_expander_row.add_row(&row);
                imp.installed_rows
                    .borrow_mut()
                    .insert(code.clone(), row);
            } else {
                let spinner = gtk::Spinner::builder()
                    .valign(gtk::Align::Center)
                    .visible(is_loading)
                    .spinning(is_loading)
                    .build();

                let download_btn = gtk::Button::builder()
                    .label("Download")
                    .valign(gtk::Align::Center)
                    .css_classes(["flat", "suggested-action"])
                    .visible(!is_loading)
                    .sensitive(!is_loading)
                    .build();

                if is_loading {
                    row.set_subtitle(&format!("{} \u{2014} Downloading\u{2026}", code));
                }

                let code_owned = code.clone();
                let obj_weak = self.downgrade();
                download_btn.connect_clicked(move |_btn| {
                    if let Some(obj) = obj_weak.upgrade() {
                        obj.download_language(&code_owned);
                    }
                });

                row.add_suffix(&spinner);
                row.add_suffix(&download_btn);
                imp.available_expander_row.add_row(&row);
                imp.available_rows
                    .borrow_mut()
                    .insert(code.clone(), row);
                imp.download_buttons
                    .borrow_mut()
                    .insert(code.clone(), download_btn);
                imp.download_spinners
                    .borrow_mut()
                    .insert(code.clone(), spinner);
            }
        }
    }

    fn download_language(&self, code: &str) {
        let lm = LanguageManager::instance();
        if lm.is_loading(code) || lm.is_downloaded(code) {
            return;
        }

        let imp = self.imp();
        if let Some(btn) = imp.download_buttons.borrow().get(code) {
            btn.set_visible(false);
            btn.set_sensitive(false);
        }
        if let Some(spinner) = imp.download_spinners.borrow().get(code) {
            spinner.set_visible(true);
            spinner.set_spinning(true);
        }
        if let Some(row) = imp.available_rows.borrow().get(code) {
            row.set_subtitle(&format!("{} \u{2014} Downloading\u{2026}", code));
        }

        lm.download(code);
    }

    fn remove_language(&self, code: &str) {
        let lm = LanguageManager::instance();
        if lm.is_loading(code) {
            return;
        }
        lm.remove_language(code);
    }

    fn check_connection(&self) {
        let monitor = gio::NetworkMonitor::default();

        let addr = gio::NetworkAddress::new("raw.githubusercontent.com", 443);
        let can_reach = monitor.can_reach(&addr, gio::Cancellable::NONE).is_ok();
        if !can_reach {
            let banner = adw::Banner::builder()
                .title("Models location unreachable. Check your internet connection.")
                .revealed(true)
                .build();
            self.set_banner(Some(&banner));
            return;
        }

        if monitor.is_network_metered() {
            let banner = adw::Banner::builder()
                .title("You are on a metered connection. Be careful to download languages.")
                .revealed(true)
                .build();
            self.set_banner(Some(&banner));
            return;
        }

        self.set_banner(None);
    }
}
