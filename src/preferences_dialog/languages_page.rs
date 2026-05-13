use adw::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use crate::language_item;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/preferences-languages.ui")]
    pub struct PreferencesLanguagesPage {
        // #[template_child]
        // pub views: TemplateChild<gtk::Stack>,
        // #[template_child]
        // pub search_box: TemplateChild<gtk::Box>,
        // #[template_child]
        // pub entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub list_store: TemplateChild<gio::ListStore>,
        // pub lang_list: Option<gio::ListStore>,
        // pub filters_list: Option<gtk::FilterListModel>,
        // pub filter: Option<gtk::CustomFilter>,
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

    impl ObjectImpl for PreferencesLanguagesPage {}
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
        glib::Object::new()
    }
}
