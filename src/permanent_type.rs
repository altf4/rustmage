//! Shared typed permanent categories for casting and resolution choices.
use serde::{Deserialize, Serialize};

/// Permanent types are choices, rather than an opaque kind mask in the action API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermanentType {
    Artifact,
    Battle,
    Creature,
    Enchantment,
    Land,
    Planeswalker,
}
impl PermanentType {
    pub(crate) const ALL: [Self; 6] = [
        Self::Artifact,
        Self::Battle,
        Self::Creature,
        Self::Enchantment,
        Self::Land,
        Self::Planeswalker,
    ];
    pub(crate) fn mask(self) -> u16 {
        match self {
            Self::Creature => 1,
            Self::Land => 2,
            Self::Artifact => 16,
            Self::Enchantment => 32,
            Self::Planeswalker => 64,
            Self::Battle => 256,
        }
    }
}
