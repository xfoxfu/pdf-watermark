use crate::utils::pdf_points::PdfPoints;
use crate::utils::text_width::helvetica_width;
use crate::AppResult;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{AppendHeaders, IntoResponse};
use axum::{body::Bytes, extract::Query};
use lopdf::content::{Content, Operation};
use lopdf::Document;
use lopdf::Object;
use lopdf::{dictionary, Dictionary};
use nalgebra::{Isometry2, Point2, Vector2};
use serde::Deserialize;
use tracing::info;

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
    doc: &[u8],
    text: &str,
    font_size_pt: f32,
    theta_deg: f32,
) -> Result<Vec<u8>, lopdf::Error> {
    let font_size = PdfPoints::new(font_size_pt);
    let w = PdfPoints::new(helvetica_width(text, font_size_pt));
    let h = font_size;
    let padding_w = PdfPoints::new(helvetica_width("xxxxxx", font_size_pt));
    let padding_h = font_size / 2.0;
    let (w, h) = (w + padding_w, h + padding_h);
    let theta_rad = theta_deg.to_radians();

    let mut doc = Document::load_mem(doc)?;

    // font
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Helvetica",
    });
    // graphics state (for opacity)
    let gs_id = doc.add_object(dictionary! {
        "ca" => 0.05
    });

    for page_id in doc.page_iter().collect::<Vec<_>>().into_iter() {
        // add font to page
        let resources_dict = doc.get_or_create_resources(page_id)?.as_dict_mut()?;
        if !resources_dict.has(b"Font") {
            resources_dict.set(b"Font", Dictionary::new());
        }
        resources_dict
            .get_mut("Font".as_bytes())?
            .as_dict_mut()?
            .set("F_VATPRC", font_id);
        // add graphics state to page
        doc.add_graphics_state(page_id, "GS_VATPRC", gs_id)?;

        // calculate page size
        let page_media_box = doc
            .get_object(page_id)?
            .as_dict()?
            .get(b"MediaBox")?
            .as_array()?;
        let (page_w, page_h) = (
            PdfPoints::new(page_media_box[2].as_float()?),
            PdfPoints::new(page_media_box[3].as_float()?),
        );

        // calculate watermark count
        let (w_max_r, _) = norm_to_rot(theta_deg, page_w, page_w, page_h);
        let (_, h_max_r) = norm_to_rot(theta_deg, page_w, PdfPoints::zero(), page_h);

        let total_i = (w_max_r.value / w.value).ceil() as u32;
        let total_j = (h_max_r.value / h.value).ceil() as u32;

        // generate watermarks
        // see https://github.com/Hopding/pdf-lib/blob/master/src/api/operations.ts#L52
        let mut operations = vec![
            // enter graphics group
            Operation::new("q", vec![]),
            // set graphics state
            Operation::new("gs", vec!["GS_VATPRC".into()]),
            // begin text region
            Operation::new("BT", vec![]),
            // set font
            Operation::new("Tf", vec!["F_VATPRC".into(), font_size.value.into()]),
        ];

        for i in 0..total_i {
            for j in 0..total_j {
                let (x, y) = (w * i as f32, h * j as f32);
                let (xx, yy) = rot_to_norm(theta_deg, page_w, x, y);
                let (xx, yy) = (xx - h * theta_rad.sin(), yy);
                // set transform matrix
                operations.push(Operation::new(
                    "Tm",
                    vec![
                        theta_rad.cos().into(),
                        theta_rad.sin().into(),
                        (-theta_rad.sin()).into(),
                        theta_rad.cos().into(),
                        xx.value.into(),
                        yy.value.into(),
                    ],
                ));
                // draw text
                operations.push(Operation::new("Tj", vec![Object::string_literal(text)]));
            }
        }

        // end text region
        operations.push(Operation::new("ET", vec![]));
        // end graphics group
        operations.push(Operation::new("Q", vec![]));

        let content = Content { operations };
        doc.add_to_page_content(page_id, content)?;
    }

    let mut vec = Vec::new();
    doc.save_to(&mut vec)?;
    Ok(vec)
}

#[derive(Default, Deserialize)]
pub struct MarkQuery {
    text: String,
    font_size: f32,
    #[allow(dead_code)]
    padding_w: f32,
    #[allow(dead_code)]
    padding_h: f32,
    rot_deg: f32,
}

pub async fn mark(query: Query<MarkQuery>, pdf: Bytes) -> AppResult<impl IntoResponse> {
    info!("request received");

    let result = mark_pdf(&pdf, &query.text, query.font_size, query.rot_deg)?;

    Ok((
        AppendHeaders([
            (CONTENT_TYPE, "application/pdf"),
            (CONTENT_DISPOSITION, "inline"),
        ]),
        result,
    ))
}
