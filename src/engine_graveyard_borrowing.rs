//! Exact acquisition lifetimes for continuous graveyard activation borrowing.
//! Inventory reads the supplied world; funding never falls back to live donors.
use super::ability_world::CharacteristicWorld;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub source: ObjectRef,
    pub provider: Card,
    pub origin: Option<u64>,
    pub controller: Player,
    pub donor: ObjectRef,
    pub definition: Card,
    pub first_slot: u64,
    pub width: u64,
    pub active: bool,
}
impl std::hash::Hash for Row {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(
            &(
                self.source,
                self.provider,
                self.origin,
                self.controller.index(),
                self.donor,
                self.definition,
                self.first_slot,
                self.width,
                self.active,
            ),
            state,
        );
    }
}
pub(super) type Rows = Option<SharedVec<Row>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<Row>>() + r.len() * std::mem::size_of::<Row>()
    })
}
fn source_subtype<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Option<&'static str> {
    let source = world.object(id);
    if source.zone != Zone::Battlefield {
        return None;
    }
    let subtype = source.card.graveyard_activation_subtype()?;
    world
        .fragment_available(id, |f| {
            matches!(f,
        crate::cards::StaticAbilityFragment::BorrowGraveyardActivations(s) if *s==subtype)
        })
        .then_some(subtype)
}
fn width(_card: Card) -> u64 {
    borrowed_activations::DONOR_SLOT_WIDTH
}
/// Existing exile and graveyard acquisitions occupy a single source-local space.
pub(super) fn next_slot<W: CharacteristicWorld + ?Sized>(world: &W, source: ObjectRef) -> u64 {
    world
        .graveyard_borrowing()
        .iter()
        .filter(|r| r.source == source)
        .map(|r| {
            r.first_slot
                .checked_add(r.width)
                .capacity("borrowed activation slots exhausted")
        })
        .chain(
            world
                .game()
                .state
                .borrowed_links
                .as_ref()
                .into_iter()
                .flat_map(|r| r.iter())
                .filter(|r| r.source == source)
                .map(|r| {
                    r.first_slot
                        .checked_add(r.width)
                        .capacity("borrowed activation slots exhausted")
                }),
        )
        .max()
        .unwrap_or(0)
}
/// Pure candidate acquisition, including new exact donors introduced by a
/// virtual sacrifice. Provisional identities agree with the subsequent real refresh.
fn current_rows<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Vec<Row> {
    let Some(subtype) = source_subtype(world, id) else {
        return Vec::new();
    };
    let source = world.reference(id);
    let provider = world.object(id).card;
    let controller = world.object(id).controller;
    let origin = world.ability_origin(id);
    let mut next = next_slot(world, source);
    let mut rows = Vec::new();
    for i in 0..world.game().state.objects.len() {
        let id = i as CardId;
        let card = world.object(id);
        if card.zone != Zone::Graveyard
            || card.owner != controller
            || !card.is_card()
            || !world.subtypes(id).has(card.card, subtype)
        {
            continue;
        }
        let count = width(card.card);
        if count == 0 {
            continue;
        }
        let donor = world.reference(id);
        let retained = world.graveyard_borrowing().iter().find(|r| {
            r.active
                && r.source == source
                && r.provider == provider
                && r.origin == origin
                && r.controller == controller
                && r.donor == donor
                && r.definition == card.card
        });
        if let Some(row) = retained {
            rows.push(*row);
        } else {
            let first_slot = next;
            next = next
                .checked_add(count)
                .capacity("borrowed activation slots exhausted");
            rows.push(Row {
                source,
                provider,
                origin,
                controller,
                donor,
                definition: card.card,
                first_slot,
                width: count,
                active: true,
            });
        }
    }
    rows
}
pub(super) fn refreshed<W: CharacteristicWorld + ?Sized>(world: &W) -> Rows {
    let old = world.graveyard_borrowing();
    // Ordinary worlds neither allocate a registry nor scan all graveyards.
    if old.is_empty()
        && !world.static_sources().iter().any(|&id| {
            world
                .object(id)
                .card
                .graveyard_activation_subtype()
                .is_some()
        })
    {
        return None;
    }
    let mut desired = Vec::new();
    for &id in world.static_sources() {
        if world
            .object(id)
            .card
            .graveyard_activation_subtype()
            .is_some()
        {
            desired.extend(current_rows(world, id));
        }
    }
    let mut rows: Vec<_> = old
        .iter()
        .filter(|r| {
            world.object(r.source.card).zone == Zone::Battlefield
                && world.reference(r.source.card) == r.source
        })
        .copied()
        .collect();
    for row in &mut rows {
        row.active = desired
            .iter()
            .any(|d| d.source == row.source && d.first_slot == row.first_slot);
    }
    for row in desired {
        if !rows
            .iter()
            .any(|r| r.source == row.source && r.first_slot == row.first_slot)
        {
            rows.push(row);
        }
    }
    if rows == old {
        return world.graveyard_borrowing_state().clone();
    }
    (!rows.is_empty()).then(|| rows.into())
}
pub(super) fn inventory<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<BorrowedDescriptor> {
    let mut result = Vec::new();
    for row in current_rows(world, id) {
        for (index, _) in row.definition.activations().iter().enumerate() {
            let ability = index as u16;
            if row.definition.legacy_subtype_mana_activation(ability) {
                continue;
            }
            if row.definition.grants_activation(ability)
                || row.definition.self_granted_activation_template(ability)
            {
                continue;
            }
            let slot = 0;
            result.push(BorrowedDescriptor {
                source: row.source,
                donor: row.donor,
                zone: Zone::Graveyard,
                definition: row.definition,
                ability,
                slot,
                used: false,
                once_per_turn: false,
                acquisition: row.first_slot,
            });
        }
        if world.kinds(row.donor.card) & 2 != 0 {
            let subtypes = world.subtypes(row.donor.card);
            for (color, subtype) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .enumerate()
            {
                if !subtypes.has(row.definition, subtype) {
                    continue;
                }
                let slot = 0;
                result.push(BorrowedDescriptor {
                    source: row.source,
                    donor: row.donor,
                    zone: Zone::Graveyard,
                    definition: row.definition,
                    ability: u16::MAX - 1 - color as u16,
                    slot,
                    used: false,
                    once_per_turn: false,
                    acquisition: row.first_slot,
                });
            }
        }
        if !row.definition.is_land() && row.definition.mana_color().is_some() {
            let slot = 0;
            result.push(BorrowedDescriptor {
                source: row.source,
                donor: row.donor,
                zone: Zone::Graveyard,
                definition: row.definition,
                ability: u16::MAX,
                slot,
                used: false,
                once_per_turn: false,
                acquisition: row.first_slot,
            });
        }
    }
    result
}
pub(super) fn forget(rows: &mut Rows, source: ObjectRef) {
    if let Some(r) = rows {
        r.retain(|r| r.source != source);
        if r.is_empty() {
            *rows = None;
        }
    }
}
impl Game {
    pub(super) fn refresh_graveyard_borrowing(&mut self) {
        self.state.graveyard_borrowing = refreshed(self);
    }
}
