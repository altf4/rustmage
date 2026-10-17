//! Layer-four creature subtype overlays. Objects retain only their printed identity;
//! captured views carry a compact value, and speculative resource queries borrow
//! the same exact attachment test over their own object overlay.
use super::*;
use serde::ser::{SerializeSeq, SerializeStruct};

const JOB_TYPES: [&str; 11] = [
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
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GrantedSubtypes {
    added: u16,
    /// None retains printed types (including changeling); Some replaces only
    /// creature types, preserving e.g. Equipment, Aura, Forest and Lesson.
    creature_types: Option<&'static [&'static str]>,
}
impl GrantedSubtypes {
    /// Membership supplied by this overlay, excluding retained printed types.
    /// Use ObjectView::has_subtype for the full effective characteristic.
    pub fn contains(self, subtype: &str) -> bool {
        self.creature_types
            .is_some_and(|types| types.contains(&subtype))
            || JOB_TYPES
                .iter()
                .position(|s| *s == subtype)
                .is_some_and(|i| self.added & (1 << i) != 0)
    }
    pub(crate) fn has(self, card: Card, subtype: &str) -> bool {
        self.contains(subtype)
            || if crate::cards::is_creature_type(subtype) {
                self.creature_types.map_or_else(
                    || card.has_subtype(subtype),
                    |types| types.contains(&subtype),
                )
            } else {
                card.has_subtype(subtype)
            }
    }
    fn replace_creature_types(&mut self, types: &'static [&'static str]) {
        self.added = 0;
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
        struct Added(u16);
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
        let mut out = serializer.serialize_struct("GrantedSubtypes", 2)?;
        out.serialize_field("added", &Added(self.added))?;
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
    ) -> GrantedSubtypes {
        let mut mask = GrantedSubtypes::default();
        if target.zone != Zone::Battlefield || self.object_kinds(id, target) & 1 == 0 {
            return mask;
        }
        let reference = ObjectRef {
            card: id,
            generation: target.generation,
        };
        // Find the latest resolving replacement first. All supported static
        // subtype effects are additions, so only additions newer than it survive.
        let latest = self.state.roles.as_ref().and_then(|state| {
            state
                .creature_types
                .iter()
                .filter(|row| row.recipient == reference)
                .max_by_key(|row| row.timestamp)
        });
        let replacement_time = latest.map(|row| row.timestamp);
        if let Some(row) = latest {
            mask.replace_creature_types(row.types);
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
        for group in target.card.static_ability_groups() {
            for fragment in group.fragments {
                if let crate::cards::StaticAbilityFragment::SourceCounterSubtype {
                    kind,
                    minimum,
                    subtype,
                    ..
                } = *fragment
                    && target.counters.get(kind) >= minimum
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
            if o.zone == Zone::Battlefield
                && self.object_kinds(source, o) & 1 == 0
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
    pub(super) fn granted_subtypes(&self, id: CardId) -> GrantedSubtypes {
        self.state
            .objects
            .get(id as usize)
            .map_or_else(GrantedSubtypes::default, |o| {
                self.granted_subtypes_with(id, o, |source| self.state.objects.get(source as usize))
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
