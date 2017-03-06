extern crate rand;
extern crate amethyst;
extern crate cgmath;

mod cmdline;
mod game;
mod raytrace;
mod springy;

use amethyst::{
    Application,
    Event,
    State,
    Trans,
    VirtualKeyCode,
    WindowEvent,
};
use amethyst::asset_manager::{
    AssetManager,
    AssetReadStorage,
    DirectoryStore,
};
use amethyst::config::Element;
use amethyst::ecs::components::{
    Mesh,
    Texture,
    Renderable,
    LocalTransform,
    Transform,
};
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{
    Layer,
    Pipeline,
    VertexPosNormal,
};
use amethyst::ecs::{
    Component,
    Join,
    RunArg,
    System,
    VecStorage,
    World,
};

use game::problem::Problem;
use game::render::{
    MouseAction,
    CardDisplay,
    Renderable as SRenderable
};
use game::grid::GridLocation;

use std::collections::HashMap;

struct CameraSystem<R:SRenderable>(std::marker::PhantomData<R>);

impl <R:SRenderable + Send> System<()> for CameraSystem<R> {
    fn run(&mut self, arg: RunArg, _:()) {
        let (low, high) = R::get_grid_extents();
        let (low, high) = (ren_to_world(&low), ren_to_world(&high));
        let mid = ((low[0]+high[0])/2.0, (low[1]+high[1])/2.0, (low[2]+high[2])/2.0);
        use amethyst::ecs::resources::{Camera, Projection};
        use amethyst::ecs::resources::ScreenDimensions;
        let (mut camera, dimensions) = arg.fetch(|w| {
            (w.write_resource::<Camera>(), w.read_resource::<ScreenDimensions>())
        });

        //println!("{},{}", dimensions.w, dimensions.h);


        let aspect = dimensions.aspect_ratio;
        let eye = [mid.0, mid.1, high[2]];
        let target = [mid.0, mid.1, mid.2];
        let up = [0.0, 1.0, 0.0];

        let projection = Projection::Orthographic {
            left: -10.0 * aspect,
            right: 10.0 * aspect,
            bottom: -10.0,
            top: 10.0,
            near: -10.0,
            far: 10.0,
        };

        camera.proj = projection;
        camera.eye = eye;
        camera.target = target;
        camera.up = up;
    }
}

struct CardThing {
    card: <game::solitaire::CardGamePercept as SRenderable>::CardId
}

impl Component for CardThing {
    type Storage = VecStorage<CardThing>;
}

struct Ident {
    id: game::cards::Ident
}

impl Component for Ident {
    type Storage = VecStorage<Ident>;
}

struct CardSystem {
    state: game::solitaire::Solitaire
}

impl System<()> for CardSystem {
    fn run(&mut self, arg: RunArg, _: ()) {
        let percept = self.state.percept();
        let card_list = percept.get_cards();
        let (cards, mut transform, mut render) = arg.fetch(|w| {
            (w.read::<CardThing>(), w.write::<LocalTransform>(), w.write::<Renderable>())
        });
        for (card, transform, render) in (&cards, &mut transform, &mut render).iter() {
            if let Some(data) = percept.get_data_for(card.card.clone()) {
                //transform.translation = [(data.pos[0] as f32)*2.5, (data.pos[1] as f32)*4.0, data.pos[2] as f32];
            }
        }
        ()
    }
}

fn get_card_asset_id(card: CardDisplay) -> String
{
    if let CardDisplay::Front(card) = card {
        let suit = match card.suit {
            0 => "spades",
            1 => "hearts",
            2 => "clubs",
            3 | _ => "diamonds"
        };
        let rank = match card.rank {
            0 => "ace".to_string(),
            10 => "jack".to_string(),
            11 => "queen".to_string(),
            12 => "king".to_string(),
            x => format!("{}", x+1)
        };
        format!("cards/card_{}_{}", rank, suit)
    } else if CardDisplay::Back == card {
        "white".to_string()
    } else {
        "gray".to_string()
    }
}

struct Test {
    state: game::solitaire::Solitaire,
    drag: Option<<game::solitaire::CardGamePercept as SRenderable>::CardId>,
    spacing: [f32; 3],
    mouse: (f32, f32),
    mouseray: raytrace::Ray
}

fn ren_to_world(pos: &GridLocation) -> [f32; 3] {
    [pos.x.to_float(1.25, 0.25), pos.y.to_float(-1.875, -0.25), pos.sort as f32*0.001]
}

impl Test {
    fn refresh(&mut self, asset_manager: &mut AssetManager, world: &mut World) {
        let mut map = HashMap::new();
        let percept = self.state.percept();
        for card in self.state.percept().get_cards() {
            if let Some(data) = percept.get_data_for(card.clone()) {
                map.insert(data.ident, card);
            }
        }
        for (ident, card) in (&world.read::<Ident>(), &mut world.write::<CardThing>()).iter() {
            if let Some(thing) = map.get(&ident.id) {
                card.card = thing.clone();
            }
        }

    }
    fn do_thing(&mut self, act: game::solitaire::CardGameAction, asset_manager: &mut AssetManager, world: &mut World) {
        println!("{:?}", act);
        let actions = self.state.actions();
        if actions.contains(&act) {
            self.state = self.state.result(act);
            self.refresh(asset_manager, world);
        }
    }
    fn mouse_moved(&mut self, x: f32, y: f32, world: &mut World) {
        use amethyst::ecs::resources::{Camera, Projection};
        use amethyst::ecs::resources::ScreenDimensions;
        let (camera, dimensions) = (world.read_resource::<Camera>(), world.read_resource::<ScreenDimensions>());
        self.mouseray = raytrace::Ray::from_camera_mouse(raytrace::res_cam_to_ren(&camera), (dimensions.w, dimensions.h), (x as i32,y as i32));
        let x = x/dimensions.w;
        let y = y/dimensions.h;

        match camera.proj {
            Projection::Orthographic{top: top, bottom: bottom, left: left, right:right, near:near, far:far} => {
                let x = left + x*(right-left) + 5.0;
                let y = bottom + y*(top-bottom);
                self.mouse = (x,-y);
            },
            _ => ()
        }
    }
    fn mouse_event(&mut self, evt: Event, asset_manager: &mut AssetManager, world: &mut World) {
        match evt {
            Event::MouseMoved(x, y) => {
                if let Some(ref card) = self.drag {
                    let percept = self.state.percept();
                    let data = percept.get_data_for(card.clone()).unwrap();
                    if let Some(ref children) = data.drag_children {
                        let base = ren_to_world(&data.pos);
                        let offset = [(x as f32)/10.0, (y as f32)/10.0, 0.0];
                        for (subject, mut transform) in (&world.read::<CardThing>(), &mut world.write::<LocalTransform>()).iter() {
                            if &subject.card == card || children.contains(&subject.card) {
                                let mut local = ren_to_world(&percept.get_data_for(subject.card.clone()).unwrap().pos);
                                for i in 0..3 {
                                    local[i] = local[i] - base[i] + offset[i];
                                }
                                transform.translation = local;
                            }
                        }
                    }
                }
            },
            Event::MouseInput(state, amethyst::MouseButton::Left) => {
                let mut ignore = Vec::new();
                if let Some(card) = self.drag.clone() {
                    ignore.push(card.clone());
                    if let Some(mut children) = self.state.percept().get_data_for(card).unwrap().drag_children {
                        for child in children.drain(..) {
                            ignore.push(child);
                        }
                    }
                }
                let mut target = None;
                let (width, height) = (2.25, 3.5);
                let cardshape = raytrace::Box::new(width, height, 0.001);
                println!("{:?}", self.mouseray);
                for (subject, transform, local) in (&world.read::<CardThing>(), &world.read::<Transform>(), &world.read::<LocalTransform>()).iter() {
                    if ignore.contains(&subject.card) {
                        continue;
                    }
                    let ray = self.mouseray.reverse_transform(transform.0);
                    println!("{:?}", ray);
                    use raytrace::Raytraceable;
                    if cardshape.raytrace(&ray).is_some() {
                        let thing = (local.translation[2], subject.card.clone());
                        if let Some((x, _)) = target {
                            if x < thing.0 {
                                target = Some(thing);
                            }
                        } else {
                            target = Some(thing);
                        }
                    }
                }
                println!("{:?}", target);
                if let Some((_,target)) = target {
                    let percept = self.state.percept();
                    if state == amethyst::ElementState::Pressed {
                        if percept.get_data_for(target.clone()).unwrap().drag_children.is_some() {
                            self.drag = Some(target);
                        }
                    } else {
                        if let Some(action) = percept.get_action_for(if let Some(ref drag) = self.drag {
                            game::render::MouseAction::Drop(drag.clone(), target)
                        } else {
                            game::render::MouseAction::Tap(target)
                        }) {
                            self.do_thing(action, asset_manager, world);
                        }
                    }
                }
                if state == amethyst::ElementState::Released {
                    self.drag = None
                }
            },
            _ => ()
        }
    }
}

impl State for Test {
    fn on_start(&mut self, world: &mut World, asset_manager: &mut AssetManager, pipe: &mut Pipeline) {
        use amethyst::renderer::pass::{Clear, DrawFlat};
        let layer = Layer::new("main",
                               vec![Clear::new([0.0,0.0,0.0,1.0]),
                                    DrawFlat::new("main", "main")]);
        pipe.layers.push(layer);
        {
            use amethyst::ecs::resources::{Camera, Projection};
            use amethyst::ecs::resources::ScreenDimensions;

            let dimensions = world.read_resource::<ScreenDimensions>();
            let mut camera = world.write_resource::<Camera>();
            let aspect = dimensions.aspect_ratio;
            let eye = [0.0, 0.0, 1.0];
            let target = [0.0, 0.0, 0.0];
            let up = [0.0, 1.0, 0.0];

            let projection = Projection::Orthographic {
                left: -5.0 * aspect,
                right: 5.0 * aspect,
                bottom: -5.0,
                top: 5.0,
                near: -40.0,
                far: 40.0,
            };

            camera.proj = projection;
            camera.eye = eye;
            camera.target = target;
            camera.up = up;
        }

        let assets_path = format!("{}/resources", env!("CARGO_MANIFEST_DIR"));
        asset_manager.register_store(DirectoryStore::new(assets_path));
        asset_manager.load_asset::<Mesh>("card", "obj");
        asset_manager.load_asset::<Mesh>("thick_card", "obj");
        asset_manager.load_asset::<Mesh>("cube", "obj");
        asset_manager.load_asset::<Texture>("amethyst_thumb", "png");

        let deck = cmdline::deck();
        for crd in deck {
            println!("{:?}", crd);
            asset_manager.load_asset::<Texture>(&get_card_asset_id(CardDisplay::Front(crd)), "png").unwrap();
        }
        //asset_manager.load_asset::<Texture>("cards/card_10_roads", "png");
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("gray", [0.5, 0.5, 0.5, 1.0]);
        asset_manager.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>("tri",isoc(1.0,1.0));
        let tri = asset_manager.create_renderable("card", "cards/card_10_clubs", "white", "white", 1.0).unwrap();
        //asset_manager.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        
        let percept = self.state.percept();
        for card in percept.get_cards() {
            let data = percept.get_data_for(card.clone()).unwrap();
                    world.create_now()
                        .with(tri.clone())
                        .with(CardThing{ card: card.clone()})
                        .with(Ident{id:data.ident})
                        .with(LocalTransform::default())
                        .with(springy::MoveTarget{pos:[0.0;3]})
                        .with(Transform::default())
                        .build();
        }
    }
    fn update(&mut self, world: &mut World, asset_manager: &mut AssetManager, _: &mut Pipeline) -> Trans {
        if self.state.is_goal() {
            return Trans::Quit;
        }
        let percept = self.state.percept();
        let card_list = percept.get_cards();
        let (cards, mut target, mut render) = (world.read::<CardThing>(), world.write::<springy::MoveTarget>(), world.write::<Renderable>());
        let textures = asset_manager.read_assets::<Texture>();
        let mut drag_offset = [0.0; 3];
        let mut dragging = Vec::new();
        if let Some(card) = self.drag.clone() {
            dragging.push(card.clone());
            let mut data = percept.get_data_for(card).unwrap();
            let pos = ren_to_world(&data.pos);
            let mut mouse = self.mouseray.along(0.1);
            mouse[2] = 1.0;
            for i in 0..3 {
                drag_offset[i] = mouse[i] - pos[i];
            }
            if let Some(mut children) = data.drag_children {
                for child in children.drain(..) {
                    dragging.push(child);
                }
            }
        }
        for (card, target, render) in (&cards, &mut target, &mut render).iter() {
            if let Some(data) = percept.get_data_for(card.card.clone()) {
                if dragging.contains(&card.card) {
                    let mut pos = ren_to_world(&data.pos);
                    for i in 0..3 {
                        pos[i] = pos[i] + drag_offset[i];
                    }
                    target.pos = pos;
                } else {
                    target.pos = ren_to_world(&data.pos);
                }
                if let Some(id) = asset_manager.id_from_name(&get_card_asset_id(data.display)) {
                    if let Some(tex) = textures.read(id) {
                        render.ambient = tex.clone();
                    }
                }
            }
        }
        Trans::None
    }
    fn handle_events(&mut self, events: &[WindowEvent], world: &mut World, asset_manager: &mut AssetManager, _: &mut Pipeline) -> Trans {
        for event in events {
            match event.payload {
                Event::Closed | Event::KeyboardInput(_, _, Some(VirtualKeyCode::Escape)) => {
                    return Trans::Quit;
                },
                Event::MouseMoved(x, y) => self.mouse_moved(x as f32, y as f32, world),
                Event::MouseInput(_, _) => self.mouse_event(event.payload.clone(), asset_manager, world),
                _ => ()
            }
        }
        Trans::None
    }
}

fn main(){
    let resource_path = format!("{}/resources", env!("CARGO_MANIFEST_DIR"));
    let config_path = format!("{}/config.yml", resource_path);
    let display_config = DisplayConfig::from_file(config_path).unwrap();
    let initial = Test {
        state: cmdline::deal_with_it(),
        drag: None,
        spacing: [2.5, 4.5, 1.0],
        mouse: (0.0, 0.0),
        mouseray: raytrace::Ray {
            start: [0.0,0.0,0.0],
            velocity: [0.0,0.0,1.0]
        }
    };
    let mut game = Application::build(initial, display_config)
        .register::<CardThing>()
        .register::<springy::MoveTarget>()
        .register::<Ident>()
        .with::<CameraSystem<game::solitaire::CardGamePercept>>(CameraSystem(std::marker::PhantomData), "aspect", 10)
        .with::<springy::MoveSystem>(springy::MoveSystem{vel:50.0}, "movement", 10)
        //.with(CardSystem { state: cmdline::deal_with_it() }, "cards", 1)
        .done();
    game.run();
    println!("FINISHED");
}

fn isoc(w:f32,h:f32) -> Vec<VertexPosNormal> {
    vec![
        VertexPosNormal{
            pos:[-w/2.0, -h/2.0, 0.0],
            normal:[0.0, 0.0, 1.0],
            tex_coord:[0.0,0.0]
        },
        VertexPosNormal{
            pos:[w/2.0, -h/2.0, 0.0],
            normal:[0.0, 0.0, 1.0],
            tex_coord:[1.0,0.0]
        },
        VertexPosNormal{
            pos:[0.0, h/2.0, 0.0],
            normal:[0.0, 0.0, 1.0],
            tex_coord:[0.5,1.0]
        },
    ]
}


