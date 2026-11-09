//! Two exact physical cards share one permanent until it changes zones.
use super::ability_world::CharacteristicWorld;
use super::*;
use crate::cards::MeldPair;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeldEntry {
    pub companion: ObjectRef,
    pub front_cards: [Card; 2],
    pub combined: Card,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeldedPermanent {
    pub object: ObjectRef,
    pub entry: MeldEntry,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MeldedPermanentView {
    pub object: ObjectRef,
    pub front_cards: [Card; 2],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MeldDepartureOrder {
    pub zone: Zone,
    /// First is uppermost in a graveyard or earliest in exile timestamp order.
    pub cards: [ObjectRef; 2],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct MeldOrderingChoice {
    pub permanent: ObjectRef,
    pub destination: Zone,
    pub player: Player,
    pub components: [ObjectRef; 2],
    pub front_cards: [Card; 2],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn owned_permanent<W: CharacteristicWorld + ?Sized>(
    world: &W,
    object: ObjectRef,
    player: Player,
) -> bool {
    let permanent = world.object(object.card);
    permanent.zone == Zone::Battlefield
        && world.reference(object.card) == object
        && permanent.owner == player
        && permanent.controller == player
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn matching_partner<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    partner: ObjectRef,
    player: Player,
    name: &str,
) -> bool {
    source != partner
        && owned_permanent(world, source, player)
        && owned_permanent(world, partner, player)
        && world.kinds(partner.card) & 1 != 0
        && world.name(partner.card) == name
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owns_source_and_named_creature<W: CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    player: Player,
    name: &str,
) -> bool {
    (0..world.game().state.objects.len())
        .any(|id| matching_partner(world, source, world.reference(id as CardId), player, name))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn begin_meld(&mut self, item: &SpellView, pair: MeldPair) -> bool {
        self.start_effect_choice(
            item,
            EffectChoiceKind::MeldPartner(pair),
            PlayerSet::Controller,
            evidence::cost(0).filter,
            1,
            None,
        )
    }

    pub(super) fn meld_chosen_pair(
        &mut self,
        player: Player,
        source: ObjectRef,
        partner: ObjectRef,
        pair: MeldPair,
    ) {
        if !matching_partner(self, source, partner, player, pair.partner.name()) {
            return;
        }
        let death_sources = self.death_trigger_sources();
        let events = self.move_objects_raw(&[source.card, partner.card], Zone::Exile);
        self.finish_zone_events(&events, &death_sources);
        let [first, second] = events.as_slice() else {
            return;
        };
        let exiled_front = |event: &ZoneChange, card: Card| {
            let object = event.after;
            event.moved
                && event.was_card
                && event.destination == Zone::Exile
                && self.object_ref(object.card) == object
                && self.state.objects[object.card as usize].card == card
                && !self.exiled_face_down(object)
        };
        if !exiled_front(first, pair.source) || !exiled_front(second, pair.partner) {
            return;
        }
        let mut movement = self.entry_move(first.after.card, Zone::Battlefield, false);
        movement.project_printed_face(pair.combined);
        movement.controller = player;
        movement.meld = Some(Box::new(MeldEntry {
            companion: second.after,
            front_cards: [pair.source, pair.partner],
            combined: pair.combined,
        }));
        self.stage_entries(vec![movement], None);
    }

    pub(super) fn finish_meld_entry(&mut self, object: ObjectRef, mut entry: MeldEntry) {
        let companion = self.object_view(entry.companion.card);
        assert_eq!(companion.reference, entry.companion);
        assert_eq!(companion.zone, Zone::Exile);
        let event = self.commit_planned_zone_move(
            companion,
            zone_replacements::Plan::unreplaced(Zone::Outside),
            false,
            Counters::default(),
            EntryIdentity::default(),
            ZoneObservers::default(),
        );
        assert!(event.moved, "the exact companion was incorporated");
        entry.companion = event.after;
        self.state
            .roles
            .get_or_insert_default()
            .melded_permanents
            .push(MeldedPermanent { object, entry });
        // The first future departure can require an owned order choice.
        self.state.zone_rules.get_or_insert_default();
    }

    pub(super) fn melded_permanent(&self, object: ObjectRef) -> Option<MeldEntry> {
        self.state
            .roles
            .as_ref()?
            .melded_permanents
            .iter()
            .find(|row| row.object == object)
            .map(|row| row.entry)
    }

    pub(super) fn forget_melded_permanent(&mut self, object: ObjectRef) {
        if let Some(roles) = &mut self.state.roles {
            roles.melded_permanents.retain(|row| row.object != object);
        }
    }

    pub(super) fn split_melded_permanent(
        &mut self,
        permanent: ObjectRef,
        entry: MeldEntry,
        destination: Zone,
        companion_first: bool,
    ) -> [ObjectRef; 2] {
        let companion = self.object_view(entry.companion.card);
        assert_eq!(companion.reference, entry.companion);
        assert_eq!(companion.zone, Zone::Outside);
        let event = self.commit_planned_zone_move(
            companion,
            zone_replacements::Plan::unreplaced(destination),
            false,
            Counters::default(),
            EntryIdentity::default(),
            ZoneObservers::default(),
        );
        assert!(event.moved, "the meld component followed the permanent");
        let cards = if companion_first {
            [event.after, permanent]
        } else {
            [permanent, event.after]
        };
        if destination == Zone::Library && !companion_first {
            let library = &mut self.state.players[event.before.owner.index()].library;
            let len = library.len();
            library.swap(len - 2, len - 1);
        }
        if matches!(destination, Zone::Graveyard | Zone::Exile) {
            self.state
                .roles
                .as_mut()
                .unwrap()
                .meld_departure_orders
                .push(MeldDepartureOrder {
                    zone: destination,
                    cards,
                });
        }
        cards
    }

    pub(super) fn forget_meld_departure_order(&mut self, object: ObjectRef) {
        if let Some(roles) = &mut self.state.roles {
            roles
                .meld_departure_orders
                .retain(|row| !row.cards.contains(&object));
        }
    }

    pub(super) fn melded_views(&self) -> Vec<MeldedPermanentView> {
        self.state.roles.as_ref().map_or_else(Vec::new, |roles| {
            roles
                .melded_permanents
                .iter()
                .map(|row| MeldedPermanentView {
                    object: row.object,
                    front_cards: row.entry.front_cards,
                })
                .collect()
        })
    }
}
