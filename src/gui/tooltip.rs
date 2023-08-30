use super::{ camera::get_screen_bounds, Attributes, Hidden, Map, Name, Pools, Position, Renderable, Rltk, World, RGB };
use crate::TileType;
use crate::data::ids::*;
use rltk::prelude::*;
use specs::prelude::*;

struct Tooltip {
    lines: Vec<(String, RGB)>,
}

const ATTRIBUTE_COLOUR: RGB = RGB { r: 1.0, g: 0.75, b: 0.8 };
const RED_WARNING: RGB = RGB { r: 1.0, g: 0.0, b: 0.0 };
const ORANGE_WARNING: RGB = RGB { r: 1.0, g: 0.65, b: 0.0 };
const YELLOW_WARNING: RGB = RGB { r: 1.0, g: 1.0, b: 0.0 };
const GREEN_WARNING: RGB = RGB { r: 0.0, g: 1.0, b: 0.0 };

impl Tooltip {
    fn new() -> Tooltip {
        return Tooltip { lines: Vec::new() };
    }
    fn add<S: ToString>(&mut self, line: S, fg: RGB) {
        self.lines.push((line.to_string(), fg));
    }
    fn width(&self) -> i32 {
        let mut max = 0;
        for s in self.lines.iter() {
            if s.0.len() > max {
                max = s.0.len();
            }
        }
        return (max as i32) + 2i32;
    }
    fn height(&self) -> i32 {
        return (self.lines.len() as i32) + 2i32;
    }
    fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        ctx.draw_box(x, y, self.width() - 1, self.height() - 1, RGB::named(WHITE), RGB::named(BLACK));
        for (i, s) in self.lines.iter().enumerate() {
            ctx.print_color(x + 1, y + (i as i32) + 1, s.1, RGB::named(BLACK), &s.0);
        }
    }
}

#[rustfmt::skip]
pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk, xy: Option<(i32, i32)>) {
    let (min_x, _max_x, min_y, _max_y, x_offset, y_offset) = get_screen_bounds(ecs, ctx);
    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let entities = ecs.entities();
    let player_entity = ecs.fetch::<Entity>();

    let mouse_pos = if xy.is_none() { ctx.mouse_pos() } else { xy.unwrap() };
    let mut mouse_pos_adjusted = mouse_pos;
    mouse_pos_adjusted.0 += min_x - x_offset;
    mouse_pos_adjusted.1 += min_y - y_offset;
    if mouse_pos_adjusted.0 >= map.width
        || mouse_pos_adjusted.1 >= map.height
        || mouse_pos_adjusted.1 < 0 // Might need to be 1, and -1 from map height/width.
        || mouse_pos_adjusted.0 < 0
    {
        return;
    }
    if !(map.visible_tiles[map.xy_idx(mouse_pos_adjusted.0, mouse_pos_adjusted.1)]
        || map.telepath_tiles[map.xy_idx(mouse_pos_adjusted.0, mouse_pos_adjusted.1)])
    {
        return;
    }

    let mut tooltips: Vec<Tooltip> = Vec::new();
    
    match map.tiles[map.xy_idx(mouse_pos_adjusted.0, mouse_pos_adjusted.1)] {
        TileType::ToLocal(n) => {
            let name = get_local_desc(n);
            let mut tip = Tooltip::new();
            tip.add(format!("You see {}.", name), get_local_col(n));
            tooltips.push(tip);
        }
        _ => {}
    }

    for (entity, position, renderable, _name, _hidden) in (&entities, &positions, &renderables, &names, !&hidden).join() {
        if position.x == mouse_pos_adjusted.0 && position.y == mouse_pos_adjusted.1 {
            let mut tip = Tooltip::new();
            tip.add(crate::gui::obfuscate_name_ecs(ecs, entity).0, renderable.fg);
            // Attributes
            let attr = attributes.get(entity);
            if let Some(a) = attr {
                let mut s = "".to_string();
                if a.strength.bonus < -2 { s += "weak "};
                if a.strength.bonus > 2 { s += "strong "};
                if a.dexterity.bonus < -2 { s += "clumsy "};
                if a.dexterity.bonus > 2 { s += "agile "};
                if a.constitution.bonus < -2 { s += "frail "};
                if a.constitution.bonus > 2 { s += "hardy "};
                if a.intelligence.bonus < -2 { s += "dim "};
                if a.intelligence.bonus > 2 { s += "smart "};
                if a.wisdom.bonus < -2 { s += "unwise "};
                if a.wisdom.bonus > 2 { s += "wisened "};
                if a.charisma.bonus < -2 { s += "ugly"};
                if a.charisma.bonus > 2 { s += "attractive"};
                if !s.is_empty() {
                    if s.ends_with(" ") {
                        s.pop();
                    }
                    tip.add(s, ATTRIBUTE_COLOUR);
                }
            }
            // Pools
            let pool = pools.get(entity);
            let player_pool = pools.get(*player_entity).unwrap();
            if let Some(p) = pool {
                let level_diff: i32 = p.level - player_pool.level;
                if level_diff <= -2 {
                    tip.add("-weak-", YELLOW_WARNING);
                } else if level_diff >= 2 {
                    tip.add("*threatening*", ORANGE_WARNING);
                }
                let health_percent: f32 = p.hit_points.current as f32 / p.hit_points.max as f32;
                if health_percent == 1.0 {
                    tip.add("healthy", GREEN_WARNING);
                } else if health_percent <= 0.25 {
                    tip.add("*critical*", RED_WARNING);
                } else if health_percent <= 0.5 {
                    tip.add("-bloodied-", ORANGE_WARNING);
                } else if health_percent <= 0.75 {
                    tip.add("injured", YELLOW_WARNING);
                }
            }
            tooltips.push(tip);
        }
    }

    if tooltips.is_empty() { return ; }

    let white = RGB::named(rltk::WHITE);

    let arrow;
    let arrow_x;
    let arrow_y = mouse_pos.1;
    if mouse_pos.0 > 35 {
        // Render to the left
        arrow = to_cp437('→');
        arrow_x = mouse_pos.0 - 1;
    } else {
        // Render to the right
        arrow = to_cp437('←');
        arrow_x = mouse_pos.0 + 1;
    }
    ctx.set(arrow_x, arrow_y, white, RGB::named(rltk::BLACK), arrow);

    let mut total_height = 0;
    for t in tooltips.iter() {
        total_height += t.height();
    }

    let mut y = mouse_pos.1 - (total_height / 2);
    while y + (total_height / 2) > 50 {
        y -= 1;
    }

    for t in tooltips.iter() {
        let x = if mouse_pos.0 > 35 {
            mouse_pos.0 - (1 + t.width())
        } else {
            mouse_pos.0 + (1 + 1)
        };
        t.render(ctx, x, y);
        y += t.height();
    }
}
