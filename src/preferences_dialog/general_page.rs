use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/preferences-general.ui")]
    pub struct PreferencesGeneralPage {
        // #[template_child]
        // pub views: TemplateChild<gtk::Stack>,
        // #[template_child]
        // pub search_box: TemplateChild<gtk::Box>,
        // #[template_child]
        // pub entry: TemplateChild<gtk::SearchEntry>,
        // #[template_child]
        // pub list_view: TemplateChild<gtk::ListBox>,

        // pub lang_list: Option<gio::ListStore>,
        // pub filters_list: Option<gtk::FilterListModel>,
        // pub filter: Option<gtk::CustomFilter>,
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

    impl ObjectImpl for PreferencesGeneralPage {}
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
        glib::Object::new()
    }
}
