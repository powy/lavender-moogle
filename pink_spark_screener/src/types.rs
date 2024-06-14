// Type describing the key binding associated with a spell

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SpellBinding {
    pub modifier: char,
    pub key: char,
}
