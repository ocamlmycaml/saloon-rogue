use rltk::{ RGB, Point, Rltk, Console };
use crate::components::{CombatStats, Player, Position, Name};
use crate::game_log::GameLog;
use crate::game_map::GameMap;
use specs::prelude::*;


pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let log = ecs.fetch::<GameLog>();

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!("HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);
        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    let mut y = 44;
    for s in log.entries.iter() {
        if y < 49 {
            ctx.print(2, y, s);
        }
        y += 1;
    }

    draw_tooltips(ecs, ctx);
}

pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<GameMap>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let (mousex, mousey) = ctx.mouse_pos();
    if mousex < map.width && mousey < map.height {
        let mut tooltip: Vec<String> = Vec::new();

        for (name, position) in (&names, &positions).join() {
            if position.x == mousex && position.y == mousey {
                tooltip.push(name.name.to_string());
            }
        }

        if !tooltip.is_empty() {
            let mut width: i32 = 0;
            for s in tooltip.iter() {
                if width < s.len() as i32 {
                    width = s.len() as i32;
                }
            }
            width += 3;

            if mousex > 40 {
                let arrow_pos = Point::new(mousex - 2, mousey);
                let leftx = mousex - width;
                let mut y = mousey;
                for s in tooltip.iter() {
                    ctx.print_color(leftx, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                    let padding = (width - s.len() as i32) - 1;
                    for i in 0..padding {
                        ctx.print_color(arrow_pos.x - i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                    }
                    y += 1;
                }
                ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"->".to_string());
            } else {
                let arrow_pos = Point::new(mousex + 1, mousey);
                let leftx = mousex + 3;
                let mut y = mousey;
                for s in tooltip.iter() {
                    ctx.print_color(leftx + 1, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), s);
                    let padding = (width - s.len() as i32) - 1;
                    for i in 0..padding {
                        ctx.print_color(arrow_pos.x + 1 + i, y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &" ".to_string());
                    }
                    y += 1;
                }
                ctx.print_color(arrow_pos.x, arrow_pos.y, RGB::named(rltk::WHITE), RGB::named(rltk::GREY), &"<-".to_string());
            }

        }

    }
}