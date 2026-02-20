use anyhow::{Context, Result};
use image::{codecs::webp::WebPEncoder, ImageEncoder};
use resvg::{tiny_skia, usvg};
use std::io::Cursor;

/// Represents WebP quality configuration
#[derive(Debug, Clone, Copy)]
pub enum WebpQuality {
    Lossless,
    Lossy(u8), // 0 to 100
}

/// Converts SVG bytes to WebP format.
pub async fn svg_to_webp(
    svg_bytes: &[u8],
    _quality: WebpQuality,
    fonts: &[String],
    cache_dir: Option<&std::path::Path>,
) -> Result<Vec<u8>> {
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();

    if !fonts.is_empty() {
        let font_mgr = crate::utils::font_manager::FontManager::new(cache_dir)?;
        let downloaded_dir = font_mgr.prepare_fonts(fonts).await?;
        opt.fontdb_mut().load_font_dir(&downloaded_dir);
    }

    let tree = usvg::Tree::from_data(svg_bytes, &opt)
        .context("Failed to parse SVG for WebP conversion")?;

    let size = tree.size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width() as u32, size.height() as u32)
        .context("Failed to allocate Pixmap for WebP conversion")?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let mut output = Cursor::new(Vec::new());

    let (width, height) = (pixmap.width(), pixmap.height());
    let data = pixmap.data(); // RGBA bytes

    let encoder = WebPEncoder::new_lossless(&mut output);
    encoder
        .write_image(data, width, height, image::ExtendedColorType::Rgba8)
        .context("Failed to encode lossless WebP")?;

    Ok(output.into_inner())
}

/// Converts PNG bytes to WebP format (useful for ditaa fallback)
pub async fn png_to_webp(png_bytes: &[u8], _quality: WebpQuality) -> Result<Vec<u8>> {
    let img = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)
        .context("Failed to decode PNG for WebP conversion")?;

    let mut output = Cursor::new(Vec::new());

    let encoder = WebPEncoder::new_lossless(&mut output);
    encoder
        .write_image(
            img.as_bytes(),
            img.width(),
            img.height(),
            img.color().into(),
        )
        .context("Failed to encode lossless WebP from PNG")?;

    Ok(output.into_inner())
}
