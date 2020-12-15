use crate::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Animation {
    textures: Vec<u32>,
    frame_rate: f32,
}

impl Animation {
    pub fn new(textures: impl IntoIterator<Item = u32>, frame_rate: f32) -> Self {
        Self {
            textures: textures.into_iter().collect(),
            frame_rate,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Animator {
    animations: HashMap<String, Animation>,
    current_animation: String,
    current_frame_time: f32,
}
serde_sync!(Animator = 768961378612578127962342341342);

impl Animator {
    pub fn new() -> AnimatorBuilder {
        AnimatorBuilder {
            animations: HashMap::new(),
        }
    }

    pub fn play(&mut self, name: impl Into<String>) {
        self.current_animation = name.into();
        self.current_frame_time = 0.0;
    }

    pub fn set_playing(&mut self, name: impl Into<String>) {
        self.current_animation = name.into();
    }

    pub fn current_frame(&self) -> usize {
        let current_animation = &self.animations[&self.current_animation];

        (self.current_frame_time / current_animation.frame_rate).floor() as usize
    }
}

pub struct AnimatorBuilder {
    animations: HashMap<String, Animation>,
}

impl AnimatorBuilder {
    pub fn add_animation(&mut self, name: impl Into<String>, animation: Animation) {
        self.animations.insert(name.into(), animation);
    }

    pub fn build(self) -> Animator {
        Animator {
            current_animation: self
                .animations
                .iter()
                .next()
                .map(|(s, _)| s.clone())
                .unwrap(),
            animations: self.animations,
            current_frame_time: 0.0,
        }
    }
}

pub fn animator_system(time: Res<Time>, mut query: Query<&mut Animator>) {
    for mut animator in query.iter_mut() {
        let Animator {
            animations,
            current_animation,
            current_frame_time,
        } = &mut *animator;

        let current_animation = &animations[current_animation];
        *current_frame_time += time.delta_seconds();

        if *current_frame_time
            > current_animation.frame_rate * (current_animation.textures.len() - 1) as f32
        {
            *current_frame_time = 0.0;
        }
    }
}

pub fn animator_sprite_system(mut query: Query<(&mut TextureAtlasSprite, &Animator)>) {
    for (mut sprite, animator) in query.iter_mut() {
        let Animator {
            animations,
            current_animation,
            current_frame_time,
        } = &*animator;

        let current_animation = &animations[current_animation];

        sprite.index = current_animation.textures
            [(*current_frame_time / current_animation.frame_rate).floor() as usize];
    }
}
