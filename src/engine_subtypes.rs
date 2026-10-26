//! Layer-four creature subtype overlays. Objects retain only their printed identity;
//! captured views carry a compact value, and speculative resource queries borrow
//! the same exact attachment test over their own object overlay.
use super::*;
use serde::ser::{SerializeSeq, SerializeStruct};

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
const CREATURE_OR_KINDRED: u16 = 1 | 128;
const JOB_MASK: u32 = (1 << JOB_TYPES.len()) - 1;
const SUPPRESS_PRINTED: u32 = 1 << JOB_TYPES.len();
const LAND_ONLY: u32 = SUPPRESS_PRINTED << 1;
const ALL_CREATURE_TYPES: u32 = LAND_ONLY << 1;
const FLAGS_MASK: u32 = SUPPRESS_PRINTED | LAND_ONLY | ALL_CREATURE_TYPES;
// Preserve all existing bit positions. Copy exceptions use previously unused
// high bits, keeping the common borrowed subtype value at its original size.
const COPY_HUMAN: u32 = ALL_CREATURE_TYPES << 1;
const COPY_VILLAIN: u32 = COPY_HUMAN << 1;
const COPY_MERCENARY: u32 = COPY_VILLAIN << 1;
const ADDED_SOLDIER: u32 = COPY_MERCENARY << 1;
const ADDED_GOBLIN: u32 = ADDED_SOLDIER << 1;
const ADDED_MASK: u32 =
    JOB_MASK | COPY_HUMAN | COPY_VILLAIN | COPY_MERCENARY | ADDED_SOLDIER | ADDED_GOBLIN;
const _: () = assert!(JOB_TYPES.len() + 5 <= u32::BITS as usize);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrantedSubtypes {
    added: u32,
    chosen: Option<crate::cards::CreatureType>,
    /// None retains printed types (including changeling); Some replaces only
    /// creature types, preserving e.g. Equipment, Aura, Forest and Lesson.
    creature_types: Option<&'static [&'static str]>,
}
impl GrantedSubtypes {
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
        (self.added & Self::copy_type_bit(subtype) != 0)
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
        self.added &= FLAGS_MASK;
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
                ] {
                    if self.0 & bit != 0 {
                        out.serialize_element(subtype)?;
                    }
                }
                out.end()
            }
        }
        let mut out = serializer.serialize_struct("GrantedSubtypes", 6)?;
        out.serialize_field("suppress_printed", &self.flag(SUPPRESS_PRINTED))?;
        out.serialize_field("land_only", &self.flag(LAND_ONLY))?;
        out.serialize_field("all_creature_types", &self.flag(ALL_CREATURE_TYPES))?;
        out.serialize_field("chosen", &self.chosen)?;
        out.serialize_field("added", &Added(self.added & ADDED_MASK))?;
        out.serialize_field("creature_types", &self.creature_types)?;
        out.end()
    }
}

/// A resolution-created replacement, lasting for this exact incarnation only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CreatureTypesEffectView {
    pub recipient: ObjectRef,
    pub source: ObjectRef,
    pub types: &'static [&'static str],
    pub additional: bool,
    pub timestamp: u64,
}
impl ObjectView {
    pub fn has_subtype(&self, subtype: &str) -> bool {
        self.granted_subtypes.has(self.card, subtype)
    }
}
impl DamageFacts {
    pub fn has_subtype(self, subtype: &str) -> bool {
        self.granted_subtypes.has(self.card, subtype)
    }
}
/// Earlier subtype layers over actual or projected source registries.
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
    changes.sort_by_key(|row| row.0);
    let replacement_time = changes
        .iter()
        .rev()
        .find(|(_, change)| !matches!(change, characteristic_layers::SubtypeChange::Add(_)))
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
            characteristic_layers::SubtypeChange::LandOnly => {
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
                    mask.set_flag(SUPPRESS_PRINTED, true);
                    mask.set_flag(LAND_ONLY, false);
                }
            }
        }
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

impl Game {
    pub(super) fn set_creature_types(
        &mut self,
        item: &SpellView,
        subject: Subject,
        types: &'static [&'static str],
    ) {
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
                source: item.object,
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
        }
    }
}

impl Game {
    pub(super) fn entry_choice_subtypes(m: &entries::EntryMove) -> GrantedSubtypes {
        let mut result = GrantedSubtypes::default();
        result.apply_copy_types(m.before.copy_recipe.map(CopyRecipe::copy_types));
        if m.before.card.static_ability_groups().iter().any(|g| {
            g.fragments
                .iter()
                .any(|f| matches!(f, crate::cards::StaticAbilityFragment::ChosenCreatureType))
        }) && let Some(EntryChoiceValue::CreatureType(typ)) = m.choice
        {
            result.chosen = Some(typ);
        }
        if m.before.kinds & CREATURE_OR_KINDRED != 0 {
            match m.return_modification.map(|(kind, _)| kind) {
                Some(ReturnKind::Angel) => result.insert("Angel"),
                Some(ReturnKind::Demon) => result.insert("Demon"),
                Some(ReturnKind::HeroFinality) => result.insert("Hero"),
                _ => {}
            }
        }
        result
    }
}

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
