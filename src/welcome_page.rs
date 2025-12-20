use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {

    use crate::language_popover::LanguagePopover;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/welcome-page.ui")]
    pub struct WelcomePage {
        // Template widgets
        #[template_child]
        pub spinner: TemplateChild<adw::Spinner>,

        #[template_child]
        pub welcome: TemplateChild<adw::StatusPage>,

        #[template_child]
        pub lang_combo: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub language_popover: TemplateChild<LanguagePopover>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WelcomePage {
        const NAME: &'static str = "WelcomePage";
        type Type = super::WelcomePage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for WelcomePage {}
    impl WidgetImpl for WelcomePage {}
    impl NavigationPageImpl for WelcomePage {}
}

glib::wrapper! {
    pub struct WelcomePage(ObjectSubclass<imp::WelcomePage>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}
