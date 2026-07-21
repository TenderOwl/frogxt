use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::gio;
use gtk::{CompositeTemplate, StringList};

use crate::config::APP_ID;
use crate::language_manager::LanguageManager;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/preferences-general.ui")]
    pub struct PreferencesGeneralPage {
        #[template_child]
        pub extra_language_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub autocopy_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub autolinks_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub telemetry_switch: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesGeneralPage {
        const NAME: &'static str = "PreferencesGeneralPage";
        type Type = super::PreferencesGeneralPage;
        type ParentType = adw::PreferencesPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesGeneralPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup();
        }
    }
    impl WidgetImpl for PreferencesGeneralPage {}
    impl PreferencesPageImpl for PreferencesGeneralPage {}
}

glib::wrapper! {
    pub struct PreferencesGeneralPage(ObjectSubclass<imp::PreferencesGeneralPage>)
        @extends gtk::Widget, adw::PreferencesPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PreferencesGeneralPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    fn setup(&self) {
        let settings = gio::Settings::new(APP_ID);
        let imp = self.imp();

        // Bind switches to settings
        let switch_ref: &adw::SwitchRow = imp.autocopy_switch.as_ref();
        settings.bind("autocopy", switch_ref, "active");
        let switch_ref: &adw::SwitchRow = imp.autolinks_switch.as_ref();
        settings.bind("autolinks", switch_ref, "active");
        let switch_ref: &adw::SwitchRow = imp.telemetry_switch.as_ref();
        settings.bind("telemetry", switch_ref, "active");

        // Populate extra language combo with installed languages
        let lm = LanguageManager::instance();
        let downloaded_codes = lm.get_downloaded_codes(false);

        let string_list = Self::build_string_list(&downloaded_codes, &lm);
        imp.extra_language_combo.set_model(Some(&string_list));

        // Find current extra-language index
        let extra_code = settings.string("extra-language").to_string();
        let selected_idx = downloaded_codes
            .iter()
            .position(|c| c == &extra_code)
            .unwrap_or(0) as u32;
        imp.extra_language_combo.set_selected(selected_idx);

        // Save extra language on change
        let combo = imp.extra_language_combo.clone();
        let codes = downloaded_codes.clone();
        imp.extra_language_combo.connect_notify_local(Some("selected"), move |_, _| {
            let idx = combo.selected() as usize;
            if let Some(code) = codes.get(idx) {
                let settings = gio::Settings::new(APP_ID);
                let _ = settings.set_string("extra-language", code);
            }
        });

        // Refresh combo when languages are downloaded or removed
        let obj_weak = self.downgrade();
        lm.connect_downloaded(move |_, _| {
            if let Some(obj) = obj_weak.upgrade() {
                obj.refresh_extra_language_combo();
            }
        });

        let obj_weak = self.downgrade();
        lm.connect_removed(move |_, _| {
            if let Some(obj) = obj_weak.upgrade() {
                obj.refresh_extra_language_combo();
            }
        });
    }

    fn build_string_list(codes: &[String], lm: &LanguageManager) -> StringList {
        let names: Vec<String> = codes
            .iter()
            .filter_map(|code| lm.get_language(code))
            .collect();
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        StringList::new(&refs)
    }

    fn refresh_extra_language_combo(&self) {
        let imp = self.imp();
        let settings = gio::Settings::new(APP_ID);
        let lm = LanguageManager::instance();
        let downloaded_codes = lm.get_downloaded_codes(false);

        let string_list = Self::build_string_list(&downloaded_codes, &lm);
        imp.extra_language_combo.set_model(Some(&string_list));

        let extra_code = settings.string("extra-language").to_string();
        let selected_idx = downloaded_codes
            .iter()
            .position(|c| c == &extra_code)
            .unwrap_or(0) as u32;
        imp.extra_language_combo.set_selected(selected_idx);
    }
}
