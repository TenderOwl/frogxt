use crate::services::preprocess::{preprocess_image, PreprocessConfig, PreprocessError};
use imageproc::image::{self, DynamicImage};
use leptess;
use once_cell::sync::Lazy;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Ошибки OCR
#[derive(thiserror::Error, Debug)]
pub enum OcrError {
    #[error("Tesseract initialization failed: {0}")]
    InitError(String),

    #[error("Failed to set image: {0}")]
    SetImageError(String),

    #[error("OCR recognition failed: {0}")]
    RecognitionError(String),

    #[error("Language '{0}' not available")]
    LanguageNotAvailable(String),

    #[error("Preprocessing failed: {0}")]
    PreprocessError(#[from] PreprocessError),

    #[error("Invalid image data")]
    InvalidImage,
}

/// Bounding box распознанного текста
#[derive(Debug, Clone, PartialEq)]
pub struct TextBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub text: String,
    pub confidence: i32,
    pub level: TextLevel,
}

/// Уровень иерархии текста
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TextLevel {
    Block,
    Paragraph,
    Line,
    Word,
    Symbol,
}

/// Результат OCR
#[derive(Debug, Clone)]
pub struct OcrResult {
    pub text: String,
    pub html: String,
    pub confidence: f32,
    pub boxes: Vec<TextBox>,
    pub language: String,
    pub orientation: i32,
    pub script_confidence: f32,
    pub word_count: usize,
}

/// Движок OCR на базе tesseract-rs
pub struct OcrEngine {
    api: leptess::LepTess,
    language: String,
    tessdata_dir: String,
}

/// Глобальный кэш движков по языку (для переиспользования)
static ENGINE_CACHE: Lazy<Mutex<Vec<(String, Arc<Mutex<OcrEngine>>)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

impl OcrEngine {
    /// Создать новый движок
    pub fn new(tessdata_dir: &str, language: &str) -> Result<Self, OcrError> {
        let mut api = leptess::LepTess::new(Some(tessdata_dir), language).map_err(|e| {
            OcrError::InitError(format!(
                "Failed to init Tesseract with lang '{}': {}",
                language, e
            ))
        })?;

        // api.init(tessdata_dir, language).map_err(|e| {
        //     OcrError::InitError(format!(
        //         "Failed to init Tesseract with lang '{}': {}",
        //         language, e
        //     ))
        // })?;

        // Оптимальные настройки для точности
        api.set_variable(leptess::Variable::TesseditPagesegMode, "3") // PSM_AUTO
            .map_err(|e| OcrError::InitError(e.to_string()))?;

        api.set_variable(leptess::Variable::TesseditOcrEngineMode, "3") // LSTM_ONLY
            .map_err(|e| OcrError::InitError(e.to_string()))?;

        Ok(Self {
            api,
            language: language.to_string(),
            tessdata_dir: tessdata_dir.to_string(),
        })
    }

    /// Получить движок из кэша или создать новый
    pub fn get_or_create(tessdata_dir: &str, language: &str) -> Result<Arc<Mutex<Self>>, OcrError> {
        let mut cache = ENGINE_CACHE.lock().unwrap();

        // Ищем в кэше
        for (lang, engine) in cache.iter() {
            if lang == language {
                return Ok(Arc::clone(engine));
            }
        }

        // Создаём новый
        let engine = Arc::new(Mutex::new(Self::new(tessdata_dir, language)?));
        cache.push((language.to_string(), Arc::clone(&engine)));

        Ok(engine)
    }

    /// Очистить кэш движков
    pub fn clear_cache() {
        let mut cache = ENGINE_CACHE.lock().unwrap();
        cache.clear();
    }

    /// Установить Page Segmentation Mode
    pub fn set_psm(&mut self, mode: i32) -> Result<(), OcrError> {
        self.api
            .set_variable(leptess::Variable::TesseditPagesegMode, &mode.to_string())
            .map_err(|e| OcrError::InitError(e.to_string()))?;
        Ok(())
    }

    /// Установить whitelist символов
    pub fn set_whitelist(&mut self, chars: &str) -> Result<(), OcrError> {
        self.api
            .set_variable(leptess::Variable::TesseditCharWhitelist, chars)
            .map_err(|e| OcrError::InitError(e.to_string()))?;
        Ok(())
    }

    /// Установить blacklist символов
    pub fn set_blacklist(&mut self, chars: &str) -> Result<(), OcrError> {
        self.api
            .set_variable(leptess::Variable::TesseditCharBlacklist, chars)
            .map_err(|e| OcrError::InitError(e.to_string()))?;
        Ok(())
    }

    /// Распознать текст из изображения (без агрессивного препроцессинга)
    pub fn recognize(&mut self, image: &DynamicImage) -> Result<OcrResult, OcrError> {
        self.recognize_raw(image)
    }

    /// Распознать с кастомным препроцессингом
    pub fn recognize_with_config(
        &mut self,
        image: &DynamicImage,
        config: &PreprocessConfig,
    ) -> Result<OcrResult, OcrError> {
        // Препроцессинг
        let preprocessed = preprocess_image(image, config)?;
        let processed = preprocessed.image;

        // Кодируем в PNG для leptonica (set_image_from_mem ожидает encoded bytes)
        let mut png_buffer = Vec::new();
        processed
            .write_to(&mut Cursor::new(&mut png_buffer), image::ImageFormat::Png)
            .map_err(|e| OcrError::SetImageError(format!("Failed to encode image to PNG: {e}")))?;

        // Устанавливаем изображение
        self.api
            .set_image_from_mem(&png_buffer)
            .map_err(|e| OcrError::SetImageError(e.to_string()))?;

        // Получаем текст
        let text = self
            .api
            .get_utf8_text()
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        // Получаем HTML (hOCR)
        let html = self
            .api
            .get_hocr_text(0)
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        // Получаем среднюю уверенность
        let confidence = self.api.mean_text_conf() as f32;
        // let confidences = self
        //     .api
        //     .mean_text_conf()
        //     .map_err(|e| OcrError::RecognitionError(e.to_string()))?;
        // let confidence = if confidences.is_empty() {
        //     0.0
        // } else {
        //     confidences.iter().sum::<i32>() as f32 / confidences.len() as f32
        // };

        // Получаем bounding boxes
        let boxes = self.get_text_boxes()?;

        // Получаем ориентацию
        let orientation = self.get_orientation()?;

        let word_count = text.split_whitespace().count();

        Ok(OcrResult {
            text,
            html,
            confidence,
            boxes,
            language: self.language.clone(),
            orientation,
            script_confidence: confidence,
            word_count,
        })
    }

    /// Распознать без препроцессинга (для уже обработанных изображений)
    pub fn recognize_raw(&mut self, image: &DynamicImage) -> Result<OcrResult, OcrError> {
        let luma = image.to_luma8();
        let mut png_buffer = Vec::new();
        luma.write_to(&mut Cursor::new(&mut png_buffer), image::ImageFormat::Png)
            .map_err(|e| OcrError::SetImageError(format!("Failed to encode image to PNG: {e}")))?;

        self.api
            .set_image_from_mem(&png_buffer)
            .map_err(|e| OcrError::SetImageError(e.to_string()))?;

        let text = self
            .api
            .get_utf8_text()
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        let html = self
            .api
            .get_hocr_text(0)
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        // let confidences = self
        //     .api
        //     .all_word_confidences()
        //     .map_err(|e| OcrError::RecognitionError(e.to_string()))?;
        // let confidence = if confidences.is_empty() {
        //     0.0
        // } else {
        //     confidences.iter().sum::<i32>() as f32 / confidences.len() as f32
        // };
        let confidence = self.api.mean_text_conf() as f32;

        let boxes = self.get_text_boxes()?;
        let orientation = self.get_orientation()?;
        let word_count = text.split_whitespace().count();

        Ok(OcrResult {
            text,
            html,
            confidence,
            boxes,
            language: self.language.clone(),
            orientation,
            script_confidence: confidence,
            word_count,
        })
    }

    /// Получить bounding boxes для всех уровней
    fn get_text_boxes(&mut self) -> Result<Vec<TextBox>, OcrError> {
        // tesseract-rs не предоставляет прямого доступа к iterator API
        // Используем hOCR парсинг для получения координат
        let hocr = self
            .api
            .get_hocr_text(0)
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        parse_hocr_boxes(&hocr)
    }

    /// Получить ориентацию страницы
    fn get_orientation(&self) -> Result<i32, OcrError> {
        // tesseract-rs v0.2 не предоставляет прямого OSD API
        // Возвращаем 0 (нет поворота) как дефолт
        // TODO: реализовать через get_osd_text если доступно
        Ok(0)
    }

    /// Автоопределение языка через OSD
    pub fn detect_language(&mut self, image: &DynamicImage) -> Result<String, OcrError> {
        let luma = image.to_luma8();
        let mut png_buffer = Vec::new();
        luma.write_to(&mut Cursor::new(&mut png_buffer), image::ImageFormat::Png)
            .map_err(|e| OcrError::SetImageError(format!("Failed to encode image to PNG: {e}")))?;

        // Временно переключаемся на OSD mode
        self.api
            .set_variable(leptess::Variable::TesseditPagesegMode, "0") // PSM_OSD_ONLY
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        self.api
            .set_image_from_mem(&png_buffer)
            .map_err(|e| OcrError::SetImageError(e.to_string()))?;

        // Получаем "текст" — в OSD mode это метаданные ориентации
        let osd_text = self
            .api
            .get_utf8_text()
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        // Возвращаем PSM_AUTO
        self.api
            .set_variable(leptess::Variable::TesseditPagesegMode, "3")
            .map_err(|e| OcrError::RecognitionError(e.to_string()))?;

        // Парсим OSD результат
        // Формат: "Orientation: X\nOrientation in degrees: Y\n..."
        parse_osd_result(&osd_text)
    }

    /// Распознать цифры только (whitelist)
    pub fn recognize_digits(&mut self, image: &DynamicImage) -> Result<OcrResult, OcrError> {
        self.set_whitelist("0123456789")?;
        self.set_psm(10)?; // PSM_SINGLE_CHAR или 7 (PSM_SINGLE_LINE)

        let result = self.recognize(image)?;

        // Сбрасываем whitelist
        self.api
            .set_variable(leptess::Variable::TesseditCharWhitelist, "")
            .map_err(|e| OcrError::InitError(e.to_string()))?;
        self.set_psm(3)?;

        Ok(result)
    }

    /// Получить версию Tesseract
    pub fn get_version() -> String {
        // tesseract-rs не предоставляет прямого API для версии
        // Можно получить через tesseract-sys если нужно
        "5.x".to_string()
    }
}

/// Парсинг hOCR для извлечения bounding boxes
fn parse_hocr_boxes(hocr: &str) -> Result<Vec<TextBox>, OcrError> {
    use regex::Regex;

    let mut boxes = Vec::new();

    // Регулярка для извлечения bbox из title="bbox x1 y1 x2 y2"
    let bbox_re = Regex::new(r#"bbox (\d+) (\d+) (\d+) (\d+)"#)
        .map_err(|_| OcrError::RecognitionError("Invalid regex".to_string()))?;

    // Регулярка для извлечения текста
    // Простой парсинг: ищем <span class="ocrx_word" ...>текст</span>
    let word_re = Regex::new(
        r#"<span class="ocrx_word"[^>]*title="bbox (\d+) (\d+) (\d+) (\d+)[^"]*"[^>]*>([^<]*)</span>"#
    ).map_err(|_| OcrError::RecognitionError("Invalid regex".to_string()))?;

    for cap in word_re.captures_iter(hocr) {
        let x1: i32 = cap[1].parse().unwrap_or(0);
        let y1: i32 = cap[2].parse().unwrap_or(0);
        let x2: i32 = cap[3].parse().unwrap_or(0);
        let y2: i32 = cap[4].parse().unwrap_or(0);
        let text = cap[5].to_string();

        // Очищаем HTML entities
        let text = text
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        boxes.push(TextBox {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
            text,
            confidence: -1, // hOCR не содержит confidence per word
            level: TextLevel::Word,
        });
    }

    Ok(boxes)
}

/// Парсинг OSD результата
fn parse_osd_result(osd_text: &str) -> Result<String, OcrError> {
    // Простой парсинг, возвращаем "eng" как fallback
    // TODO: улучшить парсинг для определения реального языка

    if osd_text.contains("Orientation: 0") {
        Ok("eng".to_string())
    } else {
        Ok("eng".to_string())
    }
}

/// Утилита: распознать изображение с автоматическим определением лучшего языка
pub fn recognize_auto(
    tessdata_dir: &str,
    image: &DynamicImage,
    available_langs: &[String],
) -> Result<OcrResult, OcrError> {
    // Пробуем каждый язык и выбираем лучший по confidence
    let mut best_result: Option<OcrResult> = None;
    let mut best_confidence = 0.0f32;

    for lang in available_langs {
        let mut engine = OcrEngine::new(tessdata_dir, lang)?;

        match engine.recognize(image) {
            Ok(result) => {
                if result.confidence > best_confidence {
                    best_confidence = result.confidence;
                    best_result = Some(result);
                }
            }
            Err(e) => {
                tracing::warn!("OCR failed for lang '{}': {}", lang, e);
            }
        }
    }

    best_result.ok_or_else(|| OcrError::RecognitionError("All languages failed".to_string()))
}

/// Утилита: распознать изображение из файла
pub fn recognize_file(
    tessdata_dir: &str,
    language: &str,
    path: &std::path::Path,
) -> Result<OcrResult, OcrError> {
    let image = image::open(path).map_err(|e| OcrError::InvalidImage)?;

    let mut engine = OcrEngine::new(tessdata_dir, language)?;
    engine.recognize(&image)
}
