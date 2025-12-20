use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, subclass::prelude::*, CompositeTemplate};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/language-popover.ui")]
    pub struct LanguagePopover {
        #[template_child]
        pub views: TemplateChild<gtk::Stack>,
        #[template_child]
        pub search_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListBox>,

        pub lang_list: Option<gio::ListStore>,
        pub filters_list: Option<gtk::FilterListModel>,
        pub filter: Option<gtk::CustomFilter>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LanguagePopover {
        const NAME: &'static str = "LanguagePopover";
        type Type = super::LanguagePopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LanguagePopover {}
    impl WidgetImpl for LanguagePopover {}
    impl PopoverImpl for LanguagePopover {}
}

#[gtk::template_callbacks]
impl LanguagePopover {
    #[template_callback]
    fn _on_popover_show(&self) {}

    #[template_callback]
    fn _on_popover_closed(&self) {}

    #[template_callback]
    fn _on_search_activate(&self, _entry: &gtk::Entry) {}

    #[template_callback]
    fn _on_search_changed(&self, _entry: &gtk::Entry) {}

    #[template_callback]
    fn _on_stop_search(&self, _entry: &gtk::Entry) {}

    #[template_callback]
    fn _on_language_activate(&self) {}

    #[template_callback]
    fn _on_add_clicked(&self) {}
}

glib::wrapper! {
    pub struct LanguagePopover(ObjectSubclass<imp::LanguagePopover>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl LanguagePopover {
    pub fn new() -> Self {
        glib::Object::new()
    }
}
