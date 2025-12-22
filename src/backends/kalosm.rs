pub struct KalosmBackend;

use kalosm::vision::{Ocr, OcrInferenceSettings};

impl KalosmBackend {
    pub fn new() -> Self {
        KalosmBackend
    }

    pub async fn process_image(&self, image_path: &str) -> Option<String> {
        let mut model = Ocr::builder().build().await.unwrap();
        let image = image::open(image_path).expect("Failed to open image");
        let result = model.recognize_text(OcrInferenceSettings::new(image));

        match result {
            Ok(text) => Some(text),
            _ => None,
        }
    }
}
