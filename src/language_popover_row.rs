use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::language_item::LanguageItem;

mod imp {
    use std::cell::RefCell;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/language-popover_row.ui")]
    pub struct LanguagePopoverRow {
        #[template_child]
        pub title: TemplateChild<gtk::Label>,
        #[template_child]
        pub selection: TemplateChild<gtk::Image>,

        pub item: RefCell<Option<LanguageItem>>,
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

    pub fn set_item(&self, item: &LanguageItem) {
        self.imp().item.borrow_mut().replace(item.clone());
        self.imp().title.set_label(&item.title());
        self.imp().selection.set_visible(item.selected());
    }

    pub fn item(&self) -> Option<LanguageItem> {
        self.imp().item.borrow().clone()
    }
}
