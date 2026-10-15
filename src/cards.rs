use serde::{Deserialize, Serialize};

/// Closed, explicit catalogue. Unknown cards fail deserialization, never become vanilla cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Card {
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
    GrizzlyBears,
    GoblinPiker,
    HillGiant,
    LlanowarElves,
    LightningBolt,
    Shock,
    GiantGrowth,
    Divination,
    Counterspell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    Creature(i32, i32),
    Damage(i32),
    Grow,
    Draw,
    Counter,
    Land,
}

/// Mana is W U B R G. Generic costs deliberately have no colour slot.
pub type Mana = [u32; 5];

impl Card {
    pub const ALL: [Card; 14] = [
        Self::Plains,
        Self::Island,
        Self::Swamp,
        Self::Mountain,
        Self::Forest,
        Self::GrizzlyBears,
        Self::GoblinPiker,
        Self::HillGiant,
        Self::LlanowarElves,
        Self::LightningBolt,
        Self::Shock,
        Self::GiantGrowth,
        Self::Divination,
        Self::Counterspell,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Plains => "Plains",
            Self::Island => "Island",
            Self::Swamp => "Swamp",
            Self::Mountain => "Mountain",
            Self::Forest => "Forest",
            Self::GrizzlyBears => "Grizzly Bears",
            Self::GoblinPiker => "Goblin Piker",
            Self::HillGiant => "Hill Giant",
            Self::LlanowarElves => "Llanowar Elves",
            Self::LightningBolt => "Lightning Bolt",
            Self::Shock => "Shock",
            Self::GiantGrowth => "Giant Growth",
            Self::Divination => "Divination",
            Self::Counterspell => "Counterspell",
        }
    }

    pub(crate) fn effect(self) -> Effect {
        match self {
            Self::Plains | Self::Island | Self::Swamp | Self::Mountain | Self::Forest => {
                Effect::Land
            }
            Self::GrizzlyBears => Effect::Creature(2, 2),
            Self::GoblinPiker => Effect::Creature(2, 1),
            Self::HillGiant => Effect::Creature(3, 3),
            Self::LlanowarElves => Effect::Creature(1, 1),
            Self::LightningBolt => Effect::Damage(3),
            Self::Shock => Effect::Damage(2),
            Self::GiantGrowth => Effect::Grow,
            Self::Divination => Effect::Draw,
            Self::Counterspell => Effect::Counter,
        }
    }

    pub fn is_creature(self) -> bool {
        matches!(self.effect(), Effect::Creature(..))
    }
    pub fn is_land(self) -> bool {
        self.effect() == Effect::Land
    }
    pub fn is_instant(self) -> bool {
        matches!(
            self.effect(),
            Effect::Damage(_) | Effect::Grow | Effect::Counter
        )
    }

    pub(crate) fn mana_color(self) -> Option<usize> {
        match self {
            Self::Plains => Some(0),
            Self::Island => Some(1),
            Self::Swamp => Some(2),
            Self::Mountain => Some(3),
            Self::Forest | Self::LlanowarElves => Some(4),
            _ => None,
        }
    }

    pub(crate) fn cost(self) -> (Mana, u32) {
        match self {
            Self::GrizzlyBears => ([0, 0, 0, 0, 1], 1),
            Self::GoblinPiker => ([0, 0, 0, 1, 0], 1),
            Self::HillGiant => ([0, 0, 0, 1, 0], 3),
            Self::LlanowarElves | Self::GiantGrowth => ([0, 0, 0, 0, 1], 0),
            Self::LightningBolt | Self::Shock => ([0, 0, 0, 1, 0], 0),
            Self::Divination => ([0, 1, 0, 0, 0], 2),
            Self::Counterspell => ([0, 2, 0, 0, 0], 0),
            _ => ([0; 5], 0),
        }
    }
}

pub(crate) fn payments(pool: Mana, colored: Mana, generic: u32, mut emit: impl FnMut(Mana)) {
    if (0..5).any(|i| pool[i] < colored[i]) {
        return;
    }
    fn visit(i: usize, left: u32, pool: Mana, mut paid: Mana, emit: &mut impl FnMut(Mana)) {
        if i == 4 {
            if pool[i] >= left {
                paid[i] += left;
                emit(paid);
            }
            return;
        }
        for n in 0..=pool[i].min(left) {
            let old = paid[i];
            paid[i] = old + n;
            visit(i + 1, left - n, pool, paid, emit);
            paid[i] = old;
        }
    }
    let remaining = std::array::from_fn(|i| pool[i] - colored[i]);
    visit(0, generic, remaining, colored, &mut emit);
}
