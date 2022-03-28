use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use js_sys::Math;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn start() {
    let mut world = World::new();
    world.set_random_grid();
    world.tick();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        world.tick();
        render(&world);
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn render(world: &World) {
    static CELL_SIZE: u32 = 10;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    canvas.set_width((CELL_SIZE + 1) * WORLD_WIDTH + 10);
    canvas.set_height((CELL_SIZE + 1) * WORLD_HEIGHT+ 10);

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.begin_path();

    for i in 0..WORLD_WIDTH + 1 {
        let x = (i * (CELL_SIZE + 1) + 1) as f64;
        let y = ((CELL_SIZE + 1) * WORLD_HEIGHT + 1) as f64;
        context.move_to(x, 0.0);
        context.line_to(x, y);
    }

    for i in 0..WORLD_HEIGHT + 1 {
        let y = (i * (CELL_SIZE + 1) + 1) as f64;
        let x = ((CELL_SIZE + 1) * WORLD_WIDTH + 1) as f64;
        context.move_to(0.0, y);
        context.line_to(x, y);
    }

    context.stroke();

    context.begin_path();

    for x in 0..WORLD_WIDTH {
        for y in 0..WORLD_HEIGHT {
            match world.get_cell(x, y) {
                Cell::Alive => context.set_fill_style(&JsValue::from_str("#000000")),
                Cell::Dead => context.set_fill_style(&JsValue::from_str("#FFFFFF")),
            }

            context.fill_rect(
                (x * (CELL_SIZE + 1) + 1) as f64,
                (y * (CELL_SIZE + 1) + 1) as f64,
                CELL_SIZE as f64,
                CELL_SIZE as f64
            );
        }
    }

    context.stroke();
}

#[derive(Clone, Copy)]
enum Cell {
    Alive,
    Dead,
}

const WORLD_WIDTH: u32 = 80;
const WORLD_HEIGHT: u32 = 100;

struct World {
    cells: [[Cell; WORLD_HEIGHT as usize]; WORLD_WIDTH as usize],
}

impl World {
    fn new() -> Self {
        World {
            cells: [[Cell::Dead; WORLD_HEIGHT as usize]; WORLD_WIDTH as usize],
        }
    }

    fn get_cell(&self, x: u32, y: u32) -> Cell {
        self.cells[x as usize][y as usize]
    }

    fn set_cell(&mut self, x: u32, y: u32, cell: Cell) {
        self.cells[x as usize][y as usize] = cell;
    }

    fn count_neighbours(&self, x: u32, y: u32) -> u8 {
        let mut total: u8 = 0;

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = (x as i32 + dx).rem_euclid(WORLD_WIDTH as i32) as u32;
                let ny = (y as i32 + dy).rem_euclid(WORLD_HEIGHT as i32) as u32;

                total += match self.get_cell(nx, ny) {
                    Cell::Alive => 1,
                    Cell::Dead => 0,
                }
            }
        }

        if total >= 8 {
            log(format!("total: {}", total).as_str());
        }

        total
    }

    fn tick(&mut self) {
        let mut new_cells = [[Cell::Dead; WORLD_HEIGHT as usize]; WORLD_WIDTH as usize];

        for x in 0..WORLD_WIDTH {
            for y in 0..WORLD_HEIGHT {
                let cell = self.get_cell(x, y);
                let nc = self.count_neighbours(x, y);

                let new_cell = match (cell, nc) {
                    (Cell::Alive, x) if x < 2 =>  Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (c, _) => c,
                };

                new_cells[x as usize][y as usize] = new_cell;
            }
        }

        self.cells = new_cells;
    }

    fn set_random_grid(&mut self) {
        for x in 0..WORLD_WIDTH {
            for y in 0..WORLD_HEIGHT {
                self.set_cell(x, y,
                    if Math::random() < 0.5 {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                )
            }
        }
    }
}
