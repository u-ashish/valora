//! Module for working with paths and path segments.

mod line_segment;

use self::line_segment::LineSegment;
use crate::{
    bounds::Bounds,
    path::{self, Path},
    V2,
};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
#[derive(Debug)]
pub enum Segment {
    LineSegment(LineSegment),
}

impl Segment {
    pub fn from_link(link: (path::Segment, path::Segment)) -> Vec<Segment> {
        match link {
            (path::Segment::MoveTo(start), path::Segment::LineTo(end))
            | (path::Segment::LineTo(start), path::Segment::LineTo(end)) => {
                LineSegment::new_rasterable(start, end)
                    .map(|ls| vec![Self::from(ls)])
                    .unwrap_or_default()
            }
            (_, path::Segment::MoveTo(_)) => vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersection {
    /// Where on the excluded axis (x or y) the intersection occurs.
    pub axis: f32,
    /// Where along the segment [0.0, 1.0] the intersection occurs.
    pub t: f32,
}

/// A trait for monotonic curves.
#[enum_dispatch(Segment)]
pub trait Curve {
    /// Returns the point at `t` on the curve, if it is defined. Curves are defined on
    /// the domain `[0, 1]` only.
    fn sample_t(&self, t: f32) -> Option<V2>;

    /// Returns the `y` and `t` coordinate where the curve intersects a vertical line at `x`.
    fn sample_x(&self, x: f32) -> Option<Intersection>;

    /// Returns the `x` and `t` coordinate where the curve intersects a horizontal line at `y`.
    fn sample_y(&self, y: f32) -> Option<Intersection>;

    /// Returns a bounding box for the curve.
    fn bounds(&self) -> &Bounds;
}