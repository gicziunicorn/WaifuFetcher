use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sites {
    Rule34,
    Nekos,
}

// implement Display to easily display the site name in the dropdown menu
impl std::fmt::Display for Sites {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Sites::Rule34 => "Rule 34",
            Sites::Nekos => "Nekos.moe",
        };
        write!(f, "{}", name)
    }
}

pub struct Site {
    pub value: Sites,
    pub name: &'static str,
    pub url: &'static str,
    pub image_path: &'static str,
}


impl Sites {
    pub fn details(&self) -> Site {
        match self {
            Sites::Rule34 => Site {
                value: *self,
                name: "Rule 34",
                url: "https://rule34.xxx/index.php?page=post&s=random",
                image_path: "assets/rule34_favicon.ico",
            },
            Sites::Nekos => Site {
                value: *self,
                name: "Nekos.moe",
                url: "https://nekos.moe/",
                image_path: "assets/nekos_favicon.ico",
            },
        }
    }

    pub fn value(&self) -> Sites {
        self.details().value
    }
    pub fn name(&self) -> &'static str {
        self.details().name
    }
    pub fn url(&self) -> &'static str {
        self.details().url
    }
    pub fn image_path(&self) -> &'static str {
        self.details().image_path
    }
}

// Give each site's response a struct so it can be parsed later
// use deserialize so if the api returns fileUrl it'll be file_url
#[derive(Debug, Clone, Deserialize)]
pub struct Rule34Post {
    pub preview_url: String,
    pub sample_url: String,
    pub file_url: String,
    pub directory: u32,
    pub hash: String,
    pub width: u32,
    pub height: u32,
    pub id: u32,
    pub image: String,
    pub change: u32,
    pub owner: String,
    pub parent_id: u32,
    pub rating: String,
    pub sample: bool,
    pub sample_height: u32,
    pub sample_width: u32,
    pub score: u32,
    pub tags: String,
    pub source: String,
    pub status: String,
    pub has_notes: bool,
    pub comment_count: u32,
}
pub type Rule34Response = Vec<Rule34Post>;

#[derive(Debug, Clone, Deserialize)]
pub struct NekosPost {
    pub id: String,
    pub nsfw: bool,
    pub artist: Option<String>,
    pub tags: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub likes: u32,
    pub favorites: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NekosResponse {
    pub images: Vec<NekosPost>,
}


pub enum Responses {
    Rule34(Rule34Response),
    Nekos(NekosResponse),
}