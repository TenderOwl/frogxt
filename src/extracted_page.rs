use std::sync::LazyLock;

use adw::subclass::prelude::*;
use gtk::{gdk, glib, prelude::*};

static URL_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r#"https?://[^\s<>"']+"#).unwrap());

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/tenderowl/frog/ui/extracted-page.ui")]
    pub struct ExtractedPage {
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
            let text_view: &gtk::TextView = self.text_view.as_ref();
            let noop = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
            noop.connect_drop(|_, _, _, _| true);
            text_view.add_controller(noop);

            // Set up link-style tag for URLs
            let tag = gtk::TextTag::builder()
                .name("link")
                .foreground("#2a76c6")
                .underline(gtk::pango::Underline::Single)
                .build();
            self.buffer.tag_table().add(&tag);

            // Handle clicks on link tags
            let gesture = gtk::GestureClick::new();
            gesture.set_button(1);
            let buffer_clone = self.buffer.clone();
            let text_view_clone = self.text_view.clone();
            gesture.connect_pressed(move |_gesture, n_press, x, y| {
                if n_press != 1 {
                    return;
                }
                let (buf_x, buf_y) = text_view_clone.window_to_buffer_coords(
                    gtk::TextWindowType::Widget,
                    x as i32,
                    y as i32,
                );
                let Some(iter) = text_view_clone.iter_at_location(buf_x, buf_y) else {
                    return;
                };
                let tag_table = buffer_clone.tag_table();
                let Some(link_tag) = tag_table.lookup("link") else {
                    return;
                };
                if iter.has_tag(&link_tag) {
                    let mut start = iter.clone();
                    start.backward_to_tag_toggle(Some(&link_tag));
                    let mut end = iter;
                    end.forward_to_tag_toggle(Some(&link_tag));
                    let url = buffer_clone.text(&start, &end, false).to_string();
                    let launcher = gtk::UriLauncher::new(&url);
                    let widget = text_view_clone
                        .root()
                        .and_then(|r| r.downcast::<gtk::Window>().ok());
                    launcher.launch(widget.as_ref(), gtk::gio::Cancellable::NONE, |result| {
                        if let Err(e) = result {
                            tracing::error!("Failed to open URL: {e}");
                        }
                    });
                }
            });
            text_view.add_controller(gesture);
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
        let imp = self.imp();
        imp.buffer.set_text(&text);
        self.highlight_urls();
    }

    fn highlight_urls(&self) {
        let imp = self.imp();
        let buffer = &imp.buffer;

        // Remove old link tags
        let (start, end) = buffer.bounds();
        buffer.remove_tag_by_name("link", &start, &end);

        // Find and tag all URLs
        let text = buffer.text(&start, &end, false).to_string();
        for m in URL_RE.find_iter(&text) {
            let byte_start = m.start();
            let byte_end = m.end();
            let iter_start = buffer.iter_at_offset(byte_start as i32);
            let iter_end = buffer.iter_at_offset(byte_end as i32);
            buffer.apply_tag_by_name("link", &iter_start, &iter_end);
        }
    }

    /// Returns all URLs found in the buffer text.
    pub fn urls(&self) -> Vec<String> {
        let imp = self.imp();
        let (start, end) = imp.buffer.bounds();
        let text = imp.buffer.text(&start, &end, false).to_string();
        URL_RE
            .find_iter(&text)
            .map(|m| m.as_str().to_string())
            .collect()
    }
}
