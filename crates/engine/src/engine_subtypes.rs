//! Layer-four creature subtype overlays. Objects retain only their printed identity;
//! captured views carry a compact value, and speculative resource queries borrow
//! the same exact attachment test over their own object overlay.
use super::*;
use crate::cards::StaticAbilityFragment;
use serde::ser::{SerializeSeq, SerializeStruct};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const LAND_TYPES: [&str; 17] = [
    "Cave",
    "Desert",
    "Forest",
    "Gate",
    "Island",
    "Lair",
    "Locus",
    "Mine",
    "Mountain",
    "Plains",
    "Power-Plant",
    "Sphere",
    "Swamp",
    "Tower",
    "Town",
    "Planet",
    "Urza's",
];
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const JOB_TYPES: [&str; 17] = [
    "Artificer",
    "Bard",
    "Cleric",
    "Knight",
    "Monk",
    "Ninja",
    "Rogue",
    "Samurai",
    "Shaman",
    "Warrior",
    "Wizard",
    "Angel",
    "Demon",
    "Nightmare",
    "Golem",
    "Fractal",
    "Hero",
];
// Named grants and projection flags share the same fixed-width mask.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const CREATURE_OR_KINDRED: u16 = 1 | 128;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const JOB_MASK: u32 = (1 << JOB_TYPES.len()) - 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const SUPPRESS_PRINTED: u32 = 1 << JOB_TYPES.len();
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const LAND_ONLY: u32 = SUPPRESS_PRINTED << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ALL_CREATURE_TYPES: u32 = LAND_ONLY << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const FLAGS_MASK: u32 =
    SUPPRESS_PRINTED | LAND_ONLY | ALL_CREATURE_TYPES | SUPPRESS_PRINTED_ARTIFACT;
// Preserve all existing bit positions. Copy exceptions use previously unused
// high bits, keeping the common borrowed subtype value at its original size.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const COPY_HUMAN: u32 = ALL_CREATURE_TYPES << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const COPY_VILLAIN: u32 = COPY_HUMAN << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const COPY_MERCENARY: u32 = COPY_VILLAIN << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_SOLDIER: u32 = COPY_MERCENARY << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_GOBLIN: u32 = ADDED_SOLDIER << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_BEAR: u32 = ADDED_GOBLIN << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_FOOD: u32 = ADDED_BEAR << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_SPIRIT: u32 = ADDED_FOOD << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_CONSTRUCT: u32 = ADDED_SPIRIT << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_EQUIPMENT: u32 = ADDED_CONSTRUCT << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const SUPPRESS_PRINTED_ARTIFACT: u32 = ADDED_EQUIPMENT << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_TREASURE: u32 = SUPPRESS_PRINTED_ARTIFACT << 1;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_ARTIFACT_SUBTYPES: u32 = ADDED_FOOD | ADDED_EQUIPMENT | ADDED_TREASURE;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const ADDED_MASK: u32 = JOB_MASK
    | COPY_HUMAN
    | COPY_VILLAIN
    | COPY_MERCENARY
    | ADDED_SOLDIER
    | ADDED_GOBLIN
    | ADDED_BEAR
    | ADDED_SPIRIT
    | ADDED_CONSTRUCT
    | ADDED_ARTIFACT_SUBTYPES;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const _: () = assert!(JOB_TYPES.len() + 15 <= u32::BITS as usize);
// Only query values and frozen views carry this fixed taxonomy mask. Hot Object
// storage is unchanged; inherited sparse grants retain their original wire form.
const CREATURE_TYPE_WORDS: usize = crate::cards::CREATURE_TYPES.len().div_ceil(64);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct CreatureTypeSet([u64; CREATURE_TYPE_WORDS]);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CreatureTypeSet {
    pub(super) fn is_empty(self) -> bool {
        self.0 == [0; CREATURE_TYPE_WORDS]
    }
    fn insert(&mut self, typ: crate::cards::CreatureType) {
        self.insert_index(typ.index());
    }
    fn insert_index(&mut self, index: usize) {
        self.0[index / 64] |= 1 << (index % 64);
    }
    pub(super) fn union(&mut self, other: Self) {
        for (word, extra) in self.0.iter_mut().zip(other.0) {
            *word |= extra;
        }
    }
    pub(super) fn intersects(self, other: Self) -> bool {
        self.0.into_iter().zip(other.0).any(|(a, b)| a & b != 0)
    }
    pub(super) fn indices(self) -> impl Iterator<Item = usize> {
        self.0
            .into_iter()
            .enumerate()
            .flat_map(|(block, mut word)| {
                std::iter::from_fn(move || {
                    if word == 0 {
                        return None;
                    }
                    let offset = word.trailing_zeros() as usize;
                    word &= word - 1;
                    Some(block * u64::BITS as usize + offset)
                })
            })
    }
    pub(super) fn contains(self, name: &str) -> bool {
        !self.is_empty()
            && crate::cards::CREATURE_TYPES
                .binary_search(&name)
                .is_ok_and(|index| self.0[index / 64] & (1 << (index % 64)) != 0)
    }
}
impl Serialize for CreatureTypeSet {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let count = self.0.iter().map(|word| word.count_ones() as usize).sum();
        let mut out = serializer.serialize_seq(Some(count))?;
        for (index, name) in crate::cards::CREATURE_TYPES.iter().enumerate() {
            if self.0[index / 64] & (1 << (index % 64)) != 0 {
                out.serialize_element(name)?;
            }
        }
        out.end()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantedSubtypes {
    added: u32,
    chosen: Option<crate::cards::CreatureType>,
    additional_creature_types: CreatureTypeSet,
    /// A layer-four effect replaces all previously held land subtypes.
    basic_land: Option<crate::cards::BasicLandType>,
    added_basic_lands: u8,
    added_cave: bool,
    effective_basic_lands: Option<u8>,
    suppress_printed_land: bool,
    /// None retains printed types (including changeling); Some replaces only
    /// creature types, preserving e.g. Equipment, Aura, Forest and Lesson.
    creature_types: Option<&'static [&'static str]>,
    /// Opt-in correlation after a card-type-setting effect; final layer-four types.
    card_types: Option<u16>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl GrantedSubtypes {
    pub(super) fn food_only() -> Self {
        Self::only_artifact_subtype(ADDED_FOOD)
    }
    pub(super) fn treasure_only() -> Self {
        Self::only_artifact_subtype(ADDED_TREASURE)
    }
    fn only_artifact_subtype(subtype: u32) -> Self {
        Self {
            added: SUPPRESS_PRINTED | subtype,
            chosen: None,
            additional_creature_types: CreatureTypeSet::default(),
            basic_land: None,
            added_basic_lands: 0,
            added_cave: false,
            effective_basic_lands: None,
            suppress_printed_land: false,
            creature_types: Some(&[]),
            card_types: None,
        }
    }
    fn flag(self, bit: u32) -> bool {
        self.added & bit != 0
    }
    fn set_flag(&mut self, bit: u32, value: bool) {
        if value {
            self.added |= bit;
        } else {
            self.added &= !bit;
        }
    }
    /// Membership supplied by this overlay, excluding retained printed types.
    /// Use ObjectView::has_subtype for the full effective characteristic.
    pub fn contains(self, subtype: &str) -> bool {
        if self
            .card_types
            .is_some_and(|kinds| kinds & subtype_card_types(subtype) == 0)
        {
            return false;
        }
        (self.added_cave && subtype == "Cave")
            || self.basic_land.is_some_and(|kind| kind.name() == subtype)
            || (self.added_basic_lands != 0
                && crate::cards::BasicLandType::ALL.iter().any(|kind| {
                    kind.name() == subtype && self.added_basic_lands & (1 << kind.index()) != 0
                }))
            || (self.added & Self::copy_type_bit(subtype) != 0)
            || (self.flag(ALL_CREATURE_TYPES) && crate::cards::is_creature_type(subtype))
            || self.chosen.is_some_and(|t| t.name() == subtype)
            || self.additional_creature_types.contains(subtype)
            || self
                .creature_types
                .is_some_and(|types| types.contains(&subtype))
            || JOB_TYPES
                .iter()
                .position(|s| *s == subtype)
                .is_some_and(|i| self.added & (1 << i) != 0)
    }
    pub(crate) fn has(self, card: Card, subtype: &str) -> bool {
        if self
            .card_types
            .is_some_and(|kinds| kinds & subtype_card_types(subtype) == 0)
        {
            return false;
        }
        if let Some(bits) = self.effective_basic_lands
            && let Some(kind) = crate::cards::BasicLandType::ALL
                .into_iter()
                .find(|kind| kind.name() == subtype)
        {
            return bits & (1 << kind.index()) != 0;
        }
        if self.suppress_printed_land && LAND_TYPES.contains(&subtype) {
            return self.contains(subtype);
        }
        if self.flag(SUPPRESS_PRINTED_ARTIFACT) && subtype_card_types(subtype) == 16 {
            return self.contains(subtype);
        }
        if LAND_TYPES.contains(&subtype)
            && let Some(basic) = self.basic_land
        {
            return basic.name() == subtype || self.contains(subtype);
        }
        // Preserve the ordinary-card hot path: no taxonomy lookup, and printed
        // membership short-circuits the additional-type mask as before.
        if self.flag(LAND_ONLY) {
            return self.contains(subtype)
                || (!self.flag(SUPPRESS_PRINTED)
                    && LAND_TYPES.contains(&subtype)
                    && card.has_subtype(subtype));
        }
        if self.flag(SUPPRESS_PRINTED)
            || (self.creature_types.is_some() && crate::cards::is_creature_type(subtype))
        {
            self.contains(subtype)
        } else {
            card.has_subtype(subtype) || self.contains(subtype)
        }
    }
    pub(super) fn creature_type_set(self, card: Card) -> CreatureTypeSet {
        let mut result = CreatureTypeSet::default();
        if self == Self::default() && !card.keywords().contains(Keywords::CHANGELING) {
            if let Some((_, types)) = card.info().type_line.split_once(" — ") {
                let mut words = types.split_whitespace().peekable();
                while let Some(mut word) = words.next() {
                    if word == "Time" && words.next_if_eq(&"Lord").is_some() {
                        word = "Time Lord";
                    }
                    if let Some(typ) = crate::cards::CreatureType::from_name(word) {
                        result.insert(typ);
                    }
                }
            }
            return result;
        }
        for (index, subtype) in crate::cards::CREATURE_TYPES.iter().enumerate() {
            if self.has(card, subtype) {
                result.insert_index(index);
            }
        }
        result
    }

    fn replace_creature_types(&mut self, types: &'static [&'static str]) {
        self.added &= FLAGS_MASK | ADDED_ARTIFACT_SUBTYPES;
        self.chosen = None;
        self.additional_creature_types = CreatureTypeSet::default();
        self.creature_types = Some(types);
    }
    fn apply_copy_subtypes(&mut self, recipe: Option<CopyRecipe>) {
        if let Some(recipe) = recipe {
            let types = recipe.copy_types();
            if types.replace {
                *self = Self::default();
                self.set_flag(SUPPRESS_PRINTED, true);
            }
            if types.human_villain {
                self.insert("Human");
                self.insert("Villain");
            }
            if types.mercenary {
                self.insert("Mercenary");
            }
            if types.nightmare {
                self.insert("Nightmare");
            }
            if types.golem {
                self.insert("Golem");
            }
            if types.fractal {
                self.insert("Fractal");
            }
            if types.zombie {
                self.insert("Zombie");
            }
            if types.mutant {
                self.insert("Mutant");
            }
            if recipe.exceptions.reflection {
                self.insert("Reflection");
            }
            if recipe.exceptions.balloon && recipe.kinds() & CREATURE_OR_KINDRED != 0 {
                self.insert("Balloon");
            }
            if recipe.exceptions.cave && recipe.kinds() & 2 != 0 {
                self.added_cave = true;
            }
            if recipe.exceptions.spider_human_hero {
                self.insert("Spider");
                self.insert("Human");
                self.insert("Hero");
            }
            if recipe.exceptions.shapeshifter_rogue {
                self.insert("Shapeshifter");
                self.insert("Rogue");
            }
            if recipe.exceptions.bird && recipe.kinds() & CREATURE_OR_KINDRED != 0 {
                self.insert("Bird");
            }
            if let Some(subtype) = recipe.replacement_subtype {
                self.insert(subtype.name());
            }
        }
    }
    fn copy_type_bit(subtype: &str) -> u32 {
        match subtype {
            "Human" => COPY_HUMAN,
            "Villain" => COPY_VILLAIN,
            "Mercenary" => COPY_MERCENARY,
            "Soldier" => ADDED_SOLDIER,
            "Goblin" => ADDED_GOBLIN,
            "Bear" => ADDED_BEAR,
            "Food" => ADDED_FOOD,
            "Spirit" => ADDED_SPIRIT,
            "Construct" => ADDED_CONSTRUCT,
            "Equipment" => ADDED_EQUIPMENT,
            "Treasure" => ADDED_TREASURE,
            _ => 0,
        }
    }
    pub(super) fn add_creature_types(&mut self, types: &'static [&'static str]) {
        for subtype in types {
            self.insert(subtype);
        }
    }
    fn insert(&mut self, subtype: &str) {
        let copy = Self::copy_type_bit(subtype);
        if copy != 0 {
            self.added |= copy;
            return;
        }
        if let Some(i) = JOB_TYPES.iter().position(|s| *s == subtype) {
            self.added |= 1 << i;
        } else {
            self.additional_creature_types.insert(
                crate::cards::CreatureType::from_name(subtype)
                    .expect("authenticated immutable creature subtype grant"),
            );
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Serialize for GrantedSubtypes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Added(u32, bool);
        impl Serialize for Added {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut out = serializer
                    .serialize_seq(Some(self.0.count_ones() as usize + usize::from(self.1)))?;
                for (i, subtype) in JOB_TYPES.iter().enumerate() {
                    if self.0 & (1 << i) != 0 {
                        out.serialize_element(subtype)?;
                    }
                }
                for (bit, subtype) in [
                    (COPY_HUMAN, "Human"),
                    (COPY_VILLAIN, "Villain"),
                    (COPY_MERCENARY, "Mercenary"),
                    (ADDED_SOLDIER, "Soldier"),
                    (ADDED_GOBLIN, "Goblin"),
                    (ADDED_BEAR, "Bear"),
                    (ADDED_FOOD, "Food"),
                    (ADDED_SPIRIT, "Spirit"),
                    (ADDED_CONSTRUCT, "Construct"),
                    (ADDED_EQUIPMENT, "Equipment"),
                    (ADDED_TREASURE, "Treasure"),
                ] {
                    if self.0 & bit != 0 {
                        out.serialize_element(subtype)?;
                    }
                }
                if self.1 {
                    out.serialize_element("Cave")?;
                }
                out.end()
            }
        }
        let mut out = serializer.serialize_struct(
            "GrantedSubtypes",
            6 + usize::from(self.card_types.is_some())
                + usize::from(self.basic_land.is_some())
                + usize::from(self.added_basic_lands != 0)
                + usize::from(self.effective_basic_lands.is_some())
                + usize::from(self.suppress_printed_land)
                + usize::from(self.flag(SUPPRESS_PRINTED_ARTIFACT))
                + usize::from(!self.additional_creature_types.is_empty()),
        )?;
        out.serialize_field("suppress_printed", &self.flag(SUPPRESS_PRINTED))?;
        out.serialize_field("land_only", &self.flag(LAND_ONLY))?;
        out.serialize_field("all_creature_types", &self.flag(ALL_CREATURE_TYPES))?;
        out.serialize_field("chosen", &self.chosen)?;
        out.serialize_field("added", &Added(self.added & ADDED_MASK, self.added_cave))?;
        out.serialize_field("creature_types", &self.creature_types)?;
        if !self.additional_creature_types.is_empty() {
            out.serialize_field("additional_creature_types", &self.additional_creature_types)?;
        }
        if let Some(bits) = self.effective_basic_lands {
            let kinds: Vec<_> = crate::cards::BasicLandType::ALL
                .into_iter()
                .filter(|kind| bits & (1 << kind.index()) != 0)
                .collect();
            out.serialize_field("effective_basic_lands", &kinds)?;
        }
        if self.suppress_printed_land {
            out.serialize_field("suppress_printed_land", &true)?;
        }
        if self.flag(SUPPRESS_PRINTED_ARTIFACT) {
            out.serialize_field("suppress_printed_artifact", &true)?;
        }
        if self.added_basic_lands != 0 {
            let kinds: Vec<_> = crate::cards::BasicLandType::ALL
                .into_iter()
                .filter(|kind| self.added_basic_lands & (1 << kind.index()) != 0)
                .collect();
            out.serialize_field("added_basic_lands", &kinds)?;
        }
        if let Some(kind) = self.basic_land {
            out.serialize_field("basic_land", &kind)?;
        }
        if let Some(kinds) = self.card_types {
            out.serialize_field("card_types", &kinds)?;
        }
        out.end()
    }
}

// Complete pinned XMage SubType.PlaneswalkerType inventory, revision
// 492369c7844b8d76479f0393ff9eb6b0b0cf30d7, SubType.java SHA256
// be30aa51ffe000d8bfffe5038b5df426d285a78fdc07061029c9d8d7a01892d2.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) const PLANESWALKER_TYPES: &[&str] = &[
    "Ajani",
    "Aminatou",
    "Angrath",
    "Arlinn",
    "Arzakon",
    "Ashiok",
    "Aurra",
    "Bahamut",
    "Basri",
    "Bolas",
    "Calix",
    "Chandra",
    "Comet",
    "Dack",
    "Dakkon",
    "Daretti",
    "Davriel",
    "Dellian",
    "Dihada",
    "Domri",
    "Dooku",
    "Dovin",
    "Dyfed",
    "Ellywick",
    "Elminster",
    "Elspeth",
    "Estrid",
    "Feroz",
    "Freyalise",
    "Garruk",
    "Gideon",
    "Greensleeves",
    "Grist",
    "Guff",
    "Huatli",
    "Inzerva",
    "Jace",
    "Jared",
    "Jaya",
    "Jeska",
    "Kaito",
    "Karn",
    "Kasmina",
    "Kaya",
    "Kiora",
    "Koth",
    "Liliana",
    "Lolth",
    "Luke",
    "Lukka",
    "Minsc",
    "Mordenkainen",
    "Nahiri",
    "Narset",
    "Niko",
    "Nissa",
    "Nixilis",
    "Obi-Wan",
    "Oko",
    "Quintorius",
    "Ral",
    "Rey",
    "Rowan",
    "Saheeli",
    "Samut",
    "Sarkhan",
    "Serra",
    "Sidious",
    "Sivitri",
    "Snoke",
    "Sorin",
    "Szat",
    "Tamiyo",
    "Tasha",
    "Teferi",
    "Teyo",
    "Tezzeret",
    "Tibalt",
    "Tyvar",
    "Ugin",
    "Urza",
    "Venser",
    "Vivien",
    "Vraska",
    "Vronos",
    "Will",
    "Windgrace",
    "Worzel",
    "Wrenn",
    "Xenagos",
    "Yanggu",
    "Yanling",
    "Yoda",
    "Zariel",
];

// CR205.3g-k,m,q. Creature names come from the pinned complete type inventory.
// Any remaining printed subtype in this permanent pool is a planeswalker type.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn subtype_card_types(subtype: &str) -> u16 {
    if crate::cards::is_creature_type(subtype) {
        return 1 | 128;
    }
    if LAND_TYPES.contains(&subtype) {
        return 2;
    }
    if [
        "Attraction",
        "Blood",
        "Bobblehead",
        "Book",
        "Clue",
        "Contraption",
        "Equipment",
        "Food",
        "Fortification",
        "Gold",
        "Heartwood",
        "Incubator",
        "Infinity",
        "Junk",
        "Lander",
        "Map",
        "Mutagen",
        "Powerstone",
        "Spacecraft",
        "Stone",
        "Treasure",
        "Vehicle",
        "Vibranium",
    ]
    .contains(&subtype)
    {
        return 16;
    }
    if [
        "Aura",
        "Background",
        "Cartouche",
        "Case",
        "Class",
        "Curse",
        "Plan",
        "Role",
        "Room",
        "Rune",
        "Saga",
        "Shard",
        "Shrine",
    ]
    .contains(&subtype)
    {
        return 32;
    }
    if ["Adventure", "Arcane", "Lesson", "Omen", "Trap"].contains(&subtype) {
        return 4 | 8;
    }
    if subtype == "Siege" {
        return 256;
    }
    64
}

/// A resolution-created replacement, lasting for this exact incarnation only.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureTypesEffectView {
    pub recipient: ObjectRef,
    pub source: ObjectRef,
    pub types: &'static [&'static str],
    pub additional: bool,
    pub timestamp: u64,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn cast_added_types(state: Option<&roles::RoleState>, spell: ObjectRef) -> &'static [&'static str] {
    state
        .and_then(|roles| {
            roles
                .creature_types
                .iter()
                .find(|row| row.recipient == spell && row.additional)
        })
        .map_or(&[], |row| row.types)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn add_cast_creature_types(&mut self, types: &'static [&'static str]) {
        self.source
            .as_mut()
            .expect("typed spell")
            .facts
            .granted_subtypes
            .add_creature_types(types);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn continue_cast_type_effects(
    state: &mut roles::RoleState,
    spell: ObjectRef,
    after: ObjectRef,
    destination: Zone,
) {
    state.creature_types.retain_mut(|row| {
        if row.recipient != spell {
            return true;
        }
        if destination != Zone::Battlefield {
            return false;
        }
        row.recipient = after;
        true
    });
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn cast_added_creature_types(&self, spell: ObjectRef) -> &'static [&'static str] {
        cast_added_types(self.state.roles.as_deref(), spell)
    }
    pub(super) fn apply_paid_graveyard_spell_types(
        &mut self,
        item: &mut SpellView,
        permission: PaidGraveyardCastPermission,
    ) {
        if permission.additional_types.is_empty() {
            return;
        }
        item.add_cast_creature_types(permission.additional_types);
        let timestamp = self.next_role_timestamp();
        self.state
            .roles
            .as_mut()
            .unwrap()
            .creature_types
            .push(CreatureTypesEffectView {
                recipient: item.source.expect("typed spell").object,
                source: permission.source,
                types: permission.additional_types,
                additional: true,
                timestamp,
            });
    }
    pub(super) fn continue_cast_creature_types(
        &mut self,
        spell: ObjectRef,
        after: ObjectRef,
        destination: Zone,
    ) {
        if let Some(state) = &mut self.state.roles {
            continue_cast_type_effects(state, spell, after, destination);
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ObjectView {
    pub fn has_subtype(&self, subtype: &str) -> bool {
        self.granted_subtypes.has(self.card, subtype)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl DamageFacts {
    pub fn has_subtype(self, subtype: &str) -> bool {
        self.granted_subtypes.has(self.card, subtype)
    }
}
/// An exact-incarnation, resolution-created additive land type effect.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct BasicLandTypesEffectView {
    pub recipient: ObjectRef,
    pub source: ObjectRef,
    pub kind: crate::cards::BasicLandType,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub while_counter: Option<CounterKind>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub until_cleanup: bool,
}

impl BasicLandTypesEffectView {
    pub(super) fn expires_with(self, object: ObjectRef, counters: &Counters) -> bool {
        self.recipient == object
            && self
                .while_counter
                .is_some_and(|kind| counters.get(kind) == 0)
    }
}

/// Add the live layer-four grant to an actual or proposed creature spell.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn spell_subtypes<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    controller: Player,
    definition: Card,
    mut subtypes: GrantedSubtypes,
) -> GrantedSubtypes {
    if definition.is_creature() {
        add_chosen_creature_subtypes(world, controller, &mut subtypes);
    }
    subtypes
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn add_chosen_creature_subtypes<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    mask: &mut GrantedSubtypes,
) {
    // All providers need a choice. Inspect the sparse choices instead of all
    // static sources for each card in a private search or look.
    for row in world.entry_values() {
        let EntryChoiceValue::CreatureType(typ) = row.value else {
            continue;
        };
        let source = row.object.card;
        let provider = world.object(source);
        if provider.zone == Zone::Battlefield
            && provider.controller == player
            && world.reference(source) == row.object
            && world.ability_origin(source) == row.origin
            && provider.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::ChosenCreatureSubtypeAcrossZones
                    )
                }) && entry_modes::group_enabled(world, row.object, group)
            })
        {
            // Layer-six ability removal follows this type-changing effect.
            mask.additional_creature_types.insert(typ);
        }
    }
}

/// Copy and cast-receipt subtypes, before live layer-four grants.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn initial_subtypes<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> GrantedSubtypes {
    let mut mask = GrantedSubtypes::default();
    let recipe = world.recipe(id);
    mask.apply_copy_subtypes(Some(recipe));
    if recipe
        .overrides
        .is_some_and(|o| o.keywords.contains(Keywords::CHANGELING))
        && world.kinds(id) & CREATURE_OR_KINDRED != 0
    {
        // A noncreature, nonkindred copy cannot have creature subtypes (CR205.3d).
        mask.set_flag(ALL_CREATURE_TYPES, true);
    }
    if world.object(id).zone == Zone::Stack {
        mask.add_creature_types(cast_added_types(world.roles(), world.reference(id)));
    }
    mask
}

/// Earlier subtype layers over actual or projected source registries.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn in_world<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> GrantedSubtypes {
    let target = world.object(id);
    if target.zone == Zone::Exile && world.game().exiled_face_down(world.reference(id)) {
        return GrantedSubtypes {
            added: SUPPRESS_PRINTED,
            ..GrantedSubtypes::default()
        };
    }
    let mut mask = initial_subtypes(world, id);
    if target.zone != Zone::Battlefield {
        let player = match target.zone {
            Zone::Stack => Some(target.controller),
            Zone::Outside => None,
            _ => target.is_card().then_some(target.owner),
        };
        if let Some(player) = player
            && !world.entry_values().is_empty()
            && world.kinds(id) & 1 != 0
        {
            add_chosen_creature_subtypes(world, player, &mut mask);
        }
        return mask;
    }
    let lookup = |source| Some(world.object(source));
    let counter_count = |source, kind| world.counter_count(source, kind);
    let kinds = |id, _: &Object| world.kinds(id);
    let mut changes = characteristic_layers::subtype_changes(world, id);
    let reference = world.reference(id);
    if let Some(state) = world.roles() {
        changes.extend(
            state
                .creature_types
                .iter()
                .filter(|row| row.recipient == reference)
                .map(|row| {
                    (
                        row.timestamp,
                        if row.additional {
                            characteristic_layers::SubtypeChange::Add(row.types)
                        } else {
                            characteristic_layers::SubtypeChange::Named(row.types, false)
                        },
                    )
                }),
        );
    }
    if let Some(state) = world.roles() {
        changes.extend(
            state
                .basic_land_types
                .iter()
                .filter(|row| row.recipient == reference)
                .map(|row| {
                    (
                        row.timestamp,
                        characteristic_layers::SubtypeChange::AddBasicLand(row.kind),
                    )
                }),
        );
    }
    // This battlefield-only conditional effect starts in layer four. It is
    // neither a changeling CDA nor a layer-six ability grant; later removal of
    // its printed ability does not undo a type effect that already started.
    if kinds(id, target) & CREATURE_OR_KINDRED != 0 {
        for group in target.card.static_ability_groups() {
            for fragment in group.fragments {
                if let crate::cards::StaticAbilityFragment::SourceGraveyardAllTypes {
                    kind,
                    minimum,
                } = *fragment
                    && activations::graveyard_cards_at_least(
                        world,
                        target.controller,
                        kind,
                        minimum,
                    )
                {
                    let timestamp = world
                        .roles()
                        .and_then(|state| {
                            state
                                .ability_timestamps
                                .iter()
                                .find(|row| row.source == reference)
                        })
                        .expect("registered conditional type source")
                        .timestamp;
                    changes.push((timestamp, characteristic_layers::SubtypeChange::All));
                }
            }
        }
    }
    // The chosen type starts in layer four and retains the source's timestamp.
    if kinds(id, target) & 2 != 0
        && target.card.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::ChosenBasicLandType
                )
            })
        })
        && let Some(EntryChoiceValue::BasicLandType(kind)) = world.entry_value(reference)
    {
        let timestamp = world
            .roles()
            .and_then(|roles| {
                roles
                    .ability_timestamps
                    .iter()
                    .find(|row| row.source == reference)
            })
            .expect("registered type source timestamp")
            .timestamp;
        changes.push((
            timestamp,
            characteristic_layers::SubtypeChange::BasicLand(kind),
        ));
    }
    let source_time = |source: ObjectRef| {
        world
            .roles()
            .and_then(|state| {
                state
                    .ability_timestamps
                    .iter()
                    .find(|row| row.source == source)
            })
            .map(|row| row.timestamp)
            .expect("registered layer-four provider timestamp")
    };
    if kinds(id, target) & CREATURE_OR_KINDRED != 0 {
        for &source in world.static_sources() {
            let origin = world.reference(source);
            if world.attached(origin, id)
                && world
                    .object(source)
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(
                                fragment,
                                crate::cards::StaticAbilityFragment::AttachedAllCreatureTypes
                            )
                        })
                    })
            {
                changes.push((
                    source_time(origin),
                    characteristic_layers::SubtypeChange::All,
                ));
            }
        }
    }
    if kinds(id, target) & 1 != 0 {
        let mut dependency_time = None;
        if target.card.static_ability_groups().iter().any(|g| {
            g.fragments
                .iter()
                .any(|f| matches!(f, crate::cards::StaticAbilityFragment::ChosenCreatureType))
        }) && let Some(EntryChoiceValue::CreatureType(typ)) = world.entry_value(reference)
        {
            changes.push((
                source_time(reference).max(*dependency_time.get_or_insert_with(|| {
                    characteristic_layers::creature_addition_timestamp(world, id).unwrap_or(0)
                })),
                characteristic_layers::SubtypeChange::Chosen(typ),
            ));
        }
        for &source in world.static_sources() {
            let provider = world.object(source);
            if provider.zone != Zone::Battlefield || provider.controller != target.controller {
                continue;
            }
            let origin = world.reference(source);
            for group in provider.card.static_ability_groups() {
                if !entry_modes::group_enabled(world, origin, group) {
                    continue;
                }
                for fragment in group.fragments {
                    use crate::cards::{CreatureSubtypeGrant, StaticAbilityFragment};
                    let (grant, required_subtype) = match *fragment {
                        StaticAbilityFragment::ControlledCreatureSubtype {
                            grant,
                            required_subtype,
                        } => (grant, required_subtype),
                        StaticAbilityFragment::ChosenCreatureSubtypeAcrossZones => {
                            (CreatureSubtypeGrant::Chosen, None)
                        }
                        StaticAbilityFragment::ControlledForeignOwnedCreatureSubtype(subtype)
                            if target.owner != target.controller =>
                        {
                            (CreatureSubtypeGrant::Named(subtype), None)
                        }
                        _ => continue,
                    };
                    let typ = match grant {
                        CreatureSubtypeGrant::Named(name) => Some(
                            crate::cards::CreatureType::from_name(name)
                                .expect("authenticated immutable creature subtype grant"),
                        ),
                        CreatureSubtypeGrant::Chosen => match world.entry_value(origin) {
                            Some(EntryChoiceValue::CreatureType(typ)) => Some(typ),
                            _ => None,
                        },
                    };
                    if let Some(typ) = typ {
                        changes.push((
                            source_time(origin).max(*dependency_time.get_or_insert_with(|| {
                                characteristic_layers::creature_addition_timestamp(world, id)
                                    .unwrap_or(0)
                            })),
                            characteristic_layers::SubtypeChange::Granted(typ, required_subtype),
                        ));
                    }
                }
            }
        }
    }
    changes.sort_by_key(|row| row.0);
    let replacement_time = changes
        .iter()
        .rev()
        .find(|(_, change)| {
            !matches!(
                change,
                characteristic_layers::SubtypeChange::Add(_)
                    | characteristic_layers::SubtypeChange::Chosen(_)
                    | characteristic_layers::SubtypeChange::Granted(..)
                    | characteristic_layers::SubtypeChange::BasicLand(_)
                    | characteristic_layers::SubtypeChange::AddBasicLand(_)
                    | characteristic_layers::SubtypeChange::EquipmentOnly
                    | characteristic_layers::SubtypeChange::AddFood
            )
        })
        .map(|row| row.0);
    for (_, change) in changes {
        match change {
            characteristic_layers::SubtypeChange::Add(types) => {
                if kinds(id, target) & CREATURE_OR_KINDRED != 0 {
                    for typ in types {
                        mask.insert(typ);
                    }
                }
            }
            characteristic_layers::SubtypeChange::RemoveCreatureTypes => {
                mask.replace_creature_types(&[]);
                mask.set_flag(ALL_CREATURE_TYPES, false);
            }
            characteristic_layers::SubtypeChange::Chosen(typ) => {
                mask.chosen = Some(typ);
            }
            characteristic_layers::SubtypeChange::Granted(typ, required_subtype) => {
                if required_subtype.is_none_or(|subtype| mask.has(target.card, subtype)) {
                    mask.additional_creature_types.insert(typ);
                }
            }
            characteristic_layers::SubtypeChange::BasicLand(kind) => {
                mask.basic_land = Some(kind);
                mask.added_basic_lands = 0;
            }
            characteristic_layers::SubtypeChange::AddBasicLand(kind) => {
                if kinds(id, target) & 2 != 0 {
                    mask.added_basic_lands |= 1 << kind.index();
                }
            }
            characteristic_layers::SubtypeChange::FoodOnly => {
                mask = GrantedSubtypes::food_only();
            }
            characteristic_layers::SubtypeChange::FoodArtifact => {
                mask.replace_creature_types(&[]);
                mask.set_flag(ALL_CREATURE_TYPES | LAND_ONLY, false);
                mask.card_types = Some(kinds(id, target));
                mask.set_flag(ADDED_FOOD, true);
            }
            characteristic_layers::SubtypeChange::AddFood => {
                mask.set_flag(ADDED_FOOD, true);
            }
            characteristic_layers::SubtypeChange::TreasureOnly => {
                mask = GrantedSubtypes::treasure_only();
            }
            characteristic_layers::SubtypeChange::EquipmentOnly => {
                mask.set_flag(ADDED_ARTIFACT_SUBTYPES, false);
                mask.set_flag(ADDED_EQUIPMENT, true);
                mask.set_flag(SUPPRESS_PRINTED_ARTIFACT, true);
            }
            characteristic_layers::SubtypeChange::LandOnly => {
                mask.set_flag(ADDED_FOOD | ADDED_TREASURE, false);
                mask.replace_creature_types(&[]);
                mask.set_flag(ALL_CREATURE_TYPES, false);
                mask.set_flag(LAND_ONLY, true);
            }
            characteristic_layers::SubtypeChange::All => {
                mask.replace_creature_types(&[]);
                mask.set_flag(ALL_CREATURE_TYPES, true);
            }
            characteristic_layers::SubtypeChange::Named(types, all) => {
                mask.replace_creature_types(types);
                mask.set_flag(ALL_CREATURE_TYPES, false);
                if all {
                    mask.set_flag(ADDED_ARTIFACT_SUBTYPES, false);
                    mask.added_cave = false;
                    mask.set_flag(SUPPRESS_PRINTED, true);
                    mask.set_flag(LAND_ONLY, false);
                }
            }
        }
    }
    let land_timeline = basic_land_timeline(world, id);
    if land_timeline.has_events {
        mask.effective_basic_lands = Some(land_timeline.final_types);
        mask.suppress_printed_land = land_timeline.lost_original_types;
        mask.added_basic_lands &= land_timeline.final_types;
        mask.added_cave &= !land_timeline.lost_original_types;
        if mask
            .basic_land
            .is_some_and(|kind| land_timeline.final_types & (1 << kind.index()) == 0)
        {
            mask.basic_land = None;
        }
    }
    // CR 205.1a: a card-type replacement suppresses correlated subtypes
    // only while the corresponding card type is absent.
    if world.roles().is_some_and(|roles| {
        roles.animations.iter().any(|row| {
            row.object == reference
                && (row.creature_only || row.artifact_noncreature || row.enchantment_noncreature)
        })
    }) {
        mask.card_types = Some(kinds(id, target));
    }
    if mask.added_cave && kinds(id, target) & 2 == 0 {
        mask.added_cave = false;
    }
    if kinds(id, target) & 16 == 0 {
        mask.set_flag(ADDED_ARTIFACT_SUBTYPES, false);
    } else if world.roles().is_some_and(|roles| {
        roles.characteristic_sources.iter().any(|&source| {
            let provider = world.object(source.card);
            provider.zone == Zone::Battlefield
                && world.reference(source.card) == source
                && provider.controller == target.controller
                && provider.card.static_ability_groups().iter().any(|group| {
                    entry_modes::group_enabled(world, source, group)
                        && group.fragments.iter().any(|fragment| {
                            matches!(fragment, StaticAbilityFragment::ControlledArtifactFood(_))
                        })
                })
        })
    }) {
        // The subtype effect depends on the earlier-layer artifact set. Losing
        // printed abilities in layer six does not undo its layer-four work.
        mask.set_flag(ADDED_FOOD, true);
    }
    if kinds(id, target) & 1 == 0 {
        return mask;
    }
    let newer = |source| replacement_time.is_none_or(|time| source_time(source) > time);
    for group in target.card.static_ability_groups() {
        for fragment in group.fragments {
            if let crate::cards::StaticAbilityFragment::SourceCounterSubtype {
                kind,
                minimum,
                subtype,
                ..
            } = *fragment
                && counter_count(id, kind) >= minimum
                && newer(reference)
            {
                // The effect starts in layer four, before ability removal.
                mask.insert(subtype);
            }
        }
    }
    for &source in world.attachment_sources() {
        let Some(o) = lookup(source) else {
            continue;
        };
        if o.zone == Zone::Battlefield
            && kinds(source, o) & 1 == 0
            && o.attached_to == Some(reference)
            && let Some(subtype) = o.card.attached_subtype()
            && newer(ObjectRef {
                card: source,
                generation: o.generation,
            })
        {
            mask.insert(subtype);
        }
    }
    mask
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn granted_subtypes(&self, id: CardId) -> GrantedSubtypes {
        self.state
            .objects
            .get(id as usize)
            .map_or_else(GrantedSubtypes::default, |_| {
                super::ability_world::CharacteristicWorld::subtypes(self, id)
            })
    }
    pub(super) fn has_subtype(&self, id: CardId, subtype: &str) -> bool {
        self.state.objects.get(id as usize).is_some_and(|o| {
            let card = if o.zone == Zone::Stack {
                self.effective_spell_card(id)
            } else {
                o.card
            };
            self.granted_subtypes(id).has(card, subtype)
        })
    }
}

/// Layer-one land types, before later type-setting and additive effects.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn native_basic_lands<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> u8 {
    let mut mask = GrantedSubtypes::default();
    mask.apply_copy_subtypes(Some(world.recipe(id)));
    crate::cards::BasicLandType::ALL
        .into_iter()
        .filter(|kind| mask.has(world.definition(id), kind.name()))
        .fold(0, |bits, kind| bits | (1 << kind.index()))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BasicLandAbilityChange {
    Remove,
    Add(crate::cards::BasicLandType),
    Set(crate::cards::BasicLandType),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) struct BasicLandTimeline {
    pub initial_types: u8,
    pub final_types: u8,
    pub lost_original_types: bool,
    pub has_events: bool,
    pub ability_changes: Vec<(u64, BasicLandAbilityChange)>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn nonbasic_land_type_setters<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
) -> impl Iterator<Item = (u64, crate::cards::BasicLandType)> {
    world.roles().into_iter().flat_map(move |roles| {
        roles
            .characteristic_sources
            .iter()
            .filter_map(move |&source| {
                let object = world.object(source.card);
                if object.zone != Zone::Battlefield || world.reference(source.card) != source {
                    return None;
                }
                let kind = object
                    .card
                    .static_ability_groups()
                    .iter()
                    .filter(|group| entry_modes::group_enabled(world, source, group))
                    .flat_map(|group| group.fragments)
                    .find_map(|fragment| match *fragment {
                        StaticAbilityFragment::NonbasicLandTypeWhileCounter { kind, counter }
                            if world.counter_count(source.card, counter) != 0 =>
                        {
                            Some(kind)
                        }
                        _ => None,
                    })?;
                let timestamp = roles
                    .ability_timestamps
                    .iter()
                    .find(|row| row.source == source)?
                    .timestamp;
                Some((timestamp, kind))
            })
    })
}

pub(super) fn is_nonbasic_land<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> bool {
    !world.object(id).card.is_basic_land() && world.kinds(id) & 2 != 0
}

/// Run all supported layer-four card-type and basic-land-type changes in
/// timestamp order, recording precisely when intrinsic mana is lost or gained.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn basic_land_timeline<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> BasicLandTimeline {
    #[derive(Clone, Copy)]
    enum Event {
        Land(bool),
        Add(crate::cards::BasicLandType),
        AddAll,
        Set(crate::cards::BasicLandType),
        Clear,
    }
    let object = world.object(id);
    let reference = world.reference(id);
    let printed_land = || world.recipe(id).kinds() & 2 != 0;
    let kind_events = characteristic_layers::land_kind_events(world, id);
    let land_since = kind_events.last().map_or_else(
        || printed_land().then_some(0),
        |&(time, land)| land.then_some(time),
    );
    let mut land_setters = nonbasic_land_type_setters(world).peekable();
    let has_land_setters = land_since.is_some() && land_setters.peek().is_some();
    let removes_printed = has_land_setters && !object.card.is_basic_land();
    // A dependent effect must wait for land status. Effects waiting on the
    // same transition still order among themselves by their own timestamps.
    let land_effect_order = |time: u64| (time.max(land_since.unwrap_or(0)), time);
    let mut events: Vec<((u64, u64), Event)> = kind_events
        .iter()
        .map(|&(time, land)| ((time, 0), Event::Land(land)))
        .collect();
    events.extend(
        world
            .roles()
            .into_iter()
            .flat_map(|state| &state.basic_land_types)
            .filter(|row| row.recipient == reference)
            .map(|row| (land_effect_order(row.timestamp), Event::Add(row.kind))),
    );
    events.extend(
        world
            .roles()
            .into_iter()
            .flat_map(|state| &state.source_linked_effects)
            .filter(|row| {
                row.recipient == reference
                    && row.kind == SourceLinkedEffectKind::BlankLandWhileBlighted
                    && source_linked_effects::active(world, row)
            })
            .map(|row| ((row.timestamp, row.timestamp), Event::Clear)),
    );
    // An external basic-type setter removes this native type-setting ability
    // before it can apply. Keep the stored choice for when the setter leaves.
    if !removes_printed
        && object.card.static_ability_groups().iter().any(|group| {
            group
                .fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::ChosenBasicLandType))
        })
        && let Some(EntryChoiceValue::BasicLandType(kind)) = world.entry_value(reference)
        && let Some(time) = world
            .roles()
            .and_then(|state| {
                state
                    .ability_timestamps
                    .iter()
                    .find(|row| row.source == reference)
            })
            .map(|row| row.timestamp)
    {
        events.push((land_effect_order(time), Event::Set(kind)));
    }
    if land_since.is_some() {
        events.extend(
            characteristic_layers::controlled_characteristic_sources(world, id, |fragment| {
                matches!(
                    fragment,
                    StaticAbilityFragment::ControlledLandsAllBasicTypes
                )
            })
            .filter(|(source, _)| !has_land_setters || !is_nonbasic_land(world, source.card))
            .map(|(_, time)| (land_effect_order(time), Event::AddAll)),
        );
        if removes_printed {
            events.extend(
                land_setters.map(|(time, kind)| (land_effect_order(time), Event::Set(kind))),
            );
        }
    }
    if events.is_empty() {
        return BasicLandTimeline {
            initial_types: 0,
            final_types: 0,
            lost_original_types: false,
            has_events: false,
            ability_changes: Vec::new(),
        };
    }
    events.sort_by_key(|event| event.0);
    let mut is_land = printed_land();
    let relevant = is_land
        || events
            .iter()
            .any(|(_, event)| !matches!(event, Event::Land(false)));
    let initial_types = if is_land {
        native_basic_lands(world, id)
    } else {
        0
    };
    let mut present = initial_types;
    let mut lost_original_types = !is_land;
    let mut ability_changes = Vec::new();
    for ((time, _), event) in &events {
        match *event {
            Event::Land(false) => {
                if is_land {
                    present = 0;
                    is_land = false;
                    lost_original_types = true;
                    ability_changes.push((*time, BasicLandAbilityChange::Remove));
                }
            }
            Event::Land(true) => {
                is_land = true;
            }
            Event::Add(kind) if is_land => {
                let bit = 1 << kind.index();
                if present & bit == 0 {
                    present |= bit;
                    ability_changes.push((*time, BasicLandAbilityChange::Add(kind)));
                }
            }
            Event::AddAll if is_land => {
                for kind in crate::cards::BasicLandType::ALL {
                    let bit = 1 << kind.index();
                    if present & bit == 0 {
                        present |= bit;
                        ability_changes.push((*time, BasicLandAbilityChange::Add(kind)));
                    }
                }
            }
            Event::Set(kind) if is_land => {
                present = 1 << kind.index();
                lost_original_types = true;
                ability_changes.push((*time, BasicLandAbilityChange::Set(kind)));
            }
            Event::Clear => {
                present = 0;
                lost_original_types = true;
                // The same timestamp's layer-six removal clears every ability.
            }
            Event::Add(_) | Event::AddAll | Event::Set(_) => {}
        }
    }
    BasicLandTimeline {
        initial_types,
        final_types: present,
        lost_original_types,
        has_events: relevant && !events.is_empty(),
        ability_changes,
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn add_creature_types(
        &mut self,
        item: &SpellView,
        subject: Subject,
        types: &'static [&'static str],
    ) {
        let Some(source) = item.source else {
            return;
        };
        assert!(types.iter().all(|t| crate::cards::is_creature_type(t)));
        let recipients: Vec<_> = self
            .keyword_subjects(item, subject)
            .into_iter()
            .filter(|&id| self.on_battlefield(id) && self.is_creature(id))
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        for recipient in recipients {
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .creature_types
                .push(CreatureTypesEffectView {
                    recipient,
                    source: source.object,
                    types,
                    additional: true,
                    timestamp,
                });
        }
    }
    pub(super) fn add_basic_land_type(
        &mut self,
        item: &SpellView,
        subject: Subject,
        kind: crate::cards::BasicLandType,
        while_counter: Option<CounterKind>,
    ) {
        let Some(source) = item.source else {
            return;
        };
        let recipients: Vec<_> = self
            .keyword_subjects(item, subject)
            .into_iter()
            .filter(|&id| {
                self.on_battlefield(id)
                    && self.current_kinds(id) & 2 != 0
                    && while_counter.is_none_or(|kind| self.counter_count(id, kind) > 0)
            })
            .map(|id| self.object_ref(id))
            .collect();
        self.record_basic_land_types(source.object, &recipients, &[kind], while_counter, false);
    }

    pub(super) fn grant_all_basic_land_types_until_cleanup(&mut self, item: &SpellView) {
        let Some(source) = item.source else { return };
        let recipients: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                (object.zone == Zone::Battlefield
                    && object.controller == item.controller
                    && self.current_kinds(id) & 2 != 0)
                    .then(|| self.object_ref(id))
            })
            .collect();
        self.record_basic_land_types(
            source.object,
            &recipients,
            &crate::cards::BasicLandType::ALL,
            None,
            true,
        );
    }

    fn record_basic_land_types(
        &mut self,
        source: ObjectRef,
        recipients: &[ObjectRef],
        kinds: &[crate::cards::BasicLandType],
        while_counter: Option<CounterKind>,
        until_cleanup: bool,
    ) {
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let effects = &mut self
            .state
            .roles
            .get_or_insert_with(Default::default)
            .basic_land_types;
        for &recipient in recipients {
            effects.extend(kinds.iter().map(|&kind| BasicLandTypesEffectView {
                recipient,
                source,
                kind,
                timestamp,
                while_counter,
                until_cleanup,
            }));
        }
    }

    pub(super) fn expire_basic_land_types(&mut self) -> bool {
        self.state.roles.as_mut().is_some_and(|roles| {
            let count = roles.basic_land_types.len();
            roles.basic_land_types.retain(|row| !row.until_cleanup);
            count != roles.basic_land_types.len()
        })
    }

    pub(super) fn set_creature_types(
        &mut self,
        item: &SpellView,
        subject: Subject,
        types: &'static [&'static str],
    ) {
        let Some(source) = item.source else {
            return;
        };
        assert!(
            types
                .iter()
                .all(|subtype| crate::cards::is_creature_type(subtype)),
            "typed creature type replacement"
        );
        let recipients: Vec<_> = self
            .keyword_subjects(item, subject)
            .into_iter()
            .filter(|&id| self.on_battlefield(id) && self.is_creature(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        for id in recipients {
            let recipient = self.object_ref(id);
            let state = self.state.roles.get_or_insert_with(Default::default);
            // A replacement supersedes older replacements for this recipient.
            state
                .creature_types
                .retain(|row| row.recipient != recipient);
            state.creature_types.push(CreatureTypesEffectView {
                recipient,
                source: source.object,
                types,
                additional: false,
                timestamp,
            });
        }
    }
    pub(super) fn forget_creature_types(&mut self, recipient: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state
                .creature_types
                .retain(|row| row.recipient != recipient);
            state
                .basic_land_types
                .retain(|row| row.recipient != recipient);
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn entry_choice_subtypes(m: &entries::EntryMove) -> GrantedSubtypes {
        Self::entry_subtypes(m, true)
    }

    /// CR 614.12: only return effects that apply as the permanent enters are present.
    pub(super) fn entry_replacement_subtypes(m: &entries::EntryMove) -> GrantedSubtypes {
        Self::entry_subtypes(m, false)
    }

    fn entry_subtypes(m: &entries::EntryMove, include_return_effect: bool) -> GrantedSubtypes {
        if m.food_return.is_some() {
            return GrantedSubtypes::food_only();
        }
        if m.as_entry_treasure_source().is_some() {
            return GrantedSubtypes::treasure_only();
        }
        let mut result = GrantedSubtypes::default();
        result.apply_copy_subtypes(m.before.copy_recipe);
        if m.before
            .copy_recipe
            .and_then(|recipe| recipe.overrides)
            .is_some_and(|o| o.keywords.contains(Keywords::CHANGELING))
        {
            result.set_flag(ALL_CREATURE_TYPES, true);
        }
        if m.before.card.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|fragment| {
                matches!(
                    fragment,
                    crate::cards::StaticAbilityFragment::ChosenBasicLandType
                )
            })
        }) && let Some(EntryChoiceValue::BasicLandType(kind)) = m.choice
            && m.entry_kinds() & 2 != 0
        {
            result.basic_land = Some(kind);
            result.added_cave = false;
        }
        if m.before.card.static_ability_groups().iter().any(|g| {
            g.fragments
                .iter()
                .any(|f| matches!(f, crate::cards::StaticAbilityFragment::ChosenCreatureType))
        }) && let Some(EntryChoiceValue::CreatureType(typ)) = m.choice
        {
            result.chosen = Some(typ);
        }
        if m.ghost_vacuum_after.is_some() && m.before.kinds & CREATURE_OR_KINDRED != 0 {
            result.insert("Spirit");
        }
        if m.as_entry_skeleton_source().is_some() && m.before.kinds & CREATURE_OR_KINDRED != 0 {
            result.insert("Skeleton");
        }
        if include_return_effect && m.before.kinds & CREATURE_OR_KINDRED != 0 {
            match m.return_modification.map(|(kind, _)| kind) {
                Some(ReturnKind::Angel) => result.insert("Angel"),
                Some(ReturnKind::Demon) => result.insert("Demon"),
                Some(ReturnKind::HeroFinality) => result.insert("Hero"),
                _ => {}
            }
        }
        if m.entry_kinds() & CREATURE_OR_KINDRED != 0 {
            result.add_creature_types(m.cast_added_creature_types);
        }
        if m.artifact_return || m.enchantment_return {
            result.card_types = Some(m.entry_kinds());
        }
        result
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn grant_amass_goblin_type(&mut self, source: ObjectRef, recipient: ObjectRef) {
        if self.object_ref(recipient.card) != recipient || !self.on_battlefield(recipient.card) {
            return;
        }
        if !self.has_subtype(recipient.card, "Goblin") {
            let timestamp = self.next_role_timestamp();
            self.state
                .roles
                .get_or_insert_with(Default::default)
                .creature_types
                .push(CreatureTypesEffectView {
                    recipient,
                    source,
                    types: &["Goblin"],
                    additional: true,
                    timestamp,
                });
        }
    }
}
