use failure::{Error, Fail};
use ggez::{
    graphics::{self, DrawMode, Point2},
    Context,
};
use snowflake::ProcessUniqueId as Uid;

use std::collections::{HashMap, HashSet};

use crate::rect::*;

/// A quad-tree node implementation
#[derive(Clone, Debug, PartialEq)]
pub struct QTreeNode {
    pub boundary: Rect,
    objects: HashMap<Uid, Rect>,
    children: Option<Box<[Self; 4]>>,
    pub capacity: usize,
}

/// An error type
#[derive(Clone, Debug, Fail)]
pub enum QTreeError {
    #[fail(display = "The supplied rectangle doesn't fit the boundary")]
    RectDoesNotFit,
}

impl QTreeNode {
    /// Creates a new quadtree node. `capacity` must be above 0.
    pub fn new(boundary: Rect, capacity: usize) -> Self {
        Self {
            boundary,
            objects: HashMap::new(),
            children: None,
            capacity,
        }
    }

    /// Subdivide this node by adding 4 sub-nodes as children.
    fn subdiv(&mut self) {
        if self.children.is_some() {
            return;
        }
        let b = &self.boundary;

        let ne = b.corner(NE).unwrap();
        let nw = b.corner(NW).unwrap();
        let sw = b.corner(SW).unwrap();
        let se = b.corner(SE).unwrap();

        let rect_ne = Rect {
            center: Point2::new((b.center.x + ne.x) / 2.0, (b.center.y + ne.y) / 2.0),
            w_half: b.w_half / 2.0,
            h_half: b.h_half / 2.0,
        };
        let rect_nw = Rect {
            center: Point2::new((b.center.x + nw.x) / 2.0, (b.center.y + nw.y) / 2.0),
            w_half: b.w_half / 2.0,
            h_half: b.h_half / 2.0,
        };
        let rect_sw = Rect {
            center: Point2::new((b.center.x + sw.x) / 2.0, (b.center.y + sw.y) / 2.0),
            w_half: b.w_half / 2.0,
            h_half: b.h_half / 2.0,
        };
        let rect_se = Rect {
            center: Point2::new((b.center.x + se.x) / 2.0, (b.center.y + se.y) / 2.0),
            w_half: b.w_half / 2.0,
            h_half: b.h_half / 2.0,
        };

        self.children = Some(Box::new([
            QTreeNode::new(rect_ne, self.capacity),
            QTreeNode::new(rect_nw, self.capacity),
            QTreeNode::new(rect_sw, self.capacity),
            QTreeNode::new(rect_se, self.capacity),
        ]))
    }

    /// Insert a bounding box Rect into the tree
    pub fn insert(&mut self, rect: &Rect, id: Uid) -> Result<(), Error> {
        if !self.boundary.contains_rect(&rect) {
            return Err(QTreeError::RectDoesNotFit.into());
        }

        if self.objects.len() < self.capacity {
            self.objects.insert(id, rect.clone());
            return Ok(());
        }

        if self.children.is_none() {
            self.subdiv();
        }

        for child in self.children.as_mut().unwrap().iter_mut() {
            match child.insert(rect, id) {
                Ok(()) => return Ok(()),
                Err(e) => match e.downcast::<QTreeError>() {
                    Ok(QTreeError::RectDoesNotFit) => {}
                    Err(e) => return Err(e),
                },
            }
        }

        // Successful sub-insert returns, insert in this node if the object doesn't fit any of the
        // children
        self.objects.insert(id, rect.clone());
        Ok(())
    }

    /// Find `limit` objects containing a point. `limit == None` means no limit
    pub fn query_point<'a>(&'a self, point: &Point2, mut limit: Option<usize>) -> HashSet<Uid> {
        let mut ret = HashSet::new();

        if !self.boundary.contains_point(point) {
            return ret;
        }

        for (id, obj) in &self.objects {
            if obj.contains_point(point) {
                ret.insert(*id);
                if let Some(limit) = limit.as_mut() {
                    *limit -= 1;
                    if *limit == 0 {
                        break;
                    }
                }
            }
        }

        if let Some(children) = self.children.as_ref() {
            for child in children.iter() {
                ret = ret
                    .union(&child.query_point(point, limit))
                    .cloned()
                    .collect();
            }
        }

        ret
    }

    /// Draw all subregions contained in the tree
    pub fn draw_regions(&self, ctx: &mut Context, mode: DrawMode) -> Result<(), Error> {
        // Draw the current boundary
        graphics::rectangle(ctx, mode, self.boundary.to_ggez())?;

        if let Some(children) = self.children.as_ref() {
            for chld in children.iter() {
                chld.draw_regions(ctx, mode)?;
            }
        }
        Ok(())
    }

    /// Draw all objects contained in the tree
    pub fn draw_objects(&self, ctx: &mut Context, mode: DrawMode) -> Result<(), Error> {
        // Draw current node's objects
        for (_id, obj) in &self.objects {
            graphics::rectangle(ctx, mode, obj.to_ggez())?;
        }

        if let Some(children) = self.children.as_ref() {
            for chld in children.iter() {
                chld.draw_objects(ctx, mode)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check that subdivision arranges subnodes correctly
    #[test]
    fn subdiv_produces_children() {
        let rect = Rect {
            center: Point2::new(rand::random(), rand::random()),
            w_half: rand::random(),
            h_half: rand::random(),
        };

        let ne = rect.corner(NE).unwrap();
        let nw = rect.corner(NW).unwrap();
        let sw = rect.corner(SW).unwrap();
        let se = rect.corner(SE).unwrap();

        let expected_rects = vec![
            Rect {
                // NE
                center: Point2::new((rect.center.x + ne.x) / 2.0, (rect.center.y + ne.y) / 2.0),
                w_half: rect.w_half / 2.0,
                h_half: rect.h_half / 2.0,
            },
            Rect {
                // NW
                center: Point2::new((rect.center.x + nw.x) / 2.0, (rect.center.y + nw.y) / 2.0),
                w_half: rect.w_half / 2.0,
                h_half: rect.h_half / 2.0,
            },
            Rect {
                // SW
                center: Point2::new((rect.center.x + sw.x) / 2.0, (rect.center.y + sw.y) / 2.0),
                w_half: rect.w_half / 2.0,
                h_half: rect.h_half / 2.0,
            },
            Rect {
                // SE
                center: Point2::new((rect.center.x + se.x) / 2.0, (rect.center.y + se.y) / 2.0),
                w_half: rect.w_half / 2.0,
                h_half: rect.h_half / 2.0,
            },
        ];

        let mut qt = QTreeNode::new(rect.clone(), 4);
        dbg!(qt.clone());
        qt.subdiv();

        assert_ne!(qt.children, None);

        let children = qt.children.unwrap();

        let found_rects: Vec<_> = children.iter().cloned().map(|node| node.boundary).collect();

        assert_eq!(found_rects, expected_rects);
    }

    #[test]
    fn insert_capacity_works() {
        let boundary = Rect::new(0.0, 0.0, 200.0, 200.0);
        let capacity = 4;

        let mut qt = QTreeNode::new(boundary.clone(), capacity);

        let mut item = Rect::new(50.0, 50.0, 50.0, 50.0);

        // None of the objects fits the subregions, so they all end up in self.objects despite
        // capacity
        for i in 0..capacity + 1 {
            qt.insert(&item).unwrap();

            assert_eq!(qt.objects[i], item);
            item.center.x += 5.0;
        }

        // But as soon as something fitting one of the quarters appears, into a subregion it goes!
        let fitting_item = Rect::new(10.0, 10.0, 10.0, 10.0);
        qt.insert(&fitting_item).unwrap();
        assert!(qt.children.is_some());

        let children = qt.children.as_ref().unwrap();
        dbg!(children);
        assert_eq!(children[NW].objects[0], fitting_item);
    }

    #[test]
    fn insert_discards_not_fitting() {
        let boundary = Rect::new(10.0, 10.0, 10.0, 10.0);

        let item = Rect::new(0.0, 0.0, 20.0, 20.0);

        let mut qt = QTreeNode::new(boundary, 4);

        assert!(qt.insert(&item).is_err());
    }

    #[test]
    fn query_point_finds_all_rects() {
        let boundary = Rect::new(0.0, 0.0, 10.0, 10.0);
        let capacity = 4;
        let mut qt = QTreeNode::new(boundary.clone(), capacity);

        for _i in 0..capacity + 1 {
            qt.insert(&boundary).unwrap();
        }

        let found_rects = qt.query_point(&Point2::new(5.0, 5.0), None);

        assert_eq!(found_rects.len(), capacity + 1);
    }
}
