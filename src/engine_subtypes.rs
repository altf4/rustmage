//! Additive layer-4 Equipment grants. Objects retain only their printed identity;
//! captured views carry a compact value, and speculative resource queries borrow
//! the same exact attachment test over their own object overlay.
use super::*;
use serde::ser::SerializeSeq;

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
pub struct GrantedSubtypes(u16);
impl GrantedSubtypes {
    pub fn contains(self, subtype: &str) -> bool {
        JOB_TYPES
            .iter()
            .position(|s| *s == subtype)
            .is_some_and(|i| self.0 & (1 << i) != 0)
    }
    fn insert(&mut self, subtype: &str) {
        let i = JOB_TYPES
            .iter()
            .position(|s| *s == subtype)
            .expect("authenticated immutable subtype grant");
        self.0 |= 1 << i;
    }
}
impl Serialize for GrantedSubtypes {
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
impl ObjectView {
    pub fn has_subtype(&self, subtype: &str) -> bool {
        self.card.has_subtype(subtype) || self.granted_subtypes.contains(subtype)
    }
}
impl DamageFacts {
    pub fn has_subtype(self, subtype: &str) -> bool {
        self.card.has_subtype(subtype) || self.granted_subtypes.contains(subtype)
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
        for &source in &self.state.attachment_sources {
            let Some(o) = lookup(source) else {
                continue;
            };
            if o.zone == Zone::Battlefield
                && self.object_kinds(source, o) & 1 == 0
                && o.attached_to == Some(reference)
                && let Some(subtype) = o.card.attached_subtype()
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
            (if o.zone == Zone::Stack {
                self.effective_spell_card(id)
            } else {
                o.card
            })
            .has_subtype(subtype)
                || self.granted_subtypes(id).contains(subtype)
        })
    }
}
