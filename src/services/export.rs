use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

pub enum ExportFormat {
    Txt,
    Pdf,
    Markdown,
    Html,
}

pub struct ExportService;

impl ExportService {
    pub fn export(
        &self,
        result: &OcrResult,
        format: ExportFormat,
        path: &std::path::Path,
    ) -> Result<(), ExportError> {
        match format {
            ExportFormat::Txt => self.export_txt(result, path)?,
            ExportFormat::Pdf => self.export_pdf(result, path)?,
            ExportFormat::Markdown => self.export_md(result, path)?,
            ExportFormat::Html => self.export_html(result, path)?,
        }
        Ok(())
    }

    fn export_pdf(&self, result: &OcrResult, path: &std::path::Path) -> Result<(), ExportError> {
        let doc = PdfDocument::empty("OCR Result");
        let (page, layer) = doc.add_page(
            Mm(210.0), // A4 width
            Mm(297.0), // A4 height
            "Layer 1",
        );

        let current_layer = doc.get_page(page).get_layer(layer);

        // Add text with proper encoding
        current_layer.use_text(
            &result.text,
            12.0, // font size
            Mm(10.0),
            Mm(280.0),
            &doc.add_builtin_font(BuiltinFont::Helvetica)?,
        );

        // Add image if available
        if let Some(img_path) = &result.image_path {
            let image = Image::from_path(img_path)?;
            image.add_to_layer(
                current_layer.clone(),
                ImageTransform {
                    translate_x: Some(Mm(10.0)),
                    translate_y: Some(Mm(150.0)),
                    scale_x: Some(0.5),
                    scale_y: Some(0.5),
                    ..Default::default()
                },
            );
        }

        doc.save(&mut BufWriter::new(File::create(path)?))?;
        Ok(())
    }

    fn export_md(&self, result: &OcrResult, path: &std::path::Path) -> Result<(), ExportError> {
        let content = format!(
            "# OCR Result\n\n**Language:** {}\n**Confidence:** {:.1}%\n\n---\n\n{}\n",
            result.language, result.confidence, result.text
        );
        std::fs::write(path, content)?;
        Ok(())
    }
}
