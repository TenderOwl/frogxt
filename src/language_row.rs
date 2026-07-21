use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::language_item::LanguageItem;
use crate::language_manager::LanguageManager;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/language-row.ui")]
    pub struct LanguageRow {
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub install_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub remove_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub progress_bar: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,

        pub item: once_cell::sync::OnceCell<LanguageItem>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LanguageRow {
        const NAME: &'static str = "LanguageRow";
        type Type = super::LanguageRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LanguageRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_signals();
            obj.update_ui();
        }
    }
    impl BoxImpl for LanguageRow {}
    impl WidgetImpl for LanguageRow {}

    #[gtk::template_callbacks]
    impl LanguageRow {
        #[template_callback]
        fn on_download(&self, _button: &gtk::Button) {
            self.obj().on_download();
        }

        #[template_callback]
        fn on_remove(&self, _button: &gtk::Button) {
            self.obj().on_remove();
        }
    }
}

glib::wrapper! {
    pub struct LanguageRow(ObjectSubclass<imp::LanguageRow>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LanguageRow {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_item(&self, item: &LanguageItem) {
        self.imp().item.set(item.clone()).expect("Item already set");
        self.imp().label.set_label(&item.title());
        self.update_ui();
    }

    pub fn item(&self) -> Option<LanguageItem> {
        self.imp().item.get().cloned()
    }

    fn setup_signals(&self) {
        let lm = LanguageManager::instance();

        let obj_weak = self.downgrade();
        lm.connect_downloading(move |_lm, code, progress| {
            if let Some(obj) = obj_weak.upgrade() {
                obj.update_progress(code, progress);
            }
        });

        let obj_weak = self.downgrade();
        lm.connect_downloaded(move |_lm, code| {
            if let Some(obj) = obj_weak.upgrade() {
                obj.on_downloaded(code);
            }
        });
    }

    fn update_ui(&self) {
        let Some(item) = self.imp().item.get() else {
            return;
        };
        let lm = LanguageManager::instance();
        let code = item.code();

        if code == "eng" {
            self.imp().install_btn.set_visible(false);
            self.imp().remove_btn.set_sensitive(false);
            return;
        }

        if lm.is_downloaded(&code) {
            self.imp().install_btn.set_visible(false);
            self.imp().remove_btn.set_visible(true);
        } else if lm.is_loading(&code) {
            self.imp().install_btn.set_sensitive(false);
        } else {
            self.imp().install_btn.set_visible(true);
            self.imp().revealer.set_reveal_child(false);
        }
    }

    fn update_progress(&self, code: &str, progress: f64) {
        if let Some(item) = self.imp().item.get() {
            if item.code() == code {
                if !self.imp().revealer.is_child_revealed() {
                    self.imp().revealer.set_reveal_child(true);
                }
                self.imp().progress_bar.set_fraction(progress / 100.0);
                if progress >= 100.0 {
                    self.imp().revealer.set_reveal_child(false);
                }
            }
        }
    }

    fn on_download(&self) {
        let Some(item) = self.imp().item.get() else {
            return;
        };
        let lm = LanguageManager::instance();
        if lm.is_loading(&item.code()) {
            return;
        }
        lm.download(&item.code());
        self.update_ui();
    }

    fn on_remove(&self) {
        let Some(item) = self.imp().item.get() else {
            return;
        };
        let lm = LanguageManager::instance();
        if lm.is_loading(&item.code()) {
            return;
        }
        if lm.is_downloaded(&item.code()) {
            lm.remove_language(&item.code());
            self.update_ui();
        }
    }

    fn on_downloaded(&self, code: &str) {
        if let Some(item) = self.imp().item.get() {
            if item.code() == code {
                self.update_ui();
            }
        }
    }
}
