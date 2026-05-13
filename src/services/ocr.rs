use image::{DynamicImage, GrayImage};
use tesseract::Tesseract;

pub struct OcrEngine {
    tess: Tesseract,
    lang: String,
}

impl OcrEngine {
    pub fn new(lang: &str) -> Result<Self, OcrError> {
        let tess = Tesseract::new(None, Some(lang))?;
        Ok(Self {
            tess,
            lang: lang.to_string(),
        })
    }

    pub fn recognize(&mut self, image: &DynamicImage) -> Result<OcrResult, OcrError> {
        // Препроцессинг
        let processed = preprocess_image(image)?;

        // OCR
        let text = self.tess.set_image(&processed)?.get_text()?;
        let confidence = self.tess.mean_text_conf()?;

        // Получение bounding boxes
        let boxes = self.get_text_boxes()?;

        Ok(OcrResult {
            text,
            confidence,
            boxes,
            language: self.lang.clone(),
        })
    }

    pub fn auto_detect_language(&mut self, image: &DynamicImage) -> Result<String, OcrError> {
        // Tesseract OSD (Orientation and Script Detection)
        self.tess.set_image(image)?;
        let osd = self.tess.get_osd()?;
        Ok(osd.script_name)
    }

    fn get_text_boxes(&self) -> Result<Vec<TextBox>, OcrError> {
        // Итерация по RIL_WORD или RIL_TEXTLINE
        let mut boxes = Vec::new();
        let iterator = self.tess.get_iterator()?;

        for item in iterator {
            let bbox = item.bounding_box()?;
            let text = item.text()?;
            let conf = item.confidence()?;

            boxes.push(TextBox {
                x: bbox.x1,
                y: bbox.y1,
                width: bbox.x2 - bbox.x1,
                height: bbox.y2 - bbox.y1,
                text,
                confidence: conf,
            });
        }

        Ok(boxes)
    }
}
