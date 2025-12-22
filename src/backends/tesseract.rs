use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use gtk::glib::{self, idle_add_local_once};
use tesseract_rs::TesseractAPI;

use crate::extracted_page;

pub struct TesseractBackend {
    tessdata_path: PathBuf,
}

impl TesseractBackend {
    pub fn new() -> Self {
        Self {
            tessdata_path: PathBuf::from("/app/share/appdata/tessdata"),
        }
    }

    pub async fn process_image(&self, image_path: &str) -> Option<String> {
        let api = TesseractAPI::new();
        if let Err(e) = api.init(
            self.tessdata_path
                .to_str()
                .expect("Failed to load Tesseract model"),
            "eng",
        ) {
            tracing::error!("Failed to initialize Tesseract API: {}", e);
            return None;
        }

        let (image_data, width, height) = self.load_test_image(image_path);

        // Share image data across threads
        let image_data = Arc::new(image_data);
        let mut extracted_text = String::new();

        // Spawn multiple threads for parallel OCR processing
        // for _ in 0..3 {
        let api_clone = api.clone(); // Clones the API with all configurations
        let image_data = Arc::clone(&image_data);

        // Set image in each thread
        let res = api_clone.set_image(
            &image_data,
            width as i32,
            height as i32,
            3,
            3 * width as i32,
        );
        assert!(res.is_ok());

        // Perform OCR in parallel
        let text = api_clone.get_utf8_text().expect("Failed to get text");
        extracted_text.push_str(&text);
        tracing::debug!("Thread result: {}", text);

        // handles.push(handle);
        // }

        // // Wait for all threads to complete
        // for handle in handles {
        //     handle.join().unwrap();
        // }
        tracing::info!("All threads completed");

        Some(extracted_text)
    }

    // Helper function to load test image
    fn load_test_image(&self, filename: &str) -> (Vec<u8>, u32, u32) {
        let img = image::open(filename)
            .expect("Failed to open image")
            .to_rgb8();
        let (width, height) = img.dimensions();
        (img.into_raw(), width, height)
    }
}
