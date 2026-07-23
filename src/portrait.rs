use macroquad::prelude::*;
use std::collections::HashMap;

pub struct PortraitManager {
    portraits: HashMap<String, Texture2D>,
}

impl PortraitManager {
    pub fn new() -> Self {
        Self {
            portraits: HashMap::new(),
        }
    }

    pub async fn load_all(&mut self) {
        let portrait_files = [
            "captain_davey_portscan",
            "blowfish",
            "twofish",
            "rufus_reverse",
            "molly_hashpass",
            "deadbeef",
            "narrator",
        ];

        for name in portrait_files {
            let path = format!("assets/portraits/{}.png", name);
            if let Ok(texture) = load_texture(&path).await {
                texture.set_filter(FilterMode::Nearest);
                self.portraits.insert(name.to_string(), texture);
            } else {
                let placeholder = self.create_placeholder(name);
                self.portraits.insert(name.to_string(), placeholder);
            }
        }
    }

    fn create_placeholder(&self, name: &str) -> Texture2D {
        let size = 128;
        let mut image = Image::gen_image_color(size, size, match name {
            "captain_davey_portscan" => RED,
            "blowfish" => ORANGE,
            "twofish" => BLUE,
            "rufus_reverse" => PURPLE,
            "molly_hashpass" => PINK,
            "deadbeef" => GREEN,
            _ => GRAY,
        });
        
        let mut texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        texture
    }

    pub fn get(&self, name: &str) -> Option<&Texture2D> {
        self.portraits.get(name)
    }

    pub fn get_character(&self, character: &Character) -> Option<&Texture2D> {
        self.get(character.portrait_filename())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Character {
    #[default]
    CaptainDavey,
    Blowfish,
    Twofish,
    RufusReverse,
    MollyHashpass,
    Deadbeef,
    Narrator,
}

impl Character {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "captain_davey" | "captaindavey" | "davey" => Some(Character::CaptainDavey),
            "blowfish" => Some(Character::Blowfish),
            "twofish" => Some(Character::Twofish),
            "rufus_reverse" | "rufusreverse" => Some(Character::RufusReverse),
            "molly_hashpass" | "mollyhashpass" => Some(Character::MollyHashpass),
            "deadbeef" => Some(Character::Deadbeef),
            "narrator" => Some(Character::Narrator),
            _ => None,
        }
    }

    pub fn portrait_filename(&self) -> &'static str {
        match self {
            Character::CaptainDavey => "captain_davey_portscan",
            Character::Blowfish => "blowfish",
            Character::Twofish => "twofish",
            Character::RufusReverse => "rufus_reverse",
            Character::MollyHashpass => "molly_hashpass",
            Character::Deadbeef => "deadbeef",
            Character::Narrator => "narrator",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Character::CaptainDavey => "Captain Davey Portscan",
            Character::Blowfish => "Blowfish",
            Character::Twofish => "Twofish",
            Character::RufusReverse => "Rufus Reverse",
            Character::MollyHashpass => "Molly Hashpass",
            Character::Deadbeef => "Deadbeef",
            Character::Narrator => "Narrator",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Character::CaptainDavey => RED,
            Character::Blowfish => ORANGE,
            Character::Twofish => BLUE,
            Character::RufusReverse => PURPLE,
            Character::MollyHashpass => PINK,
            Character::Deadbeef => GREEN,
            Character::Narrator => GRAY,
        }
    }
}