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
const JOB_TYPES: [&str; 14] = [
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
];
// Fourteen named grants and three projection flags use the view's existing padding.
const CREATURE_OR_KINDRED: u16 = 1 | 128;
const JOB_MASK: u32 = (1 << JOB_TYPES.len()) - 1;
const SUPPRESS_PRINTED: u32 = 1 << JOB_TYPES.len();
const LAND_ONLY: u32 = SUPPRESS_PRINTED << 1;
const ALL_CREATURE_TYPES: u32 = LAND_ONLY << 1;
const FLAGS_MASK: u32 = SUPPRESS_PRINTED | LAND_ONLY | ALL_CREATURE_TYPES;
const _: () = assert!(JOB_TYPES.len() + 3 <= u32::BITS as usize);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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
        (self.flag(ALL_CREATURE_TYPES) && crate::cards::is_creature_type(subtype))
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
    fn insert(&mut self, subtype: &str) {
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
                out.end()
            }
        }
        let mut out = serializer.serialize_struct("GrantedSubtypes", 6)?;
        out.serialize_field("suppress_printed", &self.flag(SUPPRESS_PRINTED))?;
        out.serialize_field("land_only", &self.flag(LAND_ONLY))?;
        out.serialize_field("all_creature_types", &self.flag(ALL_CREATURE_TYPES))?;
        out.serialize_field("chosen", &self.chosen)?;
        out.serialize_field("added", &Added(self.added & JOB_MASK))?;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_turn: Option<u32>,
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
impl Game {
    /// The index is immutable for a query. A virtual sacrifice changes the
    /// borrowed lookup's zone, so its grant disappears without mutating Game.
    pub(super) fn granted_subtypes_with<'a>(
        &self,
        id: CardId,
        target: &Object,
        lookup: impl Fn(CardId) -> Option<&'a Object>,
        counter_count: impl Fn(CardId, CounterKind) -> u64,
        kinds: impl Fn(CardId, &Object) -> u16,
        mut changes: Vec<(u64, characteristic_layers::SubtypeChange)>,
    ) -> GrantedSubtypes {
        let mut mask = GrantedSubtypes::default();
        let reference = ObjectRef {
            card: id,
            generation: target.generation,
        };
        if self
            .copy_recipe(reference)
            .is_some_and(|r| r.exceptions.nightmare)
        {
            mask.insert("Nightmare");
        }
        // Copy exceptions already exist while an owned token awaits entry.
        // Later continuous type changes apply only on the battlefield.
        if target.zone != Zone::Battlefield {
            return mask;
        }
        if let Some(state) = &self.state.roles {
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
            self.state
                .roles
                .as_ref()
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
        if self.state.entry_values.is_some()
            && target.card.static_ability_groups().iter().any(|g| {
                g.fragments
                    .iter()
                    .any(|f| matches!(f, crate::cards::StaticAbilityFragment::ChosenCreatureType))
            })
            && newer(reference)
            && let Some(EntryChoiceValue::CreatureType(typ)) = self.entry_value(reference)
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
        for &source in &self.state.attachment_sources {
            let Some(o) = lookup(source) else {
                continue;
            };
            // Attachment indexing also includes stat-only Equipment and Auras.
            // Those are not layer-four providers and need no ability timestamp.
            let all_types = o.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::AttachedAllCreatureTypes
                    )
                })
            });
            let subtype = o.card.attached_subtype();
            if o.zone == Zone::Battlefield
                && kinds(source, o) & 1 == 0
                && o.attached_to == Some(reference)
                && (subtype.is_some() || all_types)
                && newer(ObjectRef {
                    card: source,
                    generation: o.generation,
                })
            {
                if let Some(subtype) = subtype {
                    mask.insert(subtype);
                }
                if all_types {
                    mask.set_flag(ALL_CREATURE_TYPES, true);
                }
            }
        }
        mask
    }
    pub(super) fn granted_subtypes(&self, id: CardId) -> GrantedSubtypes {
        self.state
            .objects
            .get(id as usize)
            .map_or_else(GrantedSubtypes::default, |o| {
                self.granted_subtypes_with(
                    id,
                    o,
                    |source| self.state.objects.get(source as usize),
                    |source, kind| self.counter_count(source, kind),
                    |id, object| self.object_kinds(id, object),
                    characteristic_layers::subtype_changes(self, id),
                )
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
        self.set_creature_types_with_expiry(item, subject, types, None);
    }
    pub(super) fn set_creature_types_until_end_turn(
        &mut self,
        item: &SpellView,
        subject: Subject,
        types: &'static [&'static str],
    ) {
        self.set_creature_types_with_expiry(item, subject, types, Some(self.state.turn));
    }
    fn set_creature_types_with_expiry(
        &mut self,
        item: &SpellView,
        subject: Subject,
        types: &'static [&'static str],
        expires_turn: Option<u32>,
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
            // A permanent replacement supersedes older ones. A temporary
            // replacement retains them so expiration restores the earlier layer.
            if expires_turn.is_none() {
                state
                    .creature_types
                    .retain(|row| row.recipient != recipient);
            }
            state.creature_types.push(CreatureTypesEffectView {
                recipient,
                source: item.object,
                types,
                additional: false,
                expires_turn,
                timestamp,
            });
        }
    }
    pub(super) fn expire_creature_types(&mut self) {
        if let Some(state) = &mut self.state.roles {
            let turn = self.state.turn;
            state
                .creature_types
                .retain(|row| row.expires_turn.is_none_or(|expiry| expiry > turn));
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
        if m.before.copy_recipe.is_some_and(|r| r.exceptions.nightmare) {
            result.insert("Nightmare");
        }
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
                _ => {}
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_named_grants_and_projection_flags_fit_without_growing_the_view() {
        assert_eq!(std::mem::size_of::<GrantedSubtypes>(), 24);
        assert_eq!(std::mem::size_of::<Object>(), 144);
        assert_eq!(std::mem::size_of::<Pending>(), 40);
        assert_eq!(std::mem::size_of::<ManaProduction>(), 32);
        let mut view = GrantedSubtypes::default();
        for subtype in JOB_TYPES {
            view.insert(subtype);
        }
        for flag in [SUPPRESS_PRINTED, LAND_ONLY, ALL_CREATURE_TYPES] {
            view.set_flag(flag, true);
        }
        let value = serde_json::to_value(view).unwrap();
        assert_eq!(value["added"], serde_json::json!(JOB_TYPES));
        for flag in ["suppress_printed", "land_only", "all_creature_types"] {
            assert_eq!(value[flag], true);
        }
        view.replace_creature_types(&["Bear"]);
        let replaced = serde_json::to_value(view).unwrap();
        assert_eq!(replaced["added"], serde_json::json!([]));
        assert_eq!(replaced["creature_types"], serde_json::json!(["Bear"]));
        for flag in ["suppress_printed", "land_only", "all_creature_types"] {
            assert_eq!(replaced[flag], true);
        }
    }
}
