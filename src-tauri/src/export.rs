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
/// Writes a minimal PDF 1.4 by hand that embeds each JPEG page via DCTDecode.
/// Non-JPEG pages are wrapped as best-effort; the layout assumes JPEG. This
/// keeps the export dependency-free (no `lopdf` version drift).
pub fn export_pdf(folder: &Path, out: Option<&Path>) -> AppResult<PathBuf> {
    let files = list_image_files(folder);
    if files.is_empty() {
        return Err(AppError::Other("no pages to export".into()));
    }
    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => folder.with_extension("pdf"),
    };

    let mut writer = PdfWriter::new();
    for page_path in &files {
        let bytes = std::fs::read(page_path)?;
        let (w, h) = detect_jpeg_size(&bytes).unwrap_or((595, 842)); // A4 fallback in pt
        writer.add_jpeg_page(&bytes, w, h);
    }
    let pdf_bytes = writer.finish();
    std::fs::write(&out_path, pdf_bytes)?;
    Ok(out_path)
}

/// Minimal PDF writer that emits one page per JPEG.
struct PdfWriter {
    objects: Vec<Vec<u8>>, // object body bytes (object N is index N-1)
    page_refs: Vec<usize>, // object numbers of pages
}

impl PdfWriter {
    fn new() -> Self {
        // Object 1 reserved for Catalog; object 2 reserved for Pages.
        Self {
            objects: vec![Vec::new(), Vec::new()],
            page_refs: Vec::new(),
        }
    }

    fn alloc(&mut self) -> usize {
        self.objects.push(Vec::new());
        self.objects.len()
    }

    fn add_jpeg_page(&mut self, jpeg: &[u8], w: u32, h: u32) {
        // Image XObject: dictionary header + raw JPEG bytes + endstream.
        let image_id = self.alloc();
        let mut image_obj = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length {} >>\nstream\n",
            w, h, jpeg.len()
        )
        .into_bytes();
        image_obj.extend_from_slice(jpeg);
        image_obj.extend_from_slice(b"\nendstream");
        self.objects[image_id - 1] = image_obj;

        // Page content stream: scale image to fill the page.
        let content_id = self.alloc();
        let content = format!("q {} 0 0 {} 0 0 cm /Im{} Do Q", w, h, self.page_refs.len());
        self.objects[content_id - 1] = format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            content.len(),
            content
        )
        .into_bytes();

        // Page object referencing the content stream and the image XObject.
        let page_id = self.alloc();
        self.objects[page_id - 1] = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Contents {} 0 R /Resources << /XObject << /Im{} {} 0 R >> >> >>",
            w, h, content_id, self.page_refs.len(), image_id
        )
        .into_bytes();
        self.page_refs.push(page_id);
    }

    fn finish(mut self) -> Vec<u8> {
        // Catalog (object 1) points to Pages (object 2).
        self.objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
        // Pages (object 2) lists kids.
        let kids: String = self
            .page_refs
            .iter()
            .map(|r| format!("{} 0 R", r))
            .collect::<Vec<_>>()
            .join(" ");
        self.objects[1] = format!(
            "<< /Type /Pages /Kids [ {} ] /Count {} >>",
            kids,
            self.page_refs.len()
        )
        .into_bytes();

        let mut out = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

        let mut offsets: Vec<usize> = Vec::with_capacity(self.objects.len());
        for (i, body) in self.objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend_from_slice(body);
            out.extend_from_slice(b"\nendobj\n");
        }

        let xref_offset = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n", self.objects.len() + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for off in &offsets {
            out.extend_from_slice(format!("{:010} 00000 n \n", off).as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
                self.objects.len() + 1,
                xref_offset
            )
            .as_bytes(),
        );
        out
    }
}

/// Extract JPEG dimensions (w, h) from a JFIF/EXIF stream by scanning for the
/// SOF0/SOF2 marker. Returns None for non-JPEG data.
fn detect_jpeg_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut i = 2usize;
    while i + 9 < bytes.len() {
        if bytes[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = bytes[i + 1];
        i += 2;
        // SOF0..=SOF15 (excluding SOF4, SOF8, SOF12 which are rare).
        if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            let h = ((bytes[i + 3] as u32) << 8) | bytes[i + 4] as u32;
            let w = ((bytes[i + 5] as u32) << 8) | bytes[i + 6] as u32;
            return Some((w, h));
        }
        let seg_len = ((bytes[i] as usize) << 8) | bytes[i + 1] as usize;
        i += seg_len;
    }
    None
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
