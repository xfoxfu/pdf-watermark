use nalgebra::{Isometry2, Point2, Vector2};
use pdfium_render::prelude::*;

fn norm_to_rot(
    theta: f32,
    page_w: PdfPoints,
    xx: PdfPoints,
    yy: PdfPoints,
) -> (PdfPoints, PdfPoints) {
    let point = Point2::new(xx.value, yy.value);
    let proj = Isometry2::new(
        Vector2::new(0.0, page_w.value * theta.to_radians().sin()),
        -theta.to_radians(),
    );

    let point = proj.transform_point(&point);
    let (x, y) = (PdfPoints::new(point.x), PdfPoints::new(point.y));
    let (x, y) = (x, y);

    (x, y)
}

fn rot_to_norm(
    theta: f32,
    page_w: PdfPoints,
    x: PdfPoints,
    y: PdfPoints,
) -> (PdfPoints, PdfPoints) {
    let point = Point2::new(x.value, y.value);
    let proj = Isometry2::new(
        Vector2::new(0.0, page_w.value * theta.to_radians().sin()),
        -theta.to_radians(),
    );

    let point = proj.inverse_transform_point(&point);
    let (x, y) = (PdfPoints::new(point.x), PdfPoints::new(point.y));
    let (x, y) = (x, y);

    (x, y)
}

pub fn mark_pdf(
    pdf: &[u8],
    text: &str,
    text_size: f32,
    padding_w: PdfPoints,
    padding_h: PdfPoints,
    theta: f32,
) -> Result<Vec<u8>, PdfiumError> {
    let pdfium = Pdfium::default();

    let mut document = pdfium.load_pdf_from_byte_slice(pdf, None)?;

    let font = document.fonts_mut().helvetica();

    document
        .pages()
        .watermark(|group, _index, page_w, page_h| {
            let watermark =
                PdfPageTextObject::new(&document, text, font, PdfPoints::new(text_size))?;
            let (w, h) = (watermark.width()?, watermark.height()?);
            let (w, h) = (w + padding_w, h + padding_h);

            let (w_max_r, _) = norm_to_rot(theta, page_w, page_w, page_h);
            let (_, h_max_r) = norm_to_rot(theta, page_w, PdfPoints::ZERO, page_h);

            let total_i = (w_max_r.value / w.value).ceil() as u32;
            let total_j = (h_max_r.value / h.value).ceil() as u32;

            for i in 0..total_i {
                for j in 0..total_j {
                    let (x, y) = (w * i as f32, h * j as f32);
                    let (xx, yy) = rot_to_norm(theta, page_w, x, y);
                    let (xx, yy) = (xx - h * theta.to_radians().sin(), yy);

                    let mut watermark =
                        PdfPageTextObject::new(&document, text, font, PdfPoints::new(text_size))?;
                    watermark.rotate_counter_clockwise_degrees(theta)?;
                    watermark.set_fill_color(PdfColor::GREY.with_alpha(64))?;
                    watermark.translate(xx, yy)?;
                    group.push(&mut watermark.into())?;
                }
            }

            Ok(())
        })?;

    document.save_to_bytes()
}
