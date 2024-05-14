use clap::Parser;
use nalgebra::{Isometry2, Point2, Vector2};
use pdfium_render::points::PdfPoints;
use pdfium_render::prelude::*;
use serde::Deserialize;
use std::io::{Read, Write};
use tracing::{debug, span, Level};

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

fn mark_pdf(
    pdf: &[u8],
    text: &str,
    text_size: f32,
    padding_w: PdfPoints,
    padding_h: PdfPoints,
    theta: f32,
) -> Result<Vec<u8>, PdfiumError> {
    let _span = span!(Level::DEBUG, "mark_pdf").entered();

    let pdfium = Pdfium::default();

    let mut document = pdfium.load_pdf_from_byte_slice(pdf, None)?;
    debug!("pdf document loaded");

    let font = document.fonts_mut().helvetica();

    document.pages().watermark(|group, index, page_w, page_h| {
        let _span = span!(Level::DEBUG, "page", page = index).entered();

        let watermark = PdfPageTextObject::new(&document, text, font, PdfPoints::new(text_size))?;
        let (w, h) = (watermark.width()?, watermark.height()?);
        let (w, h) = (w + padding_w, h + padding_h);

        let (w_max_r, _) = norm_to_rot(theta, page_w, page_w, page_h);
        let (_, h_max_r) = norm_to_rot(theta, page_w, PdfPoints::ZERO, page_h);

        let total_i = (w_max_r.value / w.value).ceil() as u32;
        let total_j = (h_max_r.value / h.value).ceil() as u32;
        debug!(
            "total watermarks on page {} = {} x {}",
            index, total_i, total_j
        );

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

#[derive(Parser, Default, Deserialize)]
pub struct MarkQuery {
    #[arg(long)]
    text: String,
    #[arg(long)]
    font_size: f32,
    #[arg(long)]
    padding_w: f32,
    #[arg(long)]
    padding_h: f32,
    #[arg(long)]
    rot_deg: f32,
}

enum ProcessError {
    IoError(std::io::Error),
    PdfiumError(PdfiumError),
}

impl From<std::io::Error> for ProcessError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<PdfiumError> for ProcessError {
    fn from(value: PdfiumError) -> Self {
        Self::PdfiumError(value)
    }
}

fn process() -> Result<(), ProcessError> {
    let query = MarkQuery::parse();
    let mut pdf = Vec::new();
    std::io::stdin().read_to_end(&mut pdf)?;
    let marked = mark_pdf(
        &pdf,
        &query.text,
        query.font_size,
        PdfPoints::from_mm(query.padding_w),
        PdfPoints::from_mm(query.padding_h),
        query.rot_deg,
    )?;
    std::io::stdout().write_all(&marked)?;
    Ok(())
}

fn main() {
    match process() {
        Ok(_) => {}
        Err(ProcessError::IoError(e)) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
        Err(ProcessError::PdfiumError(PdfiumError::PdfiumLibraryInternalError(
            PdfiumInternalError::FormatError,
        ))) => {
            std::process::exit(2);
        }
        Err(ProcessError::PdfiumError(e)) => {
            eprint!("{}", e);
            std::process::exit(1);
        }
    }
}
