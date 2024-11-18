use self::pdf_points::PdfPoints;
use self::text_width::helvetica_width;
use crate::AppResult;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::{AppendHeaders, IntoResponse};
use axum::{body::Bytes, extract::Query};
use lopdf::content::{Content, Operation};
use lopdf::Document;
use lopdf::Object;
use lopdf::{dictionary, Dictionary};
use nalgebra::{Isometry2, Matrix3, Point2, Vector2};
use serde::Deserialize;
use tracing::info;

fn norm_to_rot(theta: f32, page_w: PdfPoints, xx: PdfPoints, yy: PdfPoints) -> (PdfPoints, PdfPoints) {
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

fn rot_to_norm(theta: f32, page_w: PdfPoints, x: PdfPoints, y: PdfPoints) -> (PdfPoints, PdfPoints) {
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

fn mark_pdf(doc: &[u8], text: &str, font_size_pt: f32, theta_deg: f32) -> anyhow::Result<Vec<u8>> {
  let doc = qpdf::QPdf::read_from_memory(doc)?;
  let doc = doc.writer().preserve_encryption(false).write_to_memory()?;

  let font_size = PdfPoints::new(font_size_pt);
  let w = PdfPoints::new(helvetica_width(text, font_size_pt));
  let h = font_size;
  let padding_w = PdfPoints::new(helvetica_width("xxxxxx", font_size_pt));
  let padding_h = font_size / 2.0;
  let (w, h) = (w + padding_w, h + padding_h);
  let theta_rad = theta_deg.to_radians();

  let mut doc = Document::load_mem(&doc)?;

  // font
  let font_id = doc.add_object(dictionary! {
      "Type" => "Font",
      "Subtype" => "Type1",
      "BaseFont" => "Helvetica",
  });
  // graphics state (for opacity)
  let gs_id = doc.add_object(dictionary! { "ca" => 0.05 });

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
    let page_media_box = doc.get_object(page_id)?.as_dict()?.get(b"MediaBox")?.as_array()?;
    let (page_w, page_h) = (
      PdfPoints::new(page_media_box[2].as_float()?),
      PdfPoints::new(page_media_box[3].as_float()?),
    );

    // calculate watermark count
    let (w_max_r, _) = norm_to_rot(theta_deg, page_w, page_w, page_h);
    let (_, h_max_r) = norm_to_rot(theta_deg, page_w, PdfPoints::zero(), page_h);

    let total_i = (w_max_r.value / w.value).ceil() as u32;
    let total_j = (h_max_r.value / h.value).ceil() as u32;

    // compute previous translation
    let mut matrix = Matrix3::<f32>::identity();
    let mut groups = Vec::new();
    for operation in Content::decode(&doc.get_page_content(page_id)?)?.operations {
      match operation.operator.as_ref() {
        "cm" => {
          if groups.is_empty() {
            matrix = Matrix3::<f32>::new(
              operation.operands[0].as_float()?,
              operation.operands[1].as_float()?,
              operation.operands[4].as_float()?,
              operation.operands[2].as_float()?,
              operation.operands[3].as_float()?,
              operation.operands[5].as_float()?,
              0.0,
              0.0,
              1.0,
            ) * matrix
          }
        }
        "q" => groups.push(()),
        "Q" => {
          groups.pop();
        }
        _ => {}
      }
    }
    let inv = matrix.try_inverse().unwrap_or_else(Matrix3::identity);

    // generate watermarks
    // see https://github.com/Hopding/pdf-lib/blob/master/src/api/operations.ts#L52
    let mut operations = vec![
      // enter graphics group
      Operation::new("q", vec![]),
      // cancel any previous translation
      Operation::new(
        "cm",
        vec![
          inv.m11.into(),
          inv.m12.into(),
          inv.m21.into(),
          inv.m22.into(),
          inv.m13.into(),
          inv.m23.into(),
        ],
      ),
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

  let doc = qpdf::QPdf::read_from_memory(vec)?;
  let vec = doc
    .writer()
    .encryption_params(qpdf::EncryptionParams::R6(qpdf::EncryptionParamsR6 {
      user_password: "".to_owned(),
      owner_password: "9bd33204-8c7c-4cc2-8c22-70e09062c102".to_owned(),
      allow_accessibility: false,
      allow_extract: false,
      allow_assemble: false,
      allow_annotate_and_form: false,
      allow_form_filling: false,
      allow_modify_other: false,
      allow_print: qpdf::PrintPermission::Low,
      encrypt_metadata: false,
    }))
    .write_to_memory()?;
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
    AppendHeaders([(CONTENT_TYPE, "application/pdf"), (CONTENT_DISPOSITION, "inline")]),
    result,
  ))
}

mod pdf_points {
  //! Copied from https://docs.rs/pdfium-render/0.8.20/src/pdfium_render/points.rs.html#12-14
  //! Licensed under either of
  //!
  //! * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
  //! * MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
  //!
  //! at your option.

  #![allow(dead_code)]

  use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

  /// The internal coordinate system inside a `PdfDocument` is measured in Points, a
  /// device-independent unit equal to 1/72 inches, roughly 0.358 mm. Points are converted to pixels
  /// when a `PdfPage` is rendered into a `PdfBitmap`.
  #[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
  pub struct PdfPoints {
    pub value: f32,
  }

  impl PdfPoints {
    /// A [PdfPoints] object with identity value 0.0.
    pub const ZERO: PdfPoints = PdfPoints::zero();

    /// A [PdfPoints] object with the largest addressable finite positive value.
    pub const MAX: PdfPoints = PdfPoints::max();

    /// A [PdfPoints] object with the smallest addressable finite negative value.
    pub const MIN: PdfPoints = PdfPoints::min();

    /// Creates a new [PdfPoints] object with the given value.
    #[inline]
    pub const fn new(value: f32) -> Self {
      Self { value }
    }

    /// Creates a new [PdfPoints] object with the value 0.0.
    ///
    /// Consider using the compile-time constant value [PdfPoints::ZERO]
    /// rather than calling this function directly.
    #[inline]
    pub const fn zero() -> Self {
      Self::new(0.0)
    }

    /// A [PdfPoints] object with the largest addressable finite positive value.
    ///
    /// In theory, this should be [f32::MAX]; in practice, values approaching [f32::MAX]
    /// are handled inconsistently by Pdfium, so this value is set to an arbitrarily large
    /// positive value that does not approach [f32::MAX] but should more than suffice
    /// for every use case.
    #[inline]
    pub const fn max() -> Self {
      Self::new(2_000_000_000.0)
    }

    /// A [PdfPoints] object with the smallest addressable finite negative value.
    ///
    /// In theory, this should be [f32::MIN]; in practice, values approaching [f32::MIN]
    /// are handled inconsistently by Pdfium, so this value is set to an arbitrarily large
    /// negative value that does not approach [f32::MIN] but should more than suffice
    /// for every use case.
    #[inline]
    pub const fn min() -> Self {
      Self::new(-2_000_000_000.0)
    }

    /// Creates a new [PdfPoints] object from the given measurement in inches.
    #[inline]
    pub fn from_inches(inches: f32) -> Self {
      Self::new(inches * 72.0)
    }

    /// Creates a new [PdfPoints] object from the given measurement in centimeters.
    #[inline]
    pub fn from_cm(cm: f32) -> Self {
      Self::from_inches(cm / 2.54)
    }

    /// Creates a new [PdfPoints] object from the given measurement in millimeters.
    #[inline]
    pub fn from_mm(mm: f32) -> Self {
      Self::from_cm(mm / 10.0)
    }

    /// Converts the value of this [PdfPoints] object to inches.
    #[inline]
    pub fn inches(&self) -> f32 {
      self.value / 72.0
    }

    /// Converts the value of this [PdfPoints] object to centimeters.
    #[inline]
    pub fn cm(&self) -> f32 {
      self.inches() * 2.54
    }

    /// Converts the value of this [PdfPoints] object to millimeters.
    #[inline]
    pub fn mm(&self) -> f32 {
      self.cm() * 10.0
    }
  }

  impl Add<PdfPoints> for PdfPoints {
    type Output = PdfPoints;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
      PdfPoints::new(self.value + rhs.value)
    }
  }

  impl AddAssign<PdfPoints> for PdfPoints {
    #[inline]
    fn add_assign(&mut self, rhs: PdfPoints) {
      self.value += rhs.value;
    }
  }

  impl Sub<PdfPoints> for PdfPoints {
    type Output = PdfPoints;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
      PdfPoints::new(self.value - rhs.value)
    }
  }

  impl SubAssign<PdfPoints> for PdfPoints {
    #[inline]
    fn sub_assign(&mut self, rhs: PdfPoints) {
      self.value -= rhs.value;
    }
  }

  impl Mul<f32> for PdfPoints {
    type Output = PdfPoints;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
      PdfPoints::new(self.value * rhs)
    }
  }

  impl Div<f32> for PdfPoints {
    type Output = PdfPoints;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
      PdfPoints::new(self.value / rhs)
    }
  }
}

mod text_width {
  /// From https://github.com/adambisek/string-pixel-width/blob/master/src/widthsMap.js
  ///
  /// MIT License
  ///
  /// Copyright (c) 2023 Adam Ernst Bisek
  ///
  /// Permission is hereby granted, free of charge, to any person obtaining a copy
  /// of this software and associated documentation files (the "Software"), to deal
  /// in the Software without restriction, including without limitation the rights
  /// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  /// copies of the Software, and to permit persons to whom the Software is
  /// furnished to do so, subject to the following conditions:
  ///
  /// The above copyright notice and this permission notice shall be included in all
  /// copies or substantial portions of the Software.
  ///
  /// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  /// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  /// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  /// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  /// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  /// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  /// SOFTWARE.
  const HELVETICA_CHAR_WIDTH: phf::Map<char, u32> = phf::phf_map! {
    '0' => 56,
    '1' => 56,
    '2' => 56,
    '3' => 56,
    '4' => 56,
    '5' => 56,
    '6' => 56,
    '7' => 56,
    '8' => 56,
    '9' => 56,
    ' ' => 28,
    '!' => 28,
    '"' => 35,
    '#' => 56,
    '$' => 56,
    '%' => 89,
    '&' => 67,
    '\'' => 19,
    '(' => 33,
    ')' => 33,
    '*' => 39,
    '+' => 58,
    ',' => 28,
    '-' => 33,
    '.' => 28,
    '/' => 28,
    ':' => 28,
    ';' => 28,
    '<' => 58,
    '=' => 58,
    '>' => 58,
    '?' => 56,
    '@' => 102,
    'A' => 67,
    'B' => 67,
    'C' => 72,
    'D' => 72,
    'E' => 67,
    'F' => 61,
    'G' => 78,
    'H' => 72,
    'I' => 28,
    'J' => 50,
    'K' => 67,
    'L' => 56,
    'M' => 83,
    'N' => 72,
    'O' => 78,
    'P' => 67,
    'Q' => 78,
    'R' => 72,
    'S' => 67,
    'T' => 61,
    'U' => 72,
    'V' => 67,
    'W' => 94,
    'X' => 67,
    'Y' => 67,
    'Z' => 61,
    '[' => 28,
    '\\' => 28,
    ']' => 28,
    '^' => 47,
    '_' => 56,
    '`' => 33,
    'a' => 56,
    'b' => 56,
    'c' => 50,
    'd' => 56,
    'e' => 56,
    'f' => 28,
    'g' => 56,
    'h' => 56,
    'i' => 22,
    'j' => 22,
    'k' => 50,
    'l' => 22,
    'm' => 83,
    'n' => 56,
    'o' => 56,
    'p' => 56,
    'q' => 56,
    'r' => 33,
    's' => 50,
    't' => 28,
    'u' => 56,
    'v' => 50,
    'w' => 72,
    'x' => 50,
    'y' => 50,
    'z' => 50,
    '{' => 33,
    '|' => 26,
    '}' => 33,
    '~' => 58,
  };

  pub fn helvetica_width(text: &str, font_size: f32) -> f32 {
    text
      .chars()
      .map(|c| {
        *HELVETICA_CHAR_WIDTH
          .get(&c)
          .unwrap_or(HELVETICA_CHAR_WIDTH.get(&'x').unwrap()) as f32
          * (font_size / 100.0)
      })
      .sum()
  }
}
