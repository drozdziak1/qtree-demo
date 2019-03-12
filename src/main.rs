#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

mod qtree;
mod rect;

use failure::Error;
use ggez::{
    event::{Keycode, Mod, MouseButton, MouseState},
    graphics::{Color, DrawMode, Point2},
    *,
};
use log::LevelFilter;
use snowflake::ProcessUniqueId as Uid;

use std::{
    collections::{HashMap, HashSet},
    env,
};

use qtree::{QTreeError, QTreeNode};
use rect::Rect;

static MIN_RADIUS: f32 = 10.0;
static SCALE_DELTA: f32 = 10.0;
static N_RANDOM_CIRCLES: usize = 1_000;

#[derive(Clone, Debug)]
struct Circle {
    pub id: Uid,
    pub coords: Point2,
    pub r: f32,
}

impl Circle {
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a new instance of an existing circle; the new instance gets a new id value
    pub fn duplicate(&self) -> Self {
        Self {
            id: Uid::new(),
            ..*self
        }
    }

    /// Checks whether a `point` is inclusively within the radius
    pub fn contains_point(&self, point: &Point2) -> bool {
        debug!("{:?}, Checking coords {}", self, point);
        (point.x - self.coords.x).powi(2) + (point.y - self.coords.y).powi(2) <= self.r.powi(2)
    }

    /// Returns the circle's bounding box
    pub fn bounding_box(&self) -> Rect {
        Rect::new(
            self.coords.x - self.r,
            self.coords.y - self.r,
            self.r * 2.0,
            self.r * 2.0,
        )
    }
}

impl Default for Circle {
    fn default() -> Self {
        Circle {
            id: Uid::new(),
            coords: Point2::new(0.0, 0.0),
            r: MIN_RADIUS,
        }
    }
}

struct MainState {
    mouse_coords: Point2,
    circles: HashMap<Uid, Circle>,
    qtree: QTreeNode,
    colliding_ids: HashSet<Uid>,
    draw_circles: bool,
    draw_boxes: bool,
    draw_regions: bool,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mode = ctx.conf.window_mode;
        let s = MainState {
            mouse_coords: Point2::new(0.0, 0.0),
            circles: HashMap::new(),
            qtree: QTreeNode::new(
                Rect::new(0.0, 0.0, mode.width as f32, mode.height as f32),
                4,
            ),
            colliding_ids: HashSet::new(),
            draw_circles: true,
            draw_boxes: false,
            draw_regions: false,
        };
        Ok(s)
    }

    fn add_circle(&mut self, circ: Circle) -> Result<(), Error> {
        if self.qtree.boundary.contains_rect(&circ.bounding_box()) {
            self.qtree.insert(&circ.bounding_box(), circ.id)?;
            self.circles.insert(circ.id, circ);
            return Ok(());
        }
        return Err(QTreeError::RectDoesNotFit.into());
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        if self.draw_circles {
            for (id, circ) in &mut self.circles {
                if self.colliding_ids.contains(id) {
                    graphics::set_color(ctx, Color::new(0.0, 0.0, 1.0, 0.5))?;
                } else {
                    graphics::set_color(ctx, Color::new(1.0, 1.0, 1.0, 0.5))?;
                }
                graphics::circle(ctx, DrawMode::Line(2.0), circ.coords, circ.r, 0.5)?;
            }
        }

        if self.draw_boxes {
            graphics::set_color(ctx, Color::new(1.0, 0.0, 0.0, 0.5))?;
            self.qtree
                .draw_objects(ctx, DrawMode::Line(2.0))
                .unwrap_or_else(|e| error!("Could not draw the qtree elements: {:?}", e));
        }

        if self.draw_regions {
            graphics::set_color(ctx, Color::new(0.0, 1.0, 0.0, 0.5))?;
            self.qtree
                .draw_regions(ctx, DrawMode::Line(2.0))
                .unwrap_or_else(|e| error!("Could not draw the qtree: {:?}", e));
        }

        graphics::present(ctx);
        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        info!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
        match button {
            MouseButton::Left => {
                info!("Creating new circle");
                let circ = Circle {
                    coords: Point2::new(x as f32, y as f32),
                    ..Default::default()
                };

                self.add_circle(circ).unwrap_or_else(|e| {
                    error!("Could not add circle: {:?}", e);
                });
            }
            MouseButton::Right => {
                info!("Purging all circles");
                self.circles = HashMap::new();

                let mode = ctx.conf.window_mode;
                self.qtree = QTreeNode::new(
                    Rect::new(0.0, 0.0, mode.width as f32, mode.height as f32),
                    4,
                );
            }
            MouseButton::Middle => {
                info!("Creating {} new circles", N_RANDOM_CIRCLES);
                for _i in 0..N_RANDOM_CIRCLES {
                    self.add_circle(Circle {
                        coords: Point2::new(
                            (rand::random::<u16>() % 800) as f32,
                            (rand::random::<u16>() % 600) as f32,
                        ),
                        ..Default::default()
                    })
                    .unwrap_or_else(|e| {
                        error!("Could not add circle: {:?}", e);
                    });
                }
            }
            other => {
                info!("Unhandled mouse button: {:?}", other);
            }
        }
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        _state: MouseState,
        x: i32,
        y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        trace!("Mouse moved: {}, {}", x, y);
        self.mouse_coords.x = x as f32;
        self.mouse_coords.y = y as f32;

        self.colliding_ids = self
            .qtree
            .query_point(&self.mouse_coords, None)
            .iter()
            .cloned()
            .filter(|id| self.circles[id].contains_point(&self.mouse_coords))
            .collect();
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: i32, y: i32) {
        info!("Got mousewheel (x: {}, y: {})", x, y);
        let mut rebuild_tree = false;
        let colliding = self.qtree.query_point(&self.mouse_coords, None);

        info!(
            "Colliding with {} bounding boxes:\n{:#?}",
            colliding.len(),
            colliding
        );

        let canvas = &self.qtree.boundary;

        let colliding = self.qtree.query_point(&self.mouse_coords, None);

        let closest_circ_opt: Option<&Circle> = colliding
            .iter()
            .map(|id| self.circles.get(id).unwrap())
            .filter(|circ| circ.contains_point(&self.mouse_coords))
            .fold(None, |cur_min: Option<&Circle>, x| {
                if let Some(cur_min_circ) = cur_min.as_ref() {
                    if cur_min_circ.r < x.r {
                        return cur_min;
                    }
                }
                Some(x)
            });

        if closest_circ_opt.is_none() {
            return;
        }

        let closest_circ = closest_circ_opt.unwrap();

        let delta = SCALE_DELTA * ((x + y) as f32);
        let mut new_circ = closest_circ.clone();
        new_circ.r += delta;
        if new_circ.r < MIN_RADIUS {
            new_circ.r = MIN_RADIUS;
        }
        if canvas.contains_rect(&new_circ.bounding_box()) {
            self.circles.insert(new_circ.id, new_circ);
            rebuild_tree = true;
        }

        if rebuild_tree {
            let mut new_qt = QTreeNode::new(self.qtree.boundary.clone(), self.qtree.capacity);
            for (id, circ) in self.circles.iter() {
                new_qt
                    .insert(&circ.bounding_box(), *id)
                    .unwrap_or_else(|e| error!("Could not insert circle {}: {:?}", id, e));
            }

            self.qtree = new_qt;
        }
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: Keycode,
        _keymod: Mod,
        _repeat: bool,
    ) {
        match keycode {
            Keycode::Num1 => {
                self.draw_circles = !self.draw_circles;
                if self.draw_circles {
                    info!("Circles ON");
                } else {
                    info!("Circles OFF");
                }
            }
            Keycode::Num2 => {
                self.draw_boxes = !self.draw_boxes;
                if self.draw_boxes {
                    info!("Boxes ON");
                } else {
                    info!("Boxes OFF");
                }
            }
            Keycode::Num3 => {
                self.draw_regions = !self.draw_regions;
                if self.draw_regions {
                    info!("Regions ON");
                } else {
                    info!("Regions OFF");
                }
            }
            _other => {}
        }
    }
}

pub fn main() {
    match env::var("RUST_LOG") {
        Ok(_) => env_logger::init(),
        Err(_e) => {
            env_logger::Builder::new()
                .filter_level(LevelFilter::Info)
                .init();
        }
    }
    let mut c = conf::Conf::new();
    c.window_setup.title = env!("CARGO_PKG_NAME").to_owned();
    let ctx = &mut Context::load_from_conf(env!("CARGO_PKG_NAME"), "drozdziak1", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();

    event::run(ctx, state).unwrap();
}
