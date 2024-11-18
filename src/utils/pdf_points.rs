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
