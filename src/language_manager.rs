use std::collections::HashMap;
use std::path::PathBuf;

use gtk::glib;
use gtk::glib::SignalHandlerId;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use once_cell::sync::Lazy;

use crate::application::resolve_tessdata_path;
use crate::config::{TESSDATA_BEST_URL, TESSDATA_URL};
use crate::language_item::LanguageItem;

#[derive(Debug, Clone)]
pub struct DownloadState {
    pub total: u64,
    pub progress: f64,
}

impl Default for DownloadState {
    fn default() -> Self {
        Self {
            total: 0,
            progress: 0.0,
        }
    }
}

static LANGUAGES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("afr", "Afrikaans");
    m.insert("amh", "Amharic");
    m.insert("ara", "Arabic");
    m.insert("asm", "Assamese");
    m.insert("aze", "Azerbaijani");
    m.insert("aze_cyrl", "Azerbaijani - Cyrilic");
    m.insert("bel", "Belarusian");
    m.insert("ben", "Bengali");
    m.insert("bod", "Tibetan");
    m.insert("bos", "Bosnian");
    m.insert("bre", "Breton");
    m.insert("bul", "Bulgarian");
    m.insert("cat", "Catalan; Valencian");
    m.insert("ceb", "Cebuano");
    m.insert("ces", "Czech");
    m.insert("chi_sim", "Chinese - Simplified");
    m.insert("chi_tra", "Chinese - Traditional");
    m.insert("chr", "Cherokee");
    m.insert("cos", "Corsican");
    m.insert("cym", "Welsh");
    m.insert("dan", "Danish");
    m.insert("deu", "German");
    m.insert("dzo", "Dzongkha");
    m.insert("ell", "Greek, Modern (1453-)");
    m.insert("eng", "English");
    m.insert("enm", "English, Middle (1100-1500)");
    m.insert("epo", "Esperanto");
    m.insert("equ", "Math / equation detection module");
    m.insert("est", "Estonian");
    m.insert("eus", "Basque");
    m.insert("fao", "Faroese");
    m.insert("fas", "Persian");
    m.insert("fil", "Filipino (old - Tagalog)");
    m.insert("fin", "Finnish");
    m.insert("fra", "French");
    m.insert("frk", "German - Fraktur");
    m.insert("frm", "French, Middle (ca.1400-1600)");
    m.insert("fry", "Western Frisian");
    m.insert("gla", "Scottish Gaelic");
    m.insert("gle", "Irish");
    m.insert("glg", "Galician");
    m.insert("grc", "Greek, Ancient (to 1453) (contrib)");
    m.insert("guj", "Gujarati");
    m.insert("hat", "Haitian; Haitian Creole");
    m.insert("heb", "Hebrew");
    m.insert("hin", "Hindi");
    m.insert("hrv", "Croatian");
    m.insert("hun", "Hungarian");
    m.insert("hye", "Armenian");
    m.insert("iku", "Inuktitut");
    m.insert("ind", "Indonesian");
    m.insert("isl", "Icelandic");
    m.insert("ita", "Italian");
    m.insert("ita_old", "Italian - Old");
    m.insert("jav", "Javanese");
    m.insert("jpn", "Japanese");
    m.insert("jpn_vert", "Japanese (vertical)");
    m.insert("kan", "Kannada");
    m.insert("kat", "Georgian");
    m.insert("kat_old", "Georgian - Old");
    m.insert("kaz", "Kazakh");
    m.insert("khm", "Central Khmer");
    m.insert("kir", "Kirghiz; Kyrgyz");
    m.insert("kmr", "Kurmanji (Kurdish - Latin Script)");
    m.insert("kor", "Korean");
    m.insert("kor_vert", "Korean (vertical)");
    m.insert("lao", "Lao");
    m.insert("lat", "Latin");
    m.insert("lav", "Latvian");
    m.insert("lit", "Lithuanian");
    m.insert("ltz", "Luxembourgish");
    m.insert("mal", "Malayalam");
    m.insert("mar", "Marathi");
    m.insert("mkd", "Macedonian");
    m.insert("mlt", "Maltese");
    m.insert("mon", "Mongolian");
    m.insert("mri", "Maori");
    m.insert("msa", "Malay");
    m.insert("mya", "Burmese");
    m.insert("nep", "Nepali");
    m.insert("nld", "Dutch; Flemish");
    m.insert("nor", "Norwegian");
    m.insert("oci", "Occitan (post 1500)");
    m.insert("ori", "Oriya");
    m.insert("osd", "Orientation and script detection module");
    m.insert("pan", "Panjabi; Punjabi");
    m.insert("pol", "Polish");
    m.insert("por", "Portuguese");
    m.insert("pus", "Pushto; Pashto");
    m.insert("que", "Quechua");
    m.insert("ron", "Romanian; Moldavian; Moldovan");
    m.insert("rus", "Russian");
    m.insert("san", "Sanskrit");
    m.insert("sin", "Sinhala; Sinhalese");
    m.insert("slk", "Slovak");
    m.insert("slv", "Slovenian");
    m.insert("snd", "Sindhi");
    m.insert("spa", "Spanish; Castilian");
    m.insert("spa_old", "Spanish; Castilian - Old");
    m.insert("sqi", "Albanian");
    m.insert("srp", "Serbian");
    m.insert("srp_latn", "Serbian - Latin");
    m.insert("sun", "Sundanese");
    m.insert("swa", "Swahili");
    m.insert("swe", "Swedish");
    m.insert("syr", "Syriac");
    m.insert("tam", "Tamil");
    m.insert("tat", "Tatar");
    m.insert("tel", "Telugu");
    m.insert("tgk", "Tajik");
    m.insert("tha", "Thai");
    m.insert("tir", "Tigrinya");
    m.insert("ton", "Tonga");
    m.insert("tur", "Turkish");
    m.insert("uig", "Uighur; Uyghur");
    m.insert("ukr", "Ukrainian");
    m.insert("urd", "Urdu");
    m.insert("uzb", "Uzbek");
    m.insert("uzb_cyrl", "Uzbek - Cyrilic");
    m.insert("vie", "Vietnamese");
    m.insert("yid", "Yiddish");
    m.insert("yor", "Yoruba");
    m
});

mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use gtk::glib;
    use gtk::glib::subclass::prelude::*;

    use super::DownloadState;

    #[derive(Debug, Default)]
    pub struct LanguageManager {
        pub downloaded_codes: RefCell<Vec<String>>,
        pub need_update_cache: RefCell<bool>,
        pub loading_languages: RefCell<HashMap<String, DownloadState>>,
        pub active_language_code: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LanguageManager {
        const NAME: &'static str = "LanguageManager";
        type Type = super::LanguageManager;
    }

    impl ObjectImpl for LanguageManager {
        fn constructed(&self) {
            self.parent_constructed();
            *self.need_update_cache.borrow_mut() = true;
            *self.active_language_code.borrow_mut() = "eng".to_string();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            super::signals::signals()
        }
    }
}

glib::wrapper! {
    pub struct LanguageManager(ObjectSubclass<imp::LanguageManager>);
}

pub(crate) mod signals {
    use gtk::glib;
    use gtk::glib::prelude::*;

    pub fn signals() -> &'static [glib::subclass::Signal] {
        use once_cell::sync::Lazy;
        static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
            vec![
                glib::subclass::Signal::builder("added")
                    .param_types([String::static_type()])
                    .build(),
                glib::subclass::Signal::builder("downloading")
                    .param_types([String::static_type(), f64::static_type()])
                    .build(),
                glib::subclass::Signal::builder("downloaded")
                    .param_types([String::static_type()])
                    .build(),
                glib::subclass::Signal::builder("removed")
                    .param_types([String::static_type()])
                    .build(),
            ]
        });
        SIGNALS.as_ref()
    }
}

use std::cell::RefCell as StdRefCell;

thread_local! {
    static INSTANCE: StdRefCell<Option<LanguageManager>> = const { StdRefCell::new(None) };
}

impl LanguageManager {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn instance() -> Self {
        INSTANCE.with(|cell| {
            let mut borrow = cell.borrow_mut();
            if let Some(ref lm) = *borrow {
                lm.clone()
            } else {
                let lm = Self::new();
                *borrow = Some(lm.clone());
                lm
            }
        })
    }

    // -- signals --

    pub fn connect_added<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId {
        self.connect_local("added", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let code = values[1].get::<String>().unwrap();
            f(&obj, &code);
            None
        })
    }

    pub fn connect_downloading<F: Fn(&Self, &str, f64) + 'static>(&self, f: F) -> SignalHandlerId {
        self.connect_local("downloading", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let code = values[1].get::<String>().unwrap();
            let progress = values[2].get::<f64>().unwrap();
            f(&obj, &code, progress);
            None
        })
    }

    pub fn connect_downloaded<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId {
        self.connect_local("downloaded", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let code = values[1].get::<String>().unwrap();
            f(&obj, &code);
            None
        })
    }

    pub fn connect_removed<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId {
        self.connect_local("removed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let code = values[1].get::<String>().unwrap();
            f(&obj, &code);
            None
        })
    }

    // -- language queries --

    pub fn get_available_codes(&self) -> Vec<String> {
        let mut codes: Vec<String> = LANGUAGES.keys().map(|k| k.to_string()).collect();
        codes.sort_by(|a, b| {
            let na = LANGUAGES.get(a.as_str()).unwrap_or(&"");
            let nb = LANGUAGES.get(b.as_str()).unwrap_or(&"");
            na.cmp(nb)
        });
        codes
    }

    pub fn get_available_languages(&self) -> Vec<String> {
        let mut langs: Vec<String> = LANGUAGES.values().map(|v| v.to_string()).collect();
        langs.sort();
        langs
    }

    pub fn get_language(&self, code: &str) -> Option<String> {
        LANGUAGES.get(code).map(|s| s.to_string())
    }

    pub fn get_language_item(&self, code: &str) -> Option<LanguageItem> {
        let title = self.get_language(code)?;
        let item = LanguageItem::with_data(code, &title);
        Some(item)
    }

    pub fn get_language_code(&self, language: &str) -> Option<String> {
        LANGUAGES
            .iter()
            .find(|(_, v)| **v == language)
            .map(|(k, _)| k.to_string())
    }

    // -- downloaded languages --

    pub fn get_downloaded_codes(&self, force: bool) -> Vec<String> {
        let imp = self.imp();
        if *imp.need_update_cache.borrow() || force {
            let dir = resolve_tessdata_path();
            let codes: Vec<String> = std::fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    let path = entry.ok()?.path();
                    let name = path.file_stem()?.to_str()?;
                    if path.extension()?.to_str()? == "traineddata" {
                        Some(name.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            *imp.downloaded_codes.borrow_mut() = codes;
            *imp.need_update_cache.borrow_mut() = false;
        }
        let mut sorted = imp.downloaded_codes.borrow().clone();
        sorted.sort_by(|a, b| {
            let na = LANGUAGES.get(a.as_str()).unwrap_or(&"");
            let nb = LANGUAGES.get(b.as_str()).unwrap_or(&"");
            na.cmp(nb)
        });
        sorted
    }

    pub fn get_downloaded_languages(&self, force: bool) -> Vec<String> {
        self.get_downloaded_codes(force)
            .iter()
            .filter_map(|code| self.get_language(code))
            .collect()
    }

    pub fn is_downloaded(&self, code: &str) -> bool {
        self.get_downloaded_codes(false).iter().any(|c| c == code)
    }

    pub fn is_loading(&self, code: &str) -> bool {
        self.imp().loading_languages.borrow().contains_key(code)
    }

    // -- active language --

    pub fn get_active_language(&self) -> LanguageItem {
        let code = self.imp().active_language_code.borrow().clone();
        let title = self
            .get_language(&code)
            .unwrap_or_else(|| "English".to_string());
        LanguageItem::with_data(&code, &title)
    }

    pub fn set_active_language(&self, item: &LanguageItem) {
        *self.imp().active_language_code.borrow_mut() = item.code().to_string();
    }

    // -- download --

    pub fn download(&self, code: &str) {
        self.emit_by_name::<()>("added", &[&code.to_value()]);

        self.imp()
            .loading_languages
            .borrow_mut()
            .insert(code.to_string(), DownloadState::default());

        self.emit_by_name::<()>("downloading", &[&code.to_value(), &0.1f64.to_value()]);

        let obj = self.clone();
        let code_owned = code.to_string();

        glib::spawn_future_local(async move {
            let result =
                gtk::gio::spawn_blocking(move || -> Option<String> { download_begin(&code_owned) })
                    .await
                    .ok()
                    .flatten();

            obj.download_done(result.as_deref());
        });
    }

    fn download_done(&self, code: Option<&str>) {
        *self.imp().need_update_cache.borrow_mut() = true;
        if let Some(c) = code {
            self.imp().loading_languages.borrow_mut().remove(c);
        }
        let code_str = code.unwrap_or("").to_string();
        self.emit_by_name::<()>("downloaded", &[&code_str.to_value()]);
    }

    // -- remove --

    pub fn remove_language(&self, code: &str) {
        let tessdata = PathBuf::from(resolve_tessdata_path());
        let path = tessdata.join(format!("{}.traineddata", code));
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        *self.imp().need_update_cache.borrow_mut() = true;
        self.emit_by_name::<()>("removed", &[&code.to_value()]);
    }
}

fn download_begin(code: &str) -> Option<String> {
    let tessdata = PathBuf::from(resolve_tessdata_path());
    if !tessdata.exists() {
        let _ = std::fs::create_dir_all(&tessdata);
    }

    let filename = format!("{}.traineddata", code);
    let dest = tessdata.join(&filename);

    let urls = [
        format!("{}{}", TESSDATA_BEST_URL, filename),
        format!("{}{}", TESSDATA_URL, filename),
    ];

    for url in &urls {
        tracing::debug!("Downloading {} to {}", url, dest.display());
        match reqwest::blocking::get(url) {
            Ok(resp) => {
                if resp.status().is_success() {
                    match std::fs::write(&dest, resp.bytes().unwrap_or_default()) {
                        Ok(_) => {
                            tracing::debug!("Successfully downloaded {}", code);
                            return Some(code.to_string());
                        }
                        Err(e) => {
                            tracing::error!("Failed to write {}: {}", dest.display(), e);
                        }
                    }
                } else {
                    tracing::debug!("URL {} returned status {}", url, resp.status());
                }
            }
            Err(e) => {
                tracing::debug!("Failed to fetch {}: {}", url, e);
            }
        }
    }

    tracing::debug!("{} was not found at tessdata", code);
    None
}
