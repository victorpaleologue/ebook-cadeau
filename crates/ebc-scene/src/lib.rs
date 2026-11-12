//! Geometry, colour and scene-graph types shared by the eBook Cadeau print pipeline.
//!
//! This crate is the **contract** between the crates that *compose* a printable card
//! (`ebc-layout`, `ebc-spine`, `ebc-card`, …) and the crate that *writes* it (`ebc-pdf`).
//! It has no dependencies and performs no I/O, so it compiles everywhere, including
//! `wasm32-unknown-unknown`.
//!
//! # Conventions
//!
//! * **Lengths are millimetres** (`f64`) unless a name says otherwise (`_pt`, `_px`).
//! * **Page coordinates** have their origin at the **top-left corner of the trim box**
//!   (the finished, cut sheet), with **x to the right and y downwards**. Artwork that
//!   bleeds therefore uses negative coordinates or coordinates beyond the trim size.
//!   The PDF writer is responsible for flipping the y axis.
//! * **Colour components** are `f32` in `0.0..=1.0`.
//! * Angles are degrees, **clockwise** on the page (because y points down).

#![forbid(unsafe_code)]

pub mod color;
pub mod document;
pub mod geom;
pub mod path;
pub mod raster;

pub use color::Color;
pub use document::{
    DocInfo, Document, FontId, FontResource, Glyph, ImageData, ImageId, ImageResource, Item,
    OutputIntent, Page, PdfStandard, PixelFormat, Stroke, StrokeCap, StrokeJoin,
};
pub use geom::{
    MM_PER_INCH, MM_PER_PT, PT_PER_MM, Point, Rect, Size, Transform, mm_to_pt, pt_to_mm,
};
pub use path::{FillRule, Path, PathSeg};
pub use raster::Pixmap;
