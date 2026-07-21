use adw::subclass::prelude::*;
use gtk::{gdk, glib, prelude::*};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/extracted-page.ui")]
    pub struct ExtractedPage {
        // Template widgets
        // #[template_child]
        // pub share_list_box: TemplateChild<gtk::ListBox>,
        // #[template_child]
        // pub grab_btn: TemplateChild<adw::SplitButton>,
        // #[template_child]
        // pub text_copy_btn: TemplateChild<gtk::Button>,
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

    impl ObjectImpl for ExtractedPage {
        fn constructed(&self) {
            self.parent_constructed();
            // Disable built-in DnD on the text view so file drops are handled
            // by the window's drop target instead of inserting file paths.
            let text_view: &gtk::TextView = self.text_view.as_ref();
            let noop = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
            noop.connect_drop(|_, _, _, _| true);
            text_view.add_controller(noop);
        }
    }
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

    pub fn set_text(&self, text: String) {
        self.imp().buffer.set_text(&text);
    }
}
