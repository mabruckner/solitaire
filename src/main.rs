extern crate rand;
extern crate amethyst;

mod cmdline;
mod game;

use amethyst::asset_manager::{
    AssetManager,
    AssetReadStorage,
    DirectoryStore,
};
use amethyst::config::Element;
use amethyst::ecs::components::rendering::{
    Mesh,
    Texture,
    Renderable,
};
use amethyst::ecs::components::transform::{
    LocalTransform,
    Transform,
};
use amethyst::engine::{Application, State, Trans};
use amethyst::event::{
    Event,
    VirtualKeyCode,
    WindowEvent,
};
use amethyst::gfx_device::DisplayConfig;
use amethyst::renderer::{
    Layer,
    Pipeline,
    VertexPosNormal,
};
use amethyst::specs::{
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
    Renderable as SRenderable
};

struct CameraSystem;

impl System<()> for CameraSystem {
    fn run(&mut self, arg: RunArg, _:()) {
        use amethyst::ecs::resources::camera::{Camera, Projection};
        use amethyst::ecs::resources::ScreenDimensions;
        let (mut camera, dimensions) = arg.fetch(|w| {
            (w.write_resource::<Camera>(), w.read_resource::<ScreenDimensions>())
        });

        //println!("{},{}", dimensions.w, dimensions.h);


        let aspect = dimensions.aspect_ratio;
        let eye = [5.0, 0.0, 1.0];
        let target = [5.0, 0.0, 0.0];
        let up = [0.0, 1.0, 0.0];

        let projection = Projection::Orthographic {
            left: -10.0,// * aspect,
            right: 10.0,// * aspect,
            bottom: -10.0,
            top: 10.0,
            near: -10.0,
            far: 10.0,
        };

        camera.projection = projection;
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
                transform.translation = [(data.pos[0] as f32)*2.5, (data.pos[1] as f32)*4.0, data.pos[2] as f32];
            }
        }
        ()
    }
}

fn get_card_asset_id(card: Option<game::cards::Card>) -> String
{
    if let Some(card) = card {
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
    } else {
        "white".to_string()
    }
}

struct Test {
    state: game::solitaire::Solitaire,
    drag: Option<<game::solitaire::CardGamePercept as SRenderable>::CardId>,
    spacing: [f32; 3],
    mouse: (f32, f32)
}

impl Test {
    fn refresh(&mut self, asset_manager: &mut AssetManager, world: &mut World) {
        {
            let cards = world.read::<CardThing>();
            for ent in world.entities().iter() {
                if cards.get(ent).is_some() {
                    world.delete_later(ent);
                }
            }
        }
        let cardbase = asset_manager.create_renderable("card", "cards/card_10_roads", "white", "white", 1.0).unwrap();

        for card in self.state.percept().get_cards() {
            world.create_now()
                .with(cardbase.clone())
                .with(CardThing{ card: card.clone()})
                .with(LocalTransform::default())
                .with(Transform::default())
                .build();
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
    fn ren_to_world(&self, pos: &[f64; 3]) -> [f32; 3] {
        [(pos[0] as f32)*self.spacing[0], (pos[1] as f32)*self.spacing[1], (pos[2] as f32)*self.spacing[2]]
    }
    fn mouse_moved(&mut self, x: f32, y: f32, world: &mut World) {
        use amethyst::ecs::resources::camera::{Camera, Projection};
        use amethyst::ecs::resources::ScreenDimensions;
        let (camera, dimensions) = (world.read_resource::<Camera>(), world.read_resource::<ScreenDimensions>());
        let x = x/dimensions.w;
        let y = y/dimensions.h;

        match camera.projection {
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
                        let base = self.ren_to_world(&data.pos);
                        let offset = [(x as f32)/10.0, (y as f32)/10.0, 0.0];
                        for (subject, mut transform) in (&world.read::<CardThing>(), &mut world.write::<LocalTransform>()).iter() {
                            if &subject.card == card || children.contains(&subject.card) {
                                let mut local = self.ren_to_world(&percept.get_data_for(subject.card.clone()).unwrap().pos);
                                for i in 0..3 {
                                    local[i] = local[i] - base[i] + offset[i];
                                }
                                transform.translation = local;
                            }
                        }
                    }
                }
            },
            Event::MouseInput(state, amethyst::event::MouseButton::Left) => {
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
                for (subject, mut transform) in (&world.read::<CardThing>(), &mut world.write::<LocalTransform>()).iter() {
                    if ignore.contains(&subject.card) {
                        continue;
                    }
                    if self.mouse.0 <= transform.translation[0] + width/2.0 && self.mouse.0 >= transform.translation[0] - width/2.0 &&
                       self.mouse.1 <= transform.translation[1] + height/2.0 && self.mouse.1 >= transform.translation[1] - height/2.0 {
                        let thing = (transform.translation[2], subject.card.clone());
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
                    if state == amethyst::event::ElementState::Pressed {
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
                if state == amethyst::event::ElementState::Released {
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
            use amethyst::ecs::resources::camera::{Camera, Projection};
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

            camera.projection = projection;
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
            asset_manager.load_asset::<Texture>(&get_card_asset_id(Some(crd)), "png").unwrap();
        }
        //asset_manager.load_asset::<Texture>("cards/card_10_roads", "png");
        asset_manager.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        asset_manager.load_asset_from_data::<Mesh, Vec<VertexPosNormal>>("tri",isoc(1.0,1.0));
        let tri = asset_manager.create_renderable("card", "cards/card_10_clubs", "white", "white", 1.0).unwrap();
        //asset_manager.load_asset_from_data::<Texture, [f32; 4]>("white", [1.0, 1.0, 1.0, 1.0]);
        
        for card in self.state.percept().get_cards() {
                    world.create_now()
                        .with(tri.clone())
                        .with(CardThing{ card: card.clone()})
                        .with(LocalTransform::default())
                        .with(Transform::default())
                        .build();
        }

        world.create_now()
            .with(tri)
            .with(LocalTransform::default())
            .with(Transform::default())
            .build();
    }
    fn update(&mut self, world: &mut World, asset_manager: &mut AssetManager, _: &mut Pipeline) -> Trans {
        let percept = self.state.percept();
        let card_list = percept.get_cards();
        let (cards, mut transform, mut render) = (world.read::<CardThing>(), world.write::<LocalTransform>(), world.write::<Renderable>());
        let textures = asset_manager.read_assets::<Texture>();
        let mut drag_offset = [0.0; 3];
        let mut dragging = Vec::new();
        if let Some(card) = self.drag.clone() {
            dragging.push(card.clone());
            let mut data = percept.get_data_for(card).unwrap();
            let pos = self.ren_to_world(&data.pos);
            let mouse = [self.mouse.0, self.mouse.1, 1.0];
            for i in 0..3 {
                drag_offset[i] = mouse[i] - pos[i];
            }
            if let Some(mut children) = data.drag_children {
                for child in children.drain(..) {
                    dragging.push(child);
                }
            }
        }
        for (card, transform, render) in (&cards, &mut transform, &mut render).iter() {
            if let Some(data) = percept.get_data_for(card.card.clone()) {
                if dragging.contains(&card.card) {
                    let mut pos = self.ren_to_world(&data.pos);
                    for i in 0..3 {
                        pos[i] = pos[i] + drag_offset[i];
                    }
                    transform.translation = pos;
                } else {
                    transform.translation = self.ren_to_world(&data.pos);
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
        mouse: (0.0, 0.0)
    };
    let mut game = Application::build(initial, display_config)
        .register::<CardThing>()
        .with(CameraSystem, "aspect", 10)
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


