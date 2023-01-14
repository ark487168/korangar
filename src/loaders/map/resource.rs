use cgmath::{Deg, Vector3};
use derive_new::new;
use procedural::{PrototypeElement};

use crate::graphics::{Transform, Color};
use crate::loaders::{ByteConvertable, ByteStream};
use crate::world::{EffectSource, LightSource, SoundSource};

#[derive(Copy, Clone, Debug)]
pub enum ResourceType {
    Object,
    LightSource,
    SoundSource,
    EffectSource,
}

impl ResourceType {
    pub fn from(index: i32) -> Self {
        match index {
            1 => ResourceType::Object,
            2 => ResourceType::LightSource,
            3 => ResourceType::SoundSource,
            4 => ResourceType::EffectSource,
            _ => panic!("invalid object type {}", index),
        }
    }
}

#[derive(new)]
pub struct ObjectData {
    pub name: Option<String>,
    pub model_name: String,
    pub transform: Transform,
}
impl ObjectData {
    pub fn offset(&mut self, offset: Vector3<f32>) {
        self.transform.position += offset;
    }
}

#[allow(dead_code)]
pub struct MapResources {
    resources_amount: usize,
    pub objects: Vec<ObjectData>,
    pub light_sources: Vec<LightSource>,
    pub sound_sources: Vec<SoundSource>,
    pub effect_sources: Vec<EffectSource>,
}

impl ByteConvertable for MapResources {
    fn from_bytes(byte_stream: &mut ByteStream, _: Option<usize>) -> Self {
        let resources_amount = byte_stream.integer32() as usize;

        let mut objects = Vec::new();
        let mut light_sources = Vec::new();
        let mut sound_sources = Vec::new();
        let mut effect_sources = Vec::new();

        for index in 0..resources_amount {
            let type_index = byte_stream.integer32();
            let resource_type = ResourceType::from(type_index);

            match resource_type {
                ResourceType::Object => {
                    if byte_stream.get_version().equals_or_above(1, 6) {
                        let name = byte_stream.string(40);
                        let _animation_type = byte_stream.integer32();
                        let _animation_speed = byte_stream.float32();
                        let _block_type = byte_stream.integer32();
                        let model_name = byte_stream.string(80);
                        let _node_name = byte_stream.string(80);
                        let position = byte_stream.vector3_flipped();
                        let rotation = byte_stream.vector3();
                        let scale = byte_stream.vector3();
                        // offset the objects slightly to avoid depth buffer fighting
                        let position = position + Vector3::new(0.0, 0.0005, 0.0) * index as f32;
                        let transform = Transform::from(position, rotation.map(Deg), scale);
                        let object = ObjectData::new(Some(name), model_name, transform);
                        objects.push(object);
                    } else {
                        let model_name = byte_stream.string(80);
                        let _node_name = byte_stream.string(80);
                        let position = byte_stream.vector3_flipped();
                        let rotation = byte_stream.vector3();
                        let scale = byte_stream.vector3();
                        let transform = Transform::from(position, rotation.map(Deg), scale);
                        let object = ObjectData::new(None, model_name, transform);
                        objects.push(object);
                    }
                }
                ResourceType::LightSource => {
                    light_sources.push(LightSource::from_bytes(byte_stream, None));
                }
                ResourceType::SoundSource => {
                    let name = byte_stream.string(80);
                    let sound_file = byte_stream.string(80);
                    let position = byte_stream.vector3_flipped();
                    let volume = byte_stream.float32();
                    let width = byte_stream.integer32();
                    let height = byte_stream.integer32();
                    let range = byte_stream.float32();

                    let cycle = match byte_stream.get_version().equals_or_above(2, 0) {
                        true => byte_stream.float32(),
                        false => 4.0,
                    };

                    sound_sources.push(SoundSource::new(
                        name,
                        sound_file,
                        position,
                        volume,
                        width as usize,
                        height as usize,
                        range,
                        cycle,
                    ));
                }
                ResourceType::EffectSource => {
                    let name = byte_stream.string(80);
                    let position = byte_stream.vector3_flipped();
                    let effect_type = byte_stream.integer32();
                    let emit_speed = byte_stream.float32();

                    let _param0 = byte_stream.float32();
                    let _param1 = byte_stream.float32();
                    let _param2 = byte_stream.float32();
                    let _param3 = byte_stream.float32();

                    effect_sources.push(EffectSource::new(name, position, effect_type as usize, emit_speed));
                }
            }
        }

        Self {
            resources_amount,
            objects,
            light_sources,
            sound_sources,
            effect_sources,
        }
    }
}

#[derive(Debug, PrototypeElement, new)]
pub struct WaterSettings {
    #[new(value = "0.0")]
    pub water_level: f32,
    #[new(value = "0")]
    pub water_type: usize,
    #[new(value = "0.0")]
    pub wave_height: f32,
    #[new(value = "0.0")]
    pub wave_speed: f32,
    #[new(value = "0.0")]
    pub wave_pitch: f32,
    #[new(value = "0")]
    pub water_animation_speed: usize,
}

impl ByteConvertable for WaterSettings {
    fn from_bytes(byte_stream: &mut crate::loaders::ByteStream, length_hint: Option<usize>) -> Self {
        let mut water_settings = WaterSettings::new();

        if byte_stream.get_version().equals_or_above(1, 3) {
            let water_level = byte_stream.float32();
            water_settings.water_level = -water_level;
        }

        if byte_stream.get_version().equals_or_above(1, 8) {
            let water_type = byte_stream.integer32();
            let wave_height = byte_stream.float32();
            let wave_speed = byte_stream.float32();
            let wave_pitch = byte_stream.float32();

            water_settings.water_type = water_type as usize;
            water_settings.wave_height = wave_height;
            water_settings.wave_speed = wave_speed;
            water_settings.wave_pitch = wave_pitch;
        }

        if byte_stream.get_version().equals_or_above(1, 9) {
            let water_animation_speed = byte_stream.integer32();
            water_settings.water_animation_speed = water_animation_speed as usize;
        }
        water_settings
    }
}

#[derive(Debug, PrototypeElement, new)]
pub struct LightSettings {
    #[new(value = "0")]
    pub light_longitude: isize,
    #[new(value = "0")]
    pub light_latitude: isize,
    #[new(value = "Color::monochrome(255)")]
    pub diffuse_color: Color,
    #[new(value = "Color::monochrome(255)")]
    pub ambient_color: Color,
    #[new(value = "1.0")]
    pub light_intensity: f32,
}

impl ByteConvertable for LightSettings {
    fn from_bytes(byte_stream: &mut crate::loaders::ByteStream, length_hint: Option<usize>) -> Self {
        let mut light_settings = LightSettings::new();

        if byte_stream.get_version().equals_or_above(1, 5) {
            let light_longitude = byte_stream.integer32();
            let light_latitude = byte_stream.integer32();
            let diffuse_color = byte_stream.color();
            let ambient_color = byte_stream.color();

            light_settings.light_longitude = light_longitude as isize;
            light_settings.light_latitude = light_latitude as isize;
            light_settings.diffuse_color = diffuse_color;
            light_settings.ambient_color = ambient_color;

            if byte_stream.get_version().equals_or_above(1, 7) {
                light_settings.light_intensity = byte_stream.float32();
            }
        }
        light_settings
    }
}
