/// sliding animations.
use std::f32;
use amethyst::ecs::{
    Component,
    Join,
    RunArg,
    System,
    VecStorage,
};
use amethyst::ecs::resources::{
    Time,
};
use amethyst::ecs::components::{
    LocalTransform
};


pub struct MoveTarget {
    pub pos: [f32; 3]
}

impl Component for MoveTarget {
    type Storage = VecStorage<MoveTarget>;
}

pub struct MoveSystem {
    pub vel: f32
}

impl System<()> for MoveSystem {
    fn run(&mut self, arg: RunArg, _:()) {
        let (mut transform, mut target, time) = arg.fetch(|w| {
            (w.write::<LocalTransform>(), w.write::<MoveTarget>(), w.read_resource::<Time>())
        });
        let delta = time.delta_time.as_secs() as f32 + time.delta_time.subsec_nanos() as f32 / 1000000000.0;
        for (mut transform, mut target) in (&mut transform, &mut target).iter() {
            if transform.translation != target.pos {
                let mut dir = [0.0; 3];
                let mut sum = 0.0;
                for i in 0..3 {
                    dir[i] = target.pos[i] - transform.translation[i];
                    sum = sum + dir[i]*dir[i];
                }
                sum = sum.sqrt();
                if sum < delta*self.vel {
                    transform.translation = target.pos;
                } else {
                    for i in 0..3 {
                        transform.translation[i] = transform.translation[i] + dir[i]*delta*self.vel/sum;
                    }
                }
            }
        }
    }
}
