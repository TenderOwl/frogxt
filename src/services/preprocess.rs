use imageproc::{
    contrast::adaptive_threshold,
    filter::median_filter,
    geometric_transformations::{rotate_about_center, Interpolation},
    image::{self, DynamicImage, GrayImage, ImageBuffer, Luma, Rgba},
};
use std::f32::consts::PI;

/// Ошибки препроцессинга
#[derive(thiserror::Error, Debug)]
pub enum PreprocessError {
    #[error("Failed to convert image: {0}")]
    ConversionError(String),

    #[error("Image too small: {0}x{1}")]
    ImageTooSmall(u32, u32),

    #[error("Deskew failed: {0}")]
    DeskewError(String),
}

/// Параметры препроцессинга
#[derive(Debug, Clone, Copy)]
pub struct PreprocessConfig {
    /// Увеличивать ли мелкие изображения (supersampling)
    pub enable_supersampling: bool,
    /// Порог для supersampling (если ширина/высота меньше — увеличиваем)
    pub supersampling_threshold: u32,
    /// Масштаб supersampling (обычно 2x)
    pub supersampling_scale: f32,
    /// Применять ли denoising (median filter)
    pub enable_denoise: bool,
    /// Радиус median filter
    pub median_radius: u32,
    /// Применять ли adaptive thresholding
    pub enable_threshold: bool,
    /// Размер окна для adaptive threshold
    pub threshold_window_size: u32,
    /// Применять ли deskew (исправление наклона)
    pub enable_deskew: bool,
    /// Максимальный угол для deskew в градусах
    pub max_skew_angle: f32,
    /// Применять ли контрастирование
    pub enable_contrast: bool,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            enable_supersampling: true,
            supersampling_threshold: 1000,
            supersampling_scale: 2.0,
            enable_denoise: true,
            median_radius: 1,
            enable_threshold: true,
            threshold_window_size: 15,
            enable_deskew: true,
            max_skew_angle: 5.0,
            enable_contrast: true,
        }
    }
}

/// Результат препроцессинга с метаданными
#[derive(Debug, Clone)]
pub struct PreprocessedImage {
    pub image: DynamicImage,
    /// Применённые шаги препроцессинга
    pub applied_steps: Vec<String>,
    /// Угол наклона до коррекции (в градусах)
    pub original_skew: f32,
    /// Масштаб, применённый при supersampling
    pub scale_factor: f32,
}

/// Основная функция препроцессинга
pub fn preprocess_image(
    image: &DynamicImage,
    config: &PreprocessConfig,
) -> Result<PreprocessedImage, PreprocessError> {
    let mut applied_steps = Vec::new();
    let mut scale_factor = 1.0f32;
    let mut original_skew = 0.0f32;

    // Шаг 1: Конвертация в RGBA8 (унификация формата)
    let mut img = image.to_rgba8();
    applied_steps.push("to_rgba8".to_string());

    // Шаг 2: Grayscale
    let mut gray = DynamicImage::ImageRgba8(img).to_luma8();
    applied_steps.push("grayscale".to_string());

    // Шаг 3: Supersampling для мелкого текста
    if config.enable_supersampling {
        let (w, h) = gray.dimensions();
        if w < config.supersampling_threshold || h < config.supersampling_threshold {
            let new_w = (w as f32 * config.supersampling_scale) as u32;
            let new_h = (h as f32 * config.supersampling_scale) as u32;

            gray =
                image::imageops::resize(&gray, new_w, new_h, image::imageops::FilterType::Lanczos3);
            scale_factor = config.supersampling_scale;
            applied_steps.push(format!("supersample {}x{} → {}x{}", w, h, new_w, new_h));
        }
    }

    // Шаг 4: Контрастирование (CLAHE или histogram stretching)
    if config.enable_contrast {
        gray = apply_clahe(&gray, 8, 2.0)?;
        applied_steps.push("clahe".to_string());
    }

    // Шаг 5: Denoising (median filter)
    if config.enable_denoise {
        gray = median_filter(&gray, config.median_radius, config.median_radius);
        applied_steps.push(format!("median_filter(r={})", config.median_radius));
    }

    // Шаг 6: Deskew (определение и коррекция угла наклона)
    if config.enable_deskew {
        let skew = detect_skew_angle(&gray)?;
        original_skew = skew;

        if skew.abs() > 0.3 && skew.abs() <= config.max_skew_angle {
            let rotated = rotate_about_center(
                &gray,
                -skew * PI / 180.0, // конвертация в радианы
                Interpolation::Bilinear,
                Luma([255]),
            );
            gray = rotated;
            applied_steps.push(format!("deskew({:.1}°)", skew));
        }
    }

    // Шаг 7: Adaptive thresholding
    if config.enable_threshold {
        gray = adaptive_threshold(&gray, config.threshold_window_size as u32, 180);
        applied_steps.push(format!(
            "adaptive_threshold(w={})",
            config.threshold_window_size
        ));
    }

    Ok(PreprocessedImage {
        image: DynamicImage::ImageLuma8(gray),
        applied_steps,
        original_skew,
        scale_factor,
    })
}

/// Упрощённый препроцессинг с дефолтными настройками
pub fn preprocess(image: &DynamicImage) -> Result<DynamicImage, PreprocessError> {
    let result = preprocess_image(image, &PreprocessConfig::default())?;
    Ok(result.image)
}

/// CLAHE (Contrast Limited Adaptive Histogram Equalization)
fn apply_clahe(
    image: &GrayImage,
    tile_size: u32,
    clip_limit: f32,
) -> Result<GrayImage, PreprocessError> {
    let (w, h) = image.dimensions();
    let mut output = ImageBuffer::new(w, h);

    // Упрощённая реализация: tile-based histogram equalization
    let tiles_x = (w / tile_size).max(1);
    let tiles_y = (h / tile_size).max(1);

    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            let x0 = tx * tile_size;
            let y0 = ty * tile_size;
            let x1 = ((tx + 1) * tile_size).min(w);
            let y1 = ((ty + 1) * tile_size).min(h);

            // Вычисляем гистограмму для тайла
            let mut hist = [0u32; 256];
            let mut total = 0u32;

            for y in y0..y1 {
                for x in x0..x1 {
                    let val = image.get_pixel(x, y)[0] as usize;
                    hist[val] += 1;
                    total += 1;
                }
            }

            // Clip limit
            let clip = (clip_limit * total as f32 / 256.0) as u32;
            let mut clipped = 0u32;
            for i in 0..256 {
                if hist[i] > clip {
                    clipped += hist[i] - clip;
                    hist[i] = clip;
                }
            }
            let redist = clipped / 256;
            for i in 0..256 {
                hist[i] += redist;
            }

            // CDF
            let mut cdf = [0u32; 256];
            cdf[0] = hist[0];
            for i in 1..256 {
                cdf[i] = cdf[i - 1] + hist[i];
            }

            // Применяем к пикселям
            let cdf_min = cdf.iter().find(|&&x| x > 0).copied().unwrap_or(0);
            let denom = total - cdf_min;

            for y in y0..y1 {
                for x in x0..x1 {
                    let val = image.get_pixel(x, y)[0] as usize;
                    let new_val = if denom > 0 {
                        ((cdf[val] - cdf_min) as f32 / denom as f32 * 255.0) as u8
                    } else {
                        val as u8
                    };
                    output.put_pixel(x, y, Luma([new_val]));
                }
            }
        }
    }

    Ok(output)
}

/// Определение угла наклона текста (Projection Profile Method)
fn detect_skew_angle(image: &GrayImage) -> Result<f32, PreprocessError> {
    let (w, h) = image.dimensions();

    if w < 50 || h < 50 {
        return Ok(0.0);
    }

    // Тестируем углы от -5 до +5 градусов с шагом 0.5
    let angles: Vec<f32> = (-10..=10).map(|i| i as f32 * 0.5).collect();
    let mut best_angle = 0.0f32;
    let mut best_variance = 0.0f32;

    for &angle in &angles {
        let rotated = if angle.abs() < 0.1 {
            image.clone()
        } else {
            rotate_about_center(
                image,
                -angle * PI / 180.0,
                Interpolation::Nearest,
                Luma([255]),
            )
        };

        // Вычисляем projection profile (сумма чёрных пикселей по строкам)
        let profile: Vec<u32> = (0..rotated.height())
            .map(|y| {
                (0..rotated.width())
                    .filter(|&x| rotated.get_pixel(x, y)[0] < 128)
                    .count() as u32
            })
            .collect();

        // Вычисляем variance — чем выше, тем более "пиковый" профиль
        let mean = profile.iter().sum::<u32>() as f32 / profile.len() as f32;
        let variance = profile
            .iter()
            .map(|&v| {
                let diff = v as f32 - mean;
                diff * diff
            })
            .sum::<f32>();

        if variance > best_variance {
            best_variance = variance;
            best_angle = angle;
        }
    }

    // Уточняем с шагом 0.1 вокруг лучшего угла
    let fine_angles: Vec<f32> = (-5..=5).map(|i| best_angle + i as f32 * 0.1).collect();
    best_variance = 0.0;

    for &angle in &fine_angles {
        let rotated = if angle.abs() < 0.1 {
            image.clone()
        } else {
            rotate_about_center(
                image,
                -angle * PI / 180.0,
                Interpolation::Nearest,
                Luma([255]),
            )
        };

        let profile: Vec<u32> = (0..rotated.height())
            .map(|y| {
                (0..rotated.width())
                    .filter(|&x| rotated.get_pixel(x, y)[0] < 128)
                    .count() as u32
            })
            .collect();

        let mean = profile.iter().sum::<u32>() as f32 / profile.len() as f32;
        let variance = profile
            .iter()
            .map(|&v| {
                let diff = v as f32 - mean;
                diff * diff
            })
            .sum::<f32>();

        if variance > best_variance {
            best_variance = variance;
            best_angle = angle;
        }
    }

    Ok(best_angle)
}

/// Быстрый препроцессинг для скриншотов (меньше шагов, быстрее)
pub fn preprocess_screenshot(image: &DynamicImage) -> Result<DynamicImage, PreprocessError> {
    let config = PreprocessConfig {
        enable_supersampling: true,
        supersampling_threshold: 800,
        supersampling_scale: 1.5,
        enable_denoise: true,
        median_radius: 1,
        enable_threshold: false, // скриншоты обычно чистые
        threshold_window_size: 15,
        enable_deskew: true,
        max_skew_angle: 3.0, // скриншоты редко сильно наклонены
        enable_contrast: true,
    };

    let result = preprocess_image(image, &config)?;
    Ok(result.image)
}

/// Агрессивный препроцессинг для фотографий/сканов
pub fn preprocess_photo(image: &DynamicImage) -> Result<DynamicImage, PreprocessError> {
    let config = PreprocessConfig {
        enable_supersampling: true,
        supersampling_threshold: 1200,
        supersampling_scale: 2.0,
        enable_denoise: true,
        median_radius: 2,
        enable_threshold: true,
        threshold_window_size: 25,
        enable_deskew: true,
        max_skew_angle: 15.0,
        enable_contrast: true,
    };

    let result = preprocess_image(image, &config)?;
    Ok(result.image)
}
