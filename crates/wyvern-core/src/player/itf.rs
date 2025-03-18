use wyvern_actors::{ActorError, ActorResult};
use wyvern_datatypes::{gamemode::Gamemode, text::Text};
use wyvern_values::{Uuid, Vec2, Vec3};

use super::{Player, PlayerComponents};

impl Player {
    pub fn teleport(&self, position: Vec3<f64>) -> ActorResult<()> {
        self.set(PlayerComponents::TELEPORT_POSITION, position)
    }

    pub fn set_velocity(&self, position: Vec3<f64>) -> ActorResult<()> {
        self.set(PlayerComponents::TELEPORT_VELOCITY, position)
    }

    pub fn set_gamemode(&self, gamemode: Gamemode) -> ActorResult<()> {
        self.set(PlayerComponents::GAMEMODE, gamemode)
    }

    pub fn set_health(&self, health: f32) -> ActorResult<()> {
        let mut hp = self.get(PlayerComponents::HEALTH)?;
        hp.health = health;
        self.set(PlayerComponents::HEALTH, hp)
    }

    pub fn set_food(&self, food: i32) -> ActorResult<()> {
        let mut hp = self.get(PlayerComponents::HEALTH)?;
        hp.food = food;
        self.set(PlayerComponents::HEALTH, hp)
    }

    pub fn set_saturation(&self, saturation: f32) -> ActorResult<()> {
        let mut hp = self.get(PlayerComponents::HEALTH)?;
        hp.saturation = saturation;
        self.set(PlayerComponents::HEALTH, hp)
    }

    pub fn username(&self) -> ActorResult<String> {
        self.get(PlayerComponents::USERNAME)
    }

    pub fn uuid(&self) -> ActorResult<Uuid> {
        self.get(PlayerComponents::UUID)
    }

    pub fn sidebar(&self) -> PlayerSidebar<'_> {
        PlayerSidebar { player: self }
    }

    pub fn world_border(&self) -> PlayerWorldBorder<'_> {
        PlayerWorldBorder { player: self }
    }
}

pub struct PlayerSidebar<'a> {
    player: &'a Player,
}

impl PlayerSidebar<'_> {
    pub fn get_line(&self, idx: usize) -> ActorResult<Text> {
        let mut a = self.player.get(PlayerComponents::SIDEBAR_LINES)?;
        Ok(std::mem::replace(
            a.get_mut(idx).ok_or(ActorError::IndexOutOfBounds)?,
            Text::literal(""),
        ))
    }

    pub fn set_line(&self, idx: usize, value: Text) -> ActorResult<()> {
        let mut a = self.player.get(PlayerComponents::SIDEBAR_LINES)?;
        while a.len() < idx {
            a.push(Text::literal(""));
        }
        a.push(value);
        self.player.set(PlayerComponents::SIDEBAR_LINES, a)?;
        Ok(())
    }

    pub fn add_line(&self, value: Text) -> ActorResult<()> {
        let mut a = self.player.get(PlayerComponents::SIDEBAR_LINES)?;
        a.push(value);
        self.player.set(PlayerComponents::SIDEBAR_LINES, a)?;
        Ok(())
    }

    pub fn remove_line(&self, idx: usize) -> ActorResult<()> {
        let mut a = self.player.get(PlayerComponents::SIDEBAR_LINES)?;
        a.remove(idx);
        self.player.set(PlayerComponents::SIDEBAR_LINES, a)?;
        Ok(())
    }

    pub fn get_name(&self) -> ActorResult<Text> {
        self.player.get(PlayerComponents::SIDEBAR_NAME)
    }
}

pub struct PlayerWorldBorder<'a> {
    player: &'a Player,
}

impl PlayerWorldBorder<'_> {
    pub fn size(&self) -> ActorResult<f64> {
        Ok(self.player.get(PlayerComponents::WORLD_BORDER)?.size)
    }

    pub fn set_size(&self, size: f64) -> ActorResult<()> {
        let mut c = self.player.get(PlayerComponents::WORLD_BORDER)?;
        c.size = size;
        self.player.set(PlayerComponents::WORLD_BORDER, c)
    }

    pub fn center(&self) -> ActorResult<Vec2<f64>> {
        Ok(self.player.get(PlayerComponents::WORLD_BORDER)?.center)
    }

    pub fn set_center(&self, center: Vec2<f64>) -> ActorResult<()> {
        let mut c = self.player.get(PlayerComponents::WORLD_BORDER)?;
        c.center = center;
        self.player.set(PlayerComponents::WORLD_BORDER, c)
    }

    pub fn warning_delay(&self) -> ActorResult<i32> {
        Ok(self
            .player
            .get(PlayerComponents::WORLD_BORDER)?
            .warning_delay)
    }

    pub fn set_warning_delay(&self, warning_delay: i32) -> ActorResult<()> {
        let mut c = self.player.get(PlayerComponents::WORLD_BORDER)?;
        c.warning_delay = warning_delay;
        self.player.set(PlayerComponents::WORLD_BORDER, c)
    }

    pub fn warning_size(&self) -> ActorResult<i32> {
        Ok(self
            .player
            .get(PlayerComponents::WORLD_BORDER)?
            .warning_distance)
    }

    pub fn set_warning_size(&self, warning_size: i32) -> ActorResult<()> {
        let mut c = self.player.get(PlayerComponents::WORLD_BORDER)?;
        c.warning_distance = warning_size;
        self.player.set(PlayerComponents::WORLD_BORDER, c)
    }
}
