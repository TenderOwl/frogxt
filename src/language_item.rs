use adw::subclass::prelude::*;
use gtk::glib::{self};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct LanguageItemData {
    pub title: String,
    pub code: String,
    pub selected: bool,
}

impl LanguageItemData {
    pub fn new(title: &str, code: &str, selected: bool) -> Self {
        Self {
            title: title.to_string(),
            code: code.to_string(),
            selected,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}

impl Default for LanguageItemData {
    fn default() -> Self {
        Self {
            title: String::new(),
            code: String::new(),
            selected: false,
        }
    }
}

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LanguageItem {
        pub data: RefCell<LanguageItemData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LanguageItem {
        const NAME: &'static str = "LanguageItem";
        type Type = super::LanguageItem;
    }

    impl ObjectImpl for LanguageItem {}
}

glib::wrapper! {
    pub struct LanguageItem(ObjectSubclass<imp::LanguageItem>);
}

impl LanguageItem {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
