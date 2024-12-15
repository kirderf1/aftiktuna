pub mod color {
    use super::Error;
    use crate::core::display::AftikColorId;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::fs::File;

    pub const DEFAULT_COLOR: AftikColorData = AftikColorData {
        primary_color: RGBColor::new(255, 255, 255),
        secondary_color: RGBColor::new(0, 0, 0),
    };

    #[derive(Clone, Serialize, Deserialize)]
    pub struct AftikColorData {
        pub primary_color: RGBColor,
        pub secondary_color: RGBColor,
    }

    #[derive(Clone, Copy, Serialize, Deserialize)]
    pub struct RGBColor {
        pub r: u8,
        pub g: u8,
        pub b: u8,
    }

    impl RGBColor {
        pub const fn new(r: u8, g: u8, b: u8) -> Self {
            Self { r, g, b }
        }
    }

    pub const AFTIK_COLORS_PATH: &str = "assets/aftik_colors.json";

    pub fn load_aftik_color_data() -> Result<HashMap<AftikColorId, AftikColorData>, Error> {
        let file = File::open(AFTIK_COLORS_PATH)?;
        Ok(serde_json::from_reader::<
            _,
            HashMap<AftikColorId, AftikColorData>,
        >(file)?)
    }

    #[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ColorSource {
        #[default]
        Uncolored,
        Primary,
        Secondary,
    }

    impl ColorSource {
        pub fn get_color(self, aftik_color_data: &AftikColorData) -> RGBColor {
            match self {
                ColorSource::Uncolored => RGBColor::new(255, 255, 255),
                ColorSource::Primary => aftik_color_data.primary_color,
                ColorSource::Secondary => aftik_color_data.secondary_color,
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
