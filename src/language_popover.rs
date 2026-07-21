use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use std::cell::RefCell;

use crate::language_item::LanguageItem;
use crate::language_manager::LanguageManager;
use crate::language_popover_row::LanguagePopoverRow;

mod imp {

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/language-popover.ui")]
    pub struct LanguagePopover {
        #[template_child]
        pub views: TemplateChild<gtk::Stack>,
        // #[template_child]
        // pub search_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ListBox>,

        pub active_language_code: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LanguagePopover {
        const NAME: &'static str = "LanguagePopover";
        type Type = super::LanguagePopover;
        type ParentType = gtk::Popover;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LanguagePopover {
        fn signals() -> &'static [glib::subclass::Signal] {
            use once_cell::sync::Lazy;
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
                vec![glib::subclass::Signal::builder("language-changed")
                    .param_types([LanguageItem::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for LanguagePopover {}
    impl PopoverImpl for LanguagePopover {}

    #[gtk::template_callbacks]
    impl LanguagePopover {
        // #[template_callback]
        // fn on_visible_changed(&self, _popover: &gtk::Popover, visible: bool) {
        //     tracing::info!("on popover visible changed: {}", visible);
        //     if visible {
        //         self.obj().populate_model();
        //     }
        // }

        #[template_callback]
        fn on_popover_show(&self) {
            self.obj().populate_model();
        }

        #[template_callback]
        fn on_popover_closed(&self) {
            self.entry.set_text("");
        }

        #[template_callback]
        fn on_search_changed(&self, entry: &gtk::SearchEntry) {
            self.obj().filter_list(entry.text().as_str());
        }

        #[template_callback]
        fn on_stop_search(&self, _entry: &gtk::SearchEntry) {
            self.obj().popdown();
        }

        #[template_callback]
        fn on_language_activate(&self, row: &gtk::ListBoxRow) {
            if let Some(popover_row) = row.downcast_ref::<LanguagePopoverRow>() {
                if let Some(item) = popover_row.item() {
                    self.obj().set_active_language_code(&item.code());
                    let lm = LanguageManager::instance();
                    lm.set_active_language(&item);
                    self.obj().emit_language_changed(&item);
                    self.obj().popdown();
                }
            }
        }

        #[template_callback]
        fn on_add_clicked(&self) {
            let app = gtk::Application::default();
            app.activate_action("app.preferences", None);
            self.obj().popdown();
        }
    }
}

glib::wrapper! {
    pub struct LanguagePopover(ObjectSubclass<imp::LanguagePopover>)
        @extends gtk::Popover, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl LanguagePopover {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn connect_language_changed<F: Fn(&Self, &LanguageItem) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("language-changed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let item = values[1].get::<LanguageItem>().unwrap();
            f(&obj, &item);
            None
        })
    }

    fn emit_language_changed(&self, item: &LanguageItem) {
        self.emit_by_name::<()>("language-changed", &[item]);
    }

    pub fn active_language_code(&self) -> String {
        self.imp().active_language_code.borrow().clone()
    }

    pub fn set_active_language_code(&self, code: &str) {
        *self.imp().active_language_code.borrow_mut() = code.to_string();
    }

    fn populate_model(&self) {
        let imp = self.imp();

        // Clear existing rows
        while let Some(child) = imp.list_view.first_child() {
            imp.list_view.remove(&child);
        }

        let lm = LanguageManager::instance();
        let downloaded = lm.get_downloaded_codes(true);
        let active_code = self.active_language_code();

        for code in &downloaded {
            let Some(item) = lm.get_language_item(code) else {
                continue;
            };
            let is_selected = *code == active_code;
            item.set_selected(is_selected);

            let row = LanguagePopoverRow::new();
            row.set_item(&item);
            imp.list_view.append(&row);
        }

        imp.list_view.set_visible(true);

        // Emit language-changed for the active language
        let active_item = lm.get_language_item(&active_code);
        let fallback_item = lm.get_language_item("eng");
        let item_to_emit = active_item.as_ref().or(fallback_item.as_ref());

        if let Some(item) = item_to_emit {
            self.emit_language_changed(item);
        }
    }

    fn filter_list(&self, query: &str) {
        let imp = self.imp();
        let query_lower = query.to_lowercase();

        let mut i = 0;
        while let Some(row) = imp.list_view.row_at_index(i) {
            let visible = if query_lower.is_empty() {
                true
            } else if let Some(popover_row) = row.downcast_ref::<LanguagePopoverRow>() {
                popover_row
                    .item()
                    .map(|item| {
                        item.title().to_lowercase().contains(&query_lower)
                            || item.code().to_lowercase().contains(&query_lower)
                    })
                    .unwrap_or(false)
            } else {
                true
            };
            row.set_visible(visible);
            i += 1;
        }

        // Toggle empty state
        let mut has_visible = false;
        let mut j = 0;
        while let Some(row) = imp.list_view.row_at_index(j) {
            if row.is_visible() {
                has_visible = true;
                break;
            }
            j += 1;
        }

        if has_visible || query_lower.is_empty() {
            imp.views.set_visible_child_name("languages_page");
        } else {
            imp.views.set_visible_child_name("empty_page");
        }
    }
}
