use std::sync::LazyLock;

use super::Id;
use voxidian_protocol::{
    packet::s2c::play::SoundCategory as PtcSoundCategory,
    registry::Registry,
    value::{Sound as PtcSound, SoundEvent},
};
use wyvern_macros::generate_sounds_types;

static SOUND_REGISTRY: LazyLock<Registry<SoundEvent>> = LazyLock::new(SoundEvent::vanilla_registry);
#[derive(Debug, Clone)]
pub enum SoundCategory {
    Master,
}

impl From<SoundCategory> for PtcSoundCategory {
    fn from(value: SoundCategory) -> Self {
        match value {
            SoundCategory::Master => PtcSoundCategory::Master,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    pub(crate) name: Id,
    pub(crate) pitch: f32,
    pub(crate) volume: f32,
    pub(crate) category: SoundCategory,
}

impl Sound {
    pub fn pitch(mut self, value: f32) -> Self {
        self.pitch = value;
        self
    }

    pub fn volume(mut self, value: f32) -> Self {
        self.volume = value;
        self
    }

    pub fn get_pitch(&self) -> f32 {
        self.pitch
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    pub fn category(mut self, category: SoundCategory) -> Self {
        self.category = category;
        self
    }
}

impl From<Sound> for PtcSound {
    fn from(value: Sound) -> Self {
        PtcSound::Inline(SoundEvent {
            name: value.name.into(),
            fixed_range: None,
        })
    }
}

impl From<PtcSound> for Sound {
    fn from(value: PtcSound) -> Self {
        match value {
            PtcSound::Registry(reg_entry) => {
                SOUND_REGISTRY.lookup(&reg_entry).unwrap().clone().into()
            }
            PtcSound::Inline(sound_event) => sound_event.into(),
        }
    }
}

impl From<SoundEvent> for Sound {
    fn from(value: SoundEvent) -> Self {
        Sound {
            name: value.name.into(),
            pitch: 1.0,
            volume: 1.0,
            category: SoundCategory::Master,
        }
    }
}

pub struct Sounds;

generate_sounds_types!();
