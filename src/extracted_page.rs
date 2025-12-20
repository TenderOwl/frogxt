use crate::language_popover::LanguagePopover;
use adw::subclass::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/extracted-page.ui")]
    pub struct ExtractedPage {
        // Template widgets
        #[template_child]
        pub share_list_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub grab_btn: TemplateChild<adw::SplitButton>,
        #[template_child]
        pub text_copy_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub text_view: TemplateChild<gtk::TextView>,
        #[template_child]
        pub buffer: TemplateChild<gtk::TextBuffer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExtractedPage {
        const NAME: &'static str = "ExtractedPage";
        type Type = super::ExtractedPage;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ExtractedPage {}
    impl WidgetImpl for ExtractedPage {}
    impl NavigationPageImpl for ExtractedPage {}
}

glib::wrapper! {
    pub struct ExtractedPage(ObjectSubclass<imp::ExtractedPage>)
        @extends gtk::Widget, adw::NavigationPage,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ExtractedPage {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
