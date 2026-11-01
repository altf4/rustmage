//! Immutable catalogue link boundary. Only the public assembly supplies data.
use super::authored::CardRules;
use super::*;

pub struct CatalogueEntry {
    pub definition: Definition,
    pub rules: &'static CardRules,
}

/// Private, exact-version workspace interface, not a dynamically loaded ABI.
pub struct Catalogue {
    pub version: &'static str,
    pub colors: &'static [u8],
    pub definitions: &'static [&'static Definition],
    pub names: &'static [(&'static str, u16)],
    pub rules: &'static [&'static CardRules],
    pub supported: &'static [Card],
    pub back_face: fn(Card) -> Option<Card>,
    pub face_parent: fn(Card) -> Option<Card>,
    pub physical_identity: fn(Card) -> Option<&'static str>,
    pub token_info: fn(Card) -> Option<&'static TokenInfo>,
    pub emblem_info: fn(Card) -> Option<&'static EmblemInfo>,
    pub card_names: &'static str,
    pub land_names: &'static str,
    pub interchangeable_names: &'static str,
    pub outside_names: &'static str,
}

unsafe extern "Rust" {
    #[link_name = "__rustmage_catalogue_v1_0_1_0"]
    static LINKED_CATALOGUE: Catalogue;
}

pub fn catalogue() -> &'static Catalogue {
    // SAFETY: rustmage's assembly defines this immutable static with the very same
    // Catalogue type from this exact-version dependency. It and all borrowed data
    // live for the process lifetime. Linking fails if the assembly is absent.
    unsafe { &LINKED_CATALOGUE }
}

/// Slice-like compatibility view for `Card::ALL`, without baking admission data
/// into the engine's compilation. Iteration by value still yields copied Cards.
#[derive(Clone, Copy, Default)]
pub struct SupportedCards;
impl SupportedCards {
    pub fn as_slice(&self) -> &'static [Card] {
        catalogue().supported
    }
    pub fn iter(&self) -> std::slice::Iter<'static, Card> {
        self.as_slice().iter()
    }
}
impl std::ops::Deref for SupportedCards {
    type Target = [Card];
    fn deref(&self) -> &'static [Card] {
        catalogue().supported
    }
}
impl AsRef<[Card]> for SupportedCards {
    fn as_ref(&self) -> &[Card] {
        self
    }
}
impl std::fmt::Debug for SupportedCards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().fmt(f)
    }
}
impl IntoIterator for SupportedCards {
    type Item = Card;
    type IntoIter = std::iter::Copied<std::slice::Iter<'static, Card>>;
    fn into_iter(self) -> Self::IntoIter {
        catalogue().supported.iter().copied()
    }
}
impl<'a> IntoIterator for &'a SupportedCards {
    type Item = &'a Card;
    type IntoIter = std::slice::Iter<'a, Card>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
use std::ops::Deref;

#[allow(non_upper_case_globals)]
impl Card {
    pub const Plains: Card = Card(0);
    pub const Island: Card = Card(1);
    pub const Swamp: Card = Card(2);
    pub const Mountain: Card = Card(3);
    pub const Forest: Card = Card(4);
    pub const GrizzlyBears: Card = Card(5);
    pub const GoblinPiker: Card = Card(6);
    pub const HillGiant: Card = Card(7);
    pub const LlanowarElves: Card = Card(8);
    pub const LightningBolt: Card = Card(9);
    pub const Shock: Card = Card(10);
    pub const GiantGrowth: Card = Card(11);
    pub const Divination: Card = Card(12);
    pub const Counterspell: Card = Card(13);
    pub const DrakeToken: Card = Card(5339);
    pub const KaitoNinjaToken: Card = Card(5340);
    pub const KaitoEmblem: Card = Card(5341);
    pub const SosFractalToken: Card = Card(5347);
    pub const SosPestToken: Card = Card(5348);
    pub const SosElementalToken: Card = Card(5349);
    pub const SosSpiritToken: Card = Card(5350);
    pub const MshLeviathanToken: Card = Card(5387);
    pub const MshRobotVillainToken: Card = Card(5388);
    pub const MshTigerGodToken: Card = Card(5389);
    pub const MshPhotonLivingLight: Card = Card(5391);
    pub const MshBlackPantherHopeEnduring: Card = Card(5392);
    pub const MshSensationalSheHulk: Card = Card(5396);
    pub const MshDoombotToken: Card = Card(5397);
    pub const MshBlueMerfolkToken: Card = Card(5398);
    pub const MshGalactusToken: Card = Card(5399);
    pub const MshSturdyShieldToken: Card = Card(5400);
    pub const GoblinArmyToken: Card = Card(5405);
    pub const JacePlaneswalkerToken: Card = Card(5435);
}

include!("catalogue_accessors.rs");
