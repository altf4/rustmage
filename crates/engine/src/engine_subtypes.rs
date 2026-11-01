//! Layer-four creature subtype overlays. Objects retain only their printed identity;
//! captured views carry a compact value, and speculative resource queries borrow
//! the same exact attachment test over their own object overlay.
use super::*;
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
// Seventeen named grants and three projection flags use the view's existing padding.
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
const FLAGS_MASK: u32 = SUPPRESS_PRINTED | LAND_ONLY | ALL_CREATURE_TYPES;
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
const ADDED_MASK: u32 = JOB_MASK
    | COPY_HUMAN
    | COPY_VILLAIN
    | COPY_MERCENARY
    | ADDED_SOLDIER
    | ADDED_GOBLIN
    | ADDED_BEAR
    | ADDED_FOOD
    | ADDED_SPIRIT
    | ADDED_CONSTRUCT;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
const _: () = assert!(JOB_TYPES.len() + 12 <= u32::BITS as usize);
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantedSubtypes {
    added: u32,
    chosen: Option<crate::cards::CreatureType>,
    /// A layer-four effect replaces all previously held land subtypes.
    basic_land: Option<crate::cards::BasicLandType>,
    added_basic_lands: u8,
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
        Self {
            added: SUPPRESS_PRINTED | ADDED_FOOD,
            chosen: None,
            basic_land: None,
            added_basic_lands: 0,
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
        self.basic_land.is_some_and(|kind| kind.name() == subtype)
            || (self.added_basic_lands != 0
                && crate::cards::BasicLandType::ALL.iter().any(|kind| {
                    kind.name() == subtype && self.added_basic_lands & (1 << kind.index()) != 0
                }))
            || (self.added & Self::copy_type_bit(subtype) != 0)
            || (self.flag(ALL_CREATURE_TYPES) && crate::cards::is_creature_type(subtype))
            || self.chosen.is_some_and(|t| t.name() == subtype)
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
    fn replace_creature_types(&mut self, types: &'static [&'static str]) {
        self.added &= FLAGS_MASK | ADDED_FOOD;
        self.chosen = None;
        self.creature_types = Some(types);
    }
    fn apply_copy_types(&mut self, types: Option<CopyTypes>) {
        if let Some(types) = types {
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
            _ => 0,
        }
    }
    fn insert(&mut self, subtype: &str) {
        let copy = Self::copy_type_bit(subtype);
        if copy != 0 {
            self.added |= copy;
            return;
        }
        let i = JOB_TYPES
            .iter()
            .position(|s| *s == subtype)
            .expect("authenticated immutable subtype grant");
        self.added |= 1 << i;
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Serialize for GrantedSubtypes {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Added(u32);
        impl Serialize for Added {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut out = serializer.serialize_seq(Some(self.0.count_ones() as usize))?;
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
                ] {
                    if self.0 & bit != 0 {
                        out.serialize_element(subtype)?;
                    }
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
                + usize::from(self.suppress_printed_land),
        )?;
        out.serialize_field("suppress_printed", &self.flag(SUPPRESS_PRINTED))?;
        out.serialize_field("land_only", &self.flag(LAND_ONLY))?;
        out.serialize_field("all_creature_types", &self.flag(ALL_CREATURE_TYPES))?;
        out.serialize_field("chosen", &self.chosen)?;
        out.serialize_field("added", &Added(self.added & ADDED_MASK))?;
        out.serialize_field("creature_types", &self.creature_types)?;
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
}

/// Earlier subtype layers over actual or projected source registries.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn in_world<W: super::ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> GrantedSubtypes {
    let target = world.object(id);
    let lookup = |source| Some(world.object(source));
    let counter_count = |source, kind| world.counter_count(source, kind);
    let kinds = |id, _: &Object| world.kinds(id);
    let mut changes = characteristic_layers::subtype_changes(world, id);
    let mut mask = GrantedSubtypes::default();
    let reference = ObjectRef {
        card: id,
        generation: target.generation,
    };
    mask.apply_copy_types(Some(world.recipe(id).copy_types()));
    // Copy exceptions already exist while an owned token awaits entry.
    // Later continuous type changes apply only on the battlefield.
    if target.zone != Zone::Battlefield {
        return mask;
    }
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
    changes.sort_by_key(|row| row.0);
    let replacement_time = changes
        .iter()
        .rev()
        .find(|(_, change)| {
            !matches!(
                change,
                characteristic_layers::SubtypeChange::Add(_)
                    | characteristic_layers::SubtypeChange::BasicLand(_)
                    | characteristic_layers::SubtypeChange::AddBasicLand(_)
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
            characteristic_layers::SubtypeChange::LandOnly => {
                mask.set_flag(ADDED_FOOD, false);
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
                    mask.set_flag(ADDED_FOOD, false);
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
        if mask
            .basic_land
            .is_some_and(|kind| land_timeline.final_types & (1 << kind.index()) == 0)
        {
            mask.basic_land = None;
        }
    }
    if world.roles().is_some_and(|roles| {
        roles.animations.iter().any(|row| {
            row.object == reference && (row.artifact_noncreature || row.enchantment_noncreature)
        })
    }) {
        mask.card_types = Some(kinds(id, target));
    }
    if kinds(id, target) & 16 == 0 {
        mask.set_flag(ADDED_FOOD, false);
    }
    if kinds(id, target) & 1 == 0 {
        return mask;
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
    let newer = |source| replacement_time.is_none_or(|time| source_time(source) > time);
    if !world.entry_values().is_empty()
        && target.card.static_ability_groups().iter().any(|g| {
            g.fragments
                .iter()
                .any(|f| matches!(f, crate::cards::StaticAbilityFragment::ChosenCreatureType))
        })
        && newer(reference)
        && let Some(EntryChoiceValue::CreatureType(typ)) = world.entry_value(reference)
    {
        mask.chosen = Some(typ);
    }
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
    mask.apply_copy_types(Some(world.recipe(id).copy_types()));
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
        Set(crate::cards::BasicLandType),
    }
    let object = world.object(id);
    let reference = world.reference(id);
    let mut events: Vec<(u64, Event)> = characteristic_layers::land_kind_events(world, id)
        .into_iter()
        .map(|(time, land)| (time, Event::Land(land)))
        .collect();
    events.extend(
        world
            .roles()
            .into_iter()
            .flat_map(|state| &state.basic_land_types)
            .filter(|row| row.recipient == reference)
            .map(|row| (row.timestamp, Event::Add(row.kind))),
    );
    if object.card.static_ability_groups().iter().any(|group| {
        group
            .fragments
            .iter()
            .any(|f| matches!(f, crate::cards::StaticAbilityFragment::ChosenBasicLandType))
    }) && let Some(EntryChoiceValue::BasicLandType(kind)) = world.entry_value(reference)
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
        events.push((time, Event::Set(kind)));
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
    let mut is_land = world.recipe(id).kinds() & 2 != 0;
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
    for (time, event) in &events {
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
            Event::Set(kind) if is_land => {
                present = 1 << kind.index();
                lost_original_types = true;
                ability_changes.push((*time, BasicLandAbilityChange::Set(kind)));
            }
            Event::Add(_) | Event::Set(_) => {}
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
    ) {
        let Some(source) = item.source else {
            return;
        };
        let recipients: Vec<_> = self
            .keyword_subjects(item, subject)
            .into_iter()
            .filter(|&id| self.on_battlefield(id) && self.current_kinds(id) & 2 != 0)
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
                .basic_land_types
                .push(BasicLandTypesEffectView {
                    recipient,
                    source: source.object,
                    kind,
                    timestamp,
                });
        }
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

    /// CR 614.12: a return effect created simultaneously with entry does not
    /// yet exist when choosing how that permanent enters.
    pub(super) fn entry_replacement_subtypes(m: &entries::EntryMove) -> GrantedSubtypes {
        Self::entry_subtypes(m, false)
    }

    fn entry_subtypes(m: &entries::EntryMove, include_return_effect: bool) -> GrantedSubtypes {
        if m.food_return.is_some() {
            return GrantedSubtypes::food_only();
        }
        let mut result = GrantedSubtypes::default();
        result.apply_copy_types(m.before.copy_recipe.map(CopyRecipe::copy_types));
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
        if include_return_effect && m.before.kinds & CREATURE_OR_KINDRED != 0 {
            match m.return_modification.map(|(kind, _)| kind) {
                Some(ReturnKind::Angel) => result.insert("Angel"),
                Some(ReturnKind::Demon) => result.insert("Demon"),
                Some(ReturnKind::HeroFinality) => result.insert("Hero"),
                _ => {}
            }
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
