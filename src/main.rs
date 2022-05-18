#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]

use rltk::{GameState, Rltk, RGB, Point };
use specs::prelude::*;

mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
pub use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
pub use visibility_system::VisibilitySystem;
mod monster_ai_system;
pub use monster_ai_system::MonsterAI;
mod map_indexing_system;
pub use map_indexing_system::MapIndexingSystem;
// TODO: implement the following:
// mod melee_combat_system;
// use melee_combat_system::MeleeCombatSystem;
mod damage_system;
pub use damage_system::DamageSystem;






#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Waiting, Pre, Player, Monster }

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);
        // let mut melee = MeleeCombatSystem {};
        // melee.run_now(&self.ecs);
        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx : &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::Pre => {
                self.run_systems();
                newrunstate = RunState::Waiting;
            },
            RunState::Waiting => {
                newrunstate = player_input(self, ctx);
            },
            RunState::Player => {
                self.run_systems();
                newrunstate = RunState::Monster;
            },
            RunState::Monster => {
                self.run_systems();
                newrunstate = RunState::Waiting;
            },
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph); } 
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    
    let context = RltkBuilder::simple80x50()
        .with_title("Rogee")
        .build()?;
    
    let mut gs = State {
        ecs: World::new(),
    };
    
    // register components from components.rs
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let map: Map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    
    // spawn in player
    let player_entity = gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name {name: "Player".to_string()})
        .with(CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 })
        .build();
    
    // spawn in the monsters
    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();
        
        let glyph: rltk::FontCharType;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => { glyph = rltk::to_cp437('g'); name = "Goblin".to_string(); },
            _ => { glyph = rltk::to_cp437('o'); name = "Orc".to_string(); },
        }

        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
            .with(Monster {})
            .with(Name { name: format!("{} #{}", &name, i) })
            .with(CombatStats { max_hp: 16, hp: 16, defense: 1, power: 4 })
            .with(BlocksTile {})
            .build();
    }
    
    gs.ecs.insert(map);
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::Pre);

    rltk::main_loop(context, gs)
}