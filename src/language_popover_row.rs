use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {

    use crate::language_item::LanguageItem;

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/language-popover.ui")]
    pub struct LanguagePopoverRow {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub selection: TemplateChild<gtk::Image>,

        pub filter: Option<LanguageItem>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LanguagePopoverRow {
        const NAME: &'static str = "LanguagePopoverRow";
        type Type = super::LanguagePopoverRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LanguagePopoverRow {}
    impl WidgetImpl for LanguagePopoverRow {}
    impl ListBoxRowImpl for LanguagePopoverRow {}
}

glib::wrapper! {
    pub struct LanguagePopoverRow(ObjectSubclass<imp::LanguagePopoverRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Actionable;
}

impl LanguagePopoverRow {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
