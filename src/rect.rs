use ggez::graphics::{Point2, Rect as GgezRect};

pub const NE: usize = 0; // north-east
pub const NW: usize = 1; // north-west, etc.
pub const SW: usize = 2;
pub const SE: usize = 3;

/// A simple rectangle
#[derive(Clone, Debug, PartialEq)]
pub struct Rect {
    pub center: Point2,
    pub w_half: f32,
    pub h_half: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            center: Point2::new((x + x + w) / 2.0, (y + y + h) / 2.0),
            w_half: w / 2.0,
            h_half: h / 2.0,
        }
    }

    pub fn corner(&self, which: usize) -> Option<Point2> {
        match which {
            NE => Some(Point2::new(
                self.center.x + self.w_half,
                self.center.y - self.h_half,
            )),
            NW => Some(Point2::new(
                self.center.x - self.w_half,
                self.center.y - self.h_half,
            )),
            SE => Some(Point2::new(
                self.center.x + self.w_half,
                self.center.y + self.h_half,
            )),
            SW => Some(Point2::new(
                self.center.x - self.w_half,
                self.center.y + self.h_half,
            )),
            _other => None,
        }
    }

    pub fn contains_rect(&self, other: &Self) -> bool {
        self.contains_point(&other.corner(NE).unwrap())
            && self.contains_point(&other.corner(NW).unwrap())
            && self.contains_point(&other.corner(SW).unwrap())
            && self.contains_point(&other.corner(SE).unwrap())
    }

    pub fn contains_point(&self, point: &Point2) -> bool {
        self.corner(NW).unwrap().x <= point.x
            && point.x <= self.corner(NE).unwrap().x
            && self.corner(NE).unwrap().y <= point.y
            && point.y <= self.corner(SE).unwrap().y
    }

    pub fn to_ggez(&self) -> GgezRect {
        GgezRect::new(
            self.center.x - self.w_half,
            self.center.y - self.h_half,
            2.0 * self.w_half,
            2.0 * self.h_half,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ultimate edge case for bound checking
    #[test]
    fn rect_contains_itself() {
        let r = Rect {
            center: Point2::new(50.0, 50.0),
            w_half: 50.0,
            h_half: 50.0,
        };
        assert!(r.contains_rect(&r));
    }

    #[test]
    fn rect_contains_corners() {
        let r = Rect {
            center: Point2::new(50.0, 50.0),
            w_half: 50.0,
            h_half: 50.0,
        };
        assert!(r.contains_point(&r.corner(NE).unwrap()));
        assert!(r.contains_point(&r.corner(NW).unwrap()));
        assert!(r.contains_point(&r.corner(SW).unwrap()));
        assert!(r.contains_point(&r.corner(SE).unwrap()));
    }

    #[test]
    fn rect_contains_center() {
        let r = Rect {
            center: Point2::new(50.0, 50.0),
            w_half: 50.0,
            h_half: 50.0,
        };

        assert!(r.contains_point(&r.center));
    }

    #[test]
    fn rect_overlap_is_not_enough() {
        let r = Rect {
            center: Point2::new(50.0, 50.0),
            w_half: 50.0,
            h_half: 50.0,
        };

        let r2 = Rect {
            center: Point2::new(r.center.x + 1.0, r.center.y + 1.0),
            w_half: r.w_half,
            h_half: r.h_half,
        };

        assert!(!r.contains_rect(&r2));
    }

    #[test]
    fn rect_no_overlap() {
        let r = Rect {
            center: Point2::new(50.0, 50.0),
            w_half: 50.0,
            h_half: 50.0,
        };

        let r2 = Rect {
            center: Point2::new(r.center.x + 1.0, r.center.y + 1.0),
            w_half: r.w_half,
            h_half: r.h_half,
        };

        assert!(!r.contains_rect(&r2));
    }
}
