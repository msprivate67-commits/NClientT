//! Gallery export to ZIP / PDF.
//!
//! Port of NClientV3's `PDF` + `ZIP` channels. Given a gallery folder on
//! disk, packs its page images into a single archive / document.

use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

/// Pack a downloaded gallery folder into a `.zip` next to it (or to `out`).
pub fn export_zip(folder: &Path, out: Option<&Path>) -> AppResult<PathBuf> {
    let files = list_image_files(folder);
    if files.is_empty() {
        return Err(AppError::Other("no pages to export".into()));
    }
    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => folder.with_extension("zip"),
    };
    let file = std::fs::File::create(&out_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for (i, page) in files.iter().enumerate() {
        let path = Path::new(page);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let entry = format!("{:03}.{}", i + 1, ext);
        zip.start_file(entry, opts)?;
        let data = std::fs::read(path)?;
        std::io::Write::write_all(&mut zip, &data)?;
    }
    zip.finish()?;
    Ok(out_path)
}

/// Pack a downloaded gallery folder into a `.pdf` next to it (or to `out`).
/// Uses `lopdf` to embed each page image (JPEG via DCTDecode, PNG via raw).
pub fn export_pdf(folder: &Path, out: Option<&Path>) -> AppResult<PathBuf> {
    let files = list_image_files(folder);
    if files.is_empty() {
        return Err(AppError::Other("no pages to export".into()));
    }
    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => folder.with_extension("pdf"),
    };

    use lopdf::{dictionary, Document, Object, ObjectId};
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let mut page_ids: Vec<ObjectId> = Vec::new();

    for (idx, page_path) in files.iter().enumerate() {
        let bytes = std::fs::read(page_path)?;
        let (width, height, filter) = match detect_format(&bytes) {
            Jpeg(w, h) => (w, h, "DCTDecode"),
            Png(w, h) => (w, h, "FlateDecode"),
            ImageFormatUnknown => (1, 1, "DCTDecode"),
        };

        let stream = lopdf::Stream::new(
            dictionary! {
                "Type" => "XObject",
                "Subtype" => "Image",
                "Width" => width,
                "Height" => height,
                "ColorSpace" => "DeviceRGB",
                "BitsPerComponent" => 8,
                "Filter" => filter,
            },
            bytes,
        );
        let image_id = doc.add_object(stream);
        let image_name = format!("Im{}", idx);

        let content = format!("q {} 0 0 {} 0 0 cm /{} Do Q", width, height, image_name);
        let content_id = doc.add_object(lopdf::Stream::new(dictionary! {}, content.into_bytes()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![Object::Integer(0), Object::Integer(0),
                               Object::Integer(width), Object::Integer(height)],
            "Contents" => content_id,
            "Resources" => dictionary! {
                "XObject" => dictionary! {
                    image_name => image_id,
                }
            },
        });
        page_ids.push(page_id);
    }

    doc.set_object(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Count" => page_ids.len() as i64,
            "Kids" => page_ids.into_iter().map(Object::Reference).collect::<Vec<_>>(),
        }),
    );

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });
    doc.set_catalog(catalog_id);

    doc.save(&out_path)?;
    Ok(out_path)
}

enum ImageFormat {
    Jpeg(i64, i64),
    Png(i64, i64),
    ImageFormatUnknown,
}

use ImageFormat::*;

fn detect_format(bytes: &[u8]) -> ImageFormat {
    if bytes.len() < 24 {
        return ImageFormatUnknown;
    }
    // PNG: 89 50 4E 47
    if &bytes[0..4] == &[0x89, 0x50, 0x4E, 0x47] {
        let w = i64::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = i64::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        return Png(w, h);
    }
    // JPEG: FF D8 FF ... SOF0/2 marker FF C0/C2 holds H+W.
    if bytes[0] == 0xFF && bytes[1] == 0xD8 {
        let mut i = 2usize;
        while i + 9 < bytes.len() {
            if bytes[i] != 0xFF {
                i += 1;
                continue;
            }
            let marker = bytes[i + 1];
            i += 2;
            if marker == 0xC0 || marker == 0xC2 {
                let h = ((bytes[i + 3] as i64) << 8) | bytes[i + 4] as i64;
                let w = ((bytes[i + 5] as i64) << 8) | bytes[i + 6] as i64;
                return Jpeg(w, h);
            }
            let seg_len = ((bytes[i] as usize) << 8) | bytes[i + 1] as usize;
            i += seg_len;
        }
    }
    ImageFormatUnknown
}

fn list_image_files(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(folder) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
                || lower.ends_with(".png")
                || lower.ends_with(".gif")
                || lower.ends_with(".webp")
            {
                files.push(e.path());
            }
        }
    }
    files.sort();
    files
}
