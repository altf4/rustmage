//! Activated abilities acquired from exact donor incarnations.
use super::ability_world::CharacteristicWorld;
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn is_false(value: &bool) -> bool {
    !*value
}
/// Acquisition identifiers are separate from public ability slots. Both are
/// allocated monotonically within an exact source incarnation.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) const DONOR_SLOT_WIDTH: u64 = 1;
// The layer evaluator and exile/graveyard inventory provision slots independently.
// Reserve separate ranges below the exact JSON integer limit. Fresh inherited
// link cohorts share the inherited range with slots so the two cannot alias.
const INHERITED_SLOT_BASE: u64 = 1 << 52;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct LinkCohort {
    id: u64,
    paired: bool,
    live: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct SlotRow {
    pub(super) source: ObjectRef,
    grantor: Option<ObjectRef>,
    acquisition: u64,
    graveyard: bool,
    pub(super) definition: Option<Card>,
    ability: u16,
    slot: u64,
    inherited: Option<inherited_activations::Acquisition>,
    link: Option<LinkCohort>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type SlotRows = Option<SharedVec<SlotRow>>;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Link {
    pub source: ObjectRef,
    pub origin: Option<u64>,
    pub exiled: ObjectRef,
    pub first_slot: u64,
    pub width: u64,
    pub once_per_turn: bool,
    /// This link supplies a continuous grant to other eligible recipients.
    pub shared: bool,
    /// Materials remain associated with the crafted permanent through copy changes.
    pub craft: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct BorrowedAbilityView {
    pub source: ObjectRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grantor: Option<ObjectRef>,
    pub exiled: ObjectRef,
    pub definition: Card,
    /// The final five indices identify the exiled object's intrinsic mana abilities.
    pub ability: u16,
    pub slot: u64,
    pub used: bool,
    /// The acquisition limits this ability to once per turn independently of
    /// any restriction printed on the donor.
    #[serde(skip_serializing_if = "is_false")]
    pub acquired_once_per_turn: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Use {
    pub source: ObjectRef,
    pub slot: u64,
    pub turn: u32,
    pub lifetime: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GraveyardBorrowedAbilityView {
    pub source: ObjectRef,
    pub graveyard: ObjectRef,
    pub definition: Card,
    pub ability: u16,
    pub slot: u64,
    pub used: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Descriptor {
    pub source: ObjectRef,
    pub grantor: Option<ObjectRef>,
    pub donor: ObjectRef,
    pub zone: Zone,
    pub definition: Card,
    pub ability: u16,
    pub slot: u64,
    pub(super) link: Option<LinkCohort>,
    pub used: bool,
    pub once_per_turn: bool,
    pub(super) acquisition: u64,
    pub(super) inherited: Option<inherited_activations::Acquisition>,
    pub(super) mana: Option<ManaProduction>,
    pub(super) attachment: Option<AttachmentTriggerView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Descriptor {
    pub(super) fn link_cohort(self) -> u64 {
        self.link.map_or(self.slot, |link| link.id)
    }

    pub(super) fn rules(self) -> &'static ActivatedAbility {
        self.definition.activation(self.ability)
    }

    pub(super) fn mana_production(self) -> Option<ManaProduction> {
        self.mana.or_else(|| self.rules().mana_production())
    }

    pub(super) fn mana_action(
        self,
        counters: Counters,
        payment: Option<Mana>,
        production: Mana,
    ) -> Action {
        let card = self.source.card;
        let ability = self.ability;
        match (self.zone, payment) {
            (Zone::Battlefield, None) => Action::AnnounceInheritedMana {
                source: self.source,
                slot: self.slot,
                counter_payment: counters,
                production,
            },
            (Zone::Battlefield, Some(payment)) => Action::ActivateInheritedMana {
                source: self.source,
                slot: self.slot,
                counter_payment: counters,
                payment,
                production,
            },
            (Zone::Graveyard, None) => Action::AnnounceGraveyardBorrowedMana {
                card,
                graveyard: self.donor,
                ability,
                counter_payment: counters,
                production,
            },
            (Zone::Graveyard, Some(payment)) => Action::ActivateGraveyardBorrowedMana {
                card,
                graveyard: self.donor,
                ability,
                counter_payment: counters,
                payment,
                production,
            },
            (_, None) => Action::AnnounceBorrowedMana {
                card,
                exiled: self.donor,
                ability,
                counter_payment: counters,
                production,
            },
            (_, Some(payment)) => Action::ActivateBorrowedMana {
                card,
                exiled: self.donor,
                ability,
                counter_payment: counters,
                payment,
                production,
            },
        }
    }

    fn slot_row(self) -> SlotRow {
        let intrinsic = self.ability >= crate::cards::activation_slots::INTRINSIC_COLORLESS
            && self.ability < crate::cards::activation_slots::INTRINSIC_FALLBACK;
        SlotRow {
            source: self.source,
            grantor: self.grantor,
            acquisition: self.acquisition,
            graveyard: self.zone == Zone::Graveyard,
            definition: (!intrinsic).then_some(self.definition),
            ability: self.ability,
            slot: self.slot,
            inherited: self.inherited,
            link: self.inherited.and(self.link),
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn abilities<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Vec<Descriptor> {
    let mut result = inventory(world, id, InventoryStage::Live);
    result.extend(inherited_activations::abilities(world, id));
    result
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn has_shared_links(game: &Game) -> bool {
    game.state.borrowed_links.as_ref().is_some_and(|links| {
        links.iter().any(|link| {
            link.shared
                && game
                    .state
                    .objects
                    .get(link.exiled.card as usize)
                    .is_some_and(|object| {
                        object.zone == Zone::Exile && object.generation == link.exiled.generation
                    })
        })
    })
}

fn source_ids<W: CharacteristicWorld + ?Sized>(world: &W, include_graveyard: bool) -> Vec<CardId> {
    if has_shared_links(world.game()) {
        return (0..world.game().state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| world.object(id).zone == Zone::Battlefield)
            .collect();
    }
    let mut sources = if include_graveyard {
        world.static_sources().to_vec()
    } else {
        Vec::new()
    };
    for link in world
        .game()
        .state
        .borrowed_links
        .as_ref()
        .into_iter()
        .flat_map(|rows| rows.iter())
    {
        if !sources.contains(&link.source.card) {
            sources.push(link.source.card);
        }
    }
    sources
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum InventoryStage {
    Live,
    Registry,
    BeforeLayerSix,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn inventory<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
    stage: InventoryStage,
) -> Vec<Descriptor> {
    let source = world.object(id);
    if source.zone != Zone::Battlefield
        || !source.card.borrows_activations() && !has_shared_links(world.game())
    {
        return Vec::new();
    }
    let recipient = world.reference(id);
    let check_available = stage == InventoryStage::Live;
    let mut result = Vec::new();
    let mut query = None;
    for link in world
        .game()
        .state
        .borrowed_links
        .as_ref()
        .map_or(&[][..], |rows| rows.as_slice())
    {
        let grantor = if link.shared {
            let provider = world.object(link.source.card);
            if provider.zone != Zone::Battlefield
                || provider.controller != source.controller
                || world.reference(link.source.card) != link.source
                || world.ability_origin(link.source.card) != link.origin
            {
                continue;
            }
            let fragment = |f: &crate::cards::StaticAbilityFragment| {
                matches!(f, crate::cards::StaticAbilityFragment::GrantLinkedExileCreatureActivationsWithCounter(kind)
                    if world.counter_count(id, *kind) != 0)
            };
            if !provider
                .card
                .static_ability_groups()
                .iter()
                .any(|g| g.fragments.iter().any(fragment))
                || world.kinds(id) & 1 == 0
                || check_available
                    && !query
                        .get_or_insert_with(|| {
                            characteristic_query::CharacteristicQuery::in_world(world)
                        })
                        .linked_exile_activation_grant_available(id, link.source, fragment)
            {
                continue;
            }
            Some(link.source)
        } else {
            if !source.card.borrows_exiled_activations()
                || link.source != recipient
                || !link.craft && link.origin != world.ability_origin(id)
                || check_available && !world.printed_abilities_available(id)
            {
                continue;
            }
            None
        };
        let card = world.object(link.exiled.card);
        if card.zone != Zone::Exile
            || world.reference(link.exiled.card) != link.exiled
            || link.shared && (!card.is_card() || !card.card.is_creature())
        {
            continue;
        }
        for (index, _) in card.card.activations().iter().enumerate() {
            if card.card.legacy_subtype_mana_activation(index as u16) {
                continue;
            }
            if card.card.grants_activation(index as u16)
                || card.card.self_granted_activation_template(index as u16)
            {
                continue;
            }
            let slot = 0;
            result.push(Descriptor {
                source: recipient,
                grantor,
                donor: link.exiled,
                zone: Zone::Exile,
                definition: card.card,
                ability: index as u16,
                slot,
                link: None,
                used: false,
                once_per_turn: link.once_per_turn,
                acquisition: link.first_slot,
                inherited: None,
                mana: None,
                attachment: None,
            });
        }
        if world.kinds(link.exiled.card) & 2 != 0 {
            let subtypes = world.subtypes(link.exiled.card);
            for (color, subtype) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .enumerate()
            {
                if !subtypes.has(card.card, subtype) {
                    continue;
                }
                let slot = 0;
                result.push(Descriptor {
                    source: recipient,
                    grantor,
                    donor: link.exiled,
                    zone: Zone::Exile,
                    definition: card.card,
                    ability: crate::cards::activation_slots::intrinsic(color),
                    slot,
                    link: None,
                    used: false,
                    once_per_turn: link.once_per_turn,
                    acquisition: link.first_slot,
                    inherited: None,
                    mana: None,
                    attachment: None,
                });
            }
        }
        if !card.card.is_land() && card.card.mana_color().is_some() {
            let slot = 0;
            result.push(Descriptor {
                source: recipient,
                grantor,
                donor: link.exiled,
                zone: Zone::Exile,
                definition: card.card,
                ability: u16::MAX,
                slot,
                link: None,
                used: false,
                once_per_turn: link.once_per_turn,
                acquisition: link.first_slot,
                inherited: None,
                mana: None,
                attachment: None,
            });
        }
    }
    if let Some(subtype) = source.card.graveyard_activation_subtype() {
        if stage == InventoryStage::BeforeLayerSix {
            result.extend(graveyard_borrowing::inventory_for_subtype(
                world, id, subtype,
            ));
        } else {
            result.extend(graveyard_borrowing::inventory(world, id));
        }
    }
    assign_slots(world, &mut result);
    result
}

// Existing slot identities stay stable. New candidates receive provisional
// slots in inventory order, the same order used when persisting the registry.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn assign_slots<W: CharacteristicWorld + ?Sized>(world: &W, abilities: &mut [Descriptor]) {
    let mut rows = world.borrowed_slots().to_vec();
    for ability in abilities {
        assign_slot(world, &mut rows, ability);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn assign_slot<W: CharacteristicWorld + ?Sized>(
    world: &W,
    rows: &mut Vec<SlotRow>,
    ability: &mut Descriptor,
) {
    let mut desired = ability.slot_row();
    let inherited = desired.inherited.is_some();
    ability.slot = if let Some(old) = rows.iter().find(|row| row.matches(desired)) {
        old.slot
    } else {
        desired.slot = next_identity(rows, desired.source, inherited);
        rows.push(desired);
        desired.slot
    };
    ability.used = world.borrowed_activation_used(ability.source, ability.slot);
    if !inherited && paired_ghost_activation(ability.definition, ability.ability) {
        ability.link = rows
            .iter()
            .filter(|row| {
                row.shares_link_cohort(desired)
                    && paired_ghost_activation(ability.definition, row.ability)
            })
            .map(|row| row.slot)
            .min()
            .map(|id| LinkCohort {
                id,
                paired: true,
                live: true,
            });
    }
}

fn next_identity(rows: &[SlotRow], source: ObjectRef, inherited: bool) -> u64 {
    let start = if inherited { INHERITED_SLOT_BASE } else { 0 };
    let identity = rows
        .iter()
        .filter(|row| row.source == source && row.inherited.is_some() == inherited)
        .map(|row| row.link.map_or(row.slot, |link| row.slot.max(link.id)))
        .max()
        .map_or(start, |previous| previous + 1);
    (identity < start + INHERITED_SLOT_BASE)
        .then_some(identity)
        .capacity("borrowed ability slots exhausted")
}

/// CR607.5: an uninterrupted set of granted abilities shares one link. A new
/// grant after losing it uses a different cohort even if its action slots match.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn assign_inherited_links(rows: &mut Vec<SlotRow>, abilities: &mut [Descriptor]) {
    for index in 0..abilities.len() {
        let ability = abilities[index];
        if ability.link.is_some() || !paired_ghost_activation(ability.definition, ability.ability) {
            continue;
        }
        let desired = ability.slot_row();
        let counterpart = ((index + 1)..abilities.len()).find(|&other| {
            let other = abilities[other];
            other.link.is_none()
                && other.ability != ability.ability
                && desired.shares_link_cohort(other.slot_row())
                && paired_ghost_activation(other.definition, other.ability)
        });
        let paired = counterpart.is_some();
        let id = rows
            .iter()
            .filter(|row| row.shares_link_cohort(desired) && (paired || row.matches(desired)))
            .find_map(|row| {
                row.link
                    .filter(|link| link.live && link.paired == paired)
                    .map(|link| link.id)
            })
            .unwrap_or_else(|| next_identity(rows, ability.source, true));
        let link = Some(LinkCohort {
            id,
            paired,
            live: true,
        });
        abilities[index].link = link;
        remember_slot(rows, abilities[index].slot_row());
        if let Some(counterpart) = counterpart {
            abilities[counterpart].link = link;
            remember_slot(rows, abilities[counterpart].slot_row());
        }
    }
}

fn remember_slot(rows: &mut Vec<SlotRow>, row: SlotRow) {
    if let Some(old) = rows.iter_mut().find(|old| old.matches(row)) {
        *old = row;
    } else {
        rows.push(row);
    }
}

// ReturnLinkedGhosts is the paired reader for one exact graveyard-exile
// template. Other acquired abilities retain their individual slot as origin.
fn paired_ghost_activation(definition: Card, index: u16) -> bool {
    let abilities = definition.activations();
    let Some(ability) = abilities.get(usize::from(index)) else {
        return false;
    };
    let writes = |ability: &ActivatedAbility| {
        ability
            .program
            .contains(&Instruction::ExileGraveTargetsLinked)
    };
    let reads =
        |ability: &ActivatedAbility| ability.program.contains(&Instruction::ReturnLinkedGhosts);
    (writes(ability) || reads(ability))
        && abilities.iter().filter(|ability| writes(ability)).count() == 1
        && abilities.iter().filter(|ability| reads(ability)).count() == 1
}

impl SlotRow {
    fn matches(&self, other: Self) -> bool {
        self.slot_identity() == other.slot_identity()
    }

    fn slot_identity(self) -> Self {
        Self {
            slot: 0,
            link: None,
            inherited: self
                .inherited
                .map(inherited_activations::Acquisition::slot_identity),
            ..self
        }
    }

    fn shares_link_cohort(&self, other: Self) -> bool {
        self.source == other.source
            && self.grantor == other.grantor
            && self.acquisition == other.acquisition
            && self.graveyard == other.graveyard
            && self.definition == other.definition
            && match (self.inherited, other.inherited) {
                (None, None) => true,
                (Some(left), Some(right)) => left.shares_link_cohort(right),
                _ => false,
            }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn refreshed_slots<W: CharacteristicWorld + ?Sized>(world: &W) -> SlotRows {
    let has_inheritance = inherited_activations::has_provider(world);
    if world.borrowed_slots().is_empty()
        && world.graveyard_borrowing().is_empty()
        && world.game().state.borrowed_links.is_none()
        && !has_inheritance
    {
        return None;
    }
    let mut rows: Vec<_> = world
        .borrowed_slots()
        .iter()
        .copied()
        .filter(|row| {
            world.object(row.source.card).zone == Zone::Battlefield
                && world.reference(row.source.card) == row.source
        })
        .map(|mut row| {
            if let Some(link) = &mut row.link {
                link.live = false;
            }
            row
        })
        .collect();
    for id in source_ids(world, true) {
        for ability in inventory(world, id, InventoryStage::Registry) {
            remember_slot(&mut rows, ability.slot_row());
        }
    }
    if has_inheritance {
        for object in ability_evaluation::evaluate_inherited(world).result.objects {
            for ability in object.inherited {
                remember_slot(&mut rows, ability.slot_row());
            }
        }
    }
    if rows == world.borrowed_slots() {
        return world.borrowed_slots_state().clone();
    }
    (!rows.is_empty()).then(|| rows.into())
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn refresh_inherited_link_continuity(&mut self) {
        if self
            .borrowed_slots()
            .iter()
            .any(|row| row.link.is_some_and(|link| link.live))
        {
            self.state.borrowed_slots = refreshed_slots(self);
        }
    }

    pub(super) fn exile_and_borrow_activated_abilities(&mut self, item: &SpellView) {
        let ids: Vec<_> = item
            .targets
            .iter()
            .filter(|bound| self.same_target_object(**bound))
            .filter_map(|bound| match bound.target {
                Target::Card(id) => Some(id),
                _ => None,
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        for event in &events {
            if event.moved && event.destination == Zone::Exile {
                self.remember_borrowed_exile(item, event.after, false);
            }
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn remember_borrowed_exile(
        &mut self,
        item: &SpellView,
        exiled: ObjectRef,
        shared: bool,
    ) {
        let Some(source) = item.source else {
            return;
        };
        if !self.on_battlefield(source.object.card)
            || self.object_ref(source.object.card) != source.object
        {
            return;
        }
        let width = DONOR_SLOT_WIDTH;
        let first_slot = graveyard_borrowing::next_slot(self, source.object);
        first_slot
            .checked_add(width)
            .capacity("borrowed activation capacity exhausted");
        self.state
            .borrowed_links
            .get_or_insert_default()
            .push(Link {
                source: source.object,
                origin: item.source_ability_origin,
                exiled,
                first_slot,
                width,
                once_per_turn: false,
                shared,
                craft: false,
            });
    }
    pub(super) fn borrowed_descriptor(
        &self,
        card: CardId,
        exiled: ObjectRef,
        index: u16,
    ) -> Descriptor {
        abilities(self, card)
            .into_iter()
            .find(|a| a.inherited.is_none() && a.donor == exiled && a.ability == index)
            .expect("legal borrowed activation")
    }
    pub(super) fn borrowed_activation_item(
        &mut self,
        actor: Player,
        card: CardId,
        exiled: ObjectRef,
        index: u16,
    ) -> SpellView {
        self.refresh_graveyard_borrowing();
        let descriptor = self.borrowed_descriptor(card, exiled, index);
        self.begin_borrowed_activation(actor, descriptor)
    }
    pub(super) fn inherited_activation_item(
        &mut self,
        actor: Player,
        source: ObjectRef,
        slot: u64,
    ) -> SpellView {
        self.state.borrowed_slots = refreshed_slots(self);
        let descriptor = inherited_activations::descriptor(self, source, slot)
            .expect("legal inherited activation");
        self.begin_borrowed_activation(actor, descriptor)
    }
    fn begin_borrowed_activation(&mut self, actor: Player, descriptor: Descriptor) -> SpellView {
        let mut item = self.borrowed_activation_packet(actor, descriptor);
        item.id = self.next_stack_id();
        self.state.passes = 0;
        item
    }
    pub(super) fn borrowed_activation_packet(
        &self,
        actor: Player,
        descriptor: Descriptor,
    ) -> SpellView {
        let mut item = self.activation_packet(actor, descriptor.source.card, descriptor.ability);
        item.attachment_trigger = descriptor.attachment.map(Box::new);
        item.set_source_definition(descriptor.definition);
        if descriptor.inherited.is_some() {
            item.source_quantity_lki = None;
            item.source_transform = descriptor
                .rules()
                .program
                .contains(&Instruction::TransformSource)
                .then(|| self.transform_serial(descriptor.source));
        }
        self.bind_activation_source_quantities(&mut item, descriptor.rules().program);
        item.source_copy_recipe = Some(Box::new(
            self.copiable_recipe(item.source.expect("activated ability has source").object),
        ));
        item.borrowed_slot = Some(descriptor.slot);
        let cohort = descriptor.link_cohort();
        item.acquired_link_cohort = (cohort != descriptor.slot).then_some(cohort);
        item
    }
    pub(super) fn activate_borrowed_mana(
        &mut self,
        mut item: SpellView,
        counters: Counters,
        payment: Mana,
        production: Mana,
    ) {
        if self.start_direct_mana_output(item.clone(), counters.clone(), payment, production) {
            return;
        }
        let source = item.source.expect("activated ability has source");
        let actor = item.controller;
        let quality = self
            .announced_mana_quality(&item, Some(production))
            .map(|(_, quality)| quality);
        if let Some(phase) = self.pay_activation(&mut item, payment, counters) {
            self.suspend_activation_payment(
                item,
                phase,
                ActivationCostRoute::Printed,
                activations::ActivationPaymentCompletion::DirectMana {
                    actor,
                    card: source.id,
                    production,
                    quality,
                },
            );
        } else {
            self.add_mana_tap_event(actor, item.tap_mana_bonus(self), production, quality);
            self.committed_mana_followups(&item);
            self.reopen_evidence_after_mana_sacrifice(source.id);
        }
    }
    pub(super) fn record_borrowed_use(&mut self, source: ObjectRef, slot: u64, lifetime: bool) {
        if !self.state.borrowed_uses.as_ref().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.source == source
                    && row.slot == slot
                    && (row.lifetime || row.turn == self.state.turn)
            })
        }) {
            self.state.borrowed_uses.get_or_insert_default().push(Use {
                source,
                slot,
                turn: self.state.turn,
                lifetime,
            });
        }
    }
    pub(super) fn forget_borrowed_source(&mut self, source: ObjectRef) {
        graveyard_borrowing::forget(&mut self.state.graveyard_borrowing, source);
        if let Some(rows) = &mut self.state.borrowed_slots {
            rows.retain(|row| row.source != source);
            if rows.is_empty() {
                self.state.borrowed_slots = None;
            }
        }
        if let Some(rows) = &mut self.state.borrowed_links {
            rows.retain(|row| row.source != source);
            if rows.is_empty() {
                self.state.borrowed_links = None;
            }
        }
        if let Some(rows) = &mut self.state.borrowed_uses {
            rows.retain(|row| row.source != source);
            if rows.is_empty() {
                self.state.borrowed_uses = None;
            }
        }
    }
    pub(super) fn borrowed_ability_views(&self) -> Vec<BorrowedAbilityView> {
        source_ids(self, false)
            .into_iter()
            .flat_map(|id| abilities(self, id))
            .filter(|a| a.zone == Zone::Exile)
            .map(|a| BorrowedAbilityView {
                source: a.source,
                grantor: a.grantor,
                exiled: a.donor,
                definition: a.definition,
                ability: a.ability,
                slot: a.slot,
                used: a.used,
                acquired_once_per_turn: a.once_per_turn,
            })
            .collect()
    }
    pub(super) fn graveyard_borrowed_ability_views(
        &self,
    ) -> Option<Vec<GraveyardBorrowedAbilityView>> {
        let rows: Vec<_> = self
            .state
            .static_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.state.objects[id as usize]
                    .card
                    .graveyard_activation_subtype()
                    .is_some()
            })
            .flat_map(|id| abilities(self, id))
            .filter(|a| a.zone == Zone::Graveyard)
            .map(|a| GraveyardBorrowedAbilityView {
                source: a.source,
                graveyard: a.donor,
                definition: a.definition,
                ability: a.ability,
                slot: a.slot,
                used: a.used,
            })
            .collect();
        (!rows.is_empty()).then_some(rows)
    }
    pub(super) fn visit_borrowed_activations(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if (!self.state.objects[card as usize].card.borrows_activations()
            && !self.state.objects[card as usize]
                .card
                .inherits_creature_activations()
            && !has_shared_links(self))
            || CharacteristicWorld::activation_prohibited(self, actor, card)
        {
            return false;
        }
        let object = &self.state.objects[card as usize];
        let descriptors = abilities(self, card);
        let loyalty_used = descriptors
            .iter()
            .any(|a| a.rules().cost.loyalty.is_some() && a.used);
        let mut ready_object = object.clone();
        ready_object.used_abilities = 0;
        for descriptor in &descriptors {
            let ability = descriptor.rules();
            let readiness = self.activation_readiness(actor, ability);
            if activations::pure_loyalty_placement_prohibited(self, descriptor.source, &readiness) {
                continue;
            }
            if ability.cost.loyalty.is_some() && loyalty_used {
                continue;
            }
            if !self.power_up_available(
                descriptor.source,
                descriptor.definition,
                descriptor.ability,
                Some(descriptor.slot),
            ) || descriptor.used && (descriptor.once_per_turn || readiness.has_use_limit())
            {
                continue;
            }
            if (ability.cost.sacrifice && CharacteristicWorld::sacrifice_prohibited(self, card))
                || !self.activation_guard_available(ability.guard, card, actor)
                || (descriptor.mana_production().is_none()
                    && (self.nonmana_actions_prohibited()
                        || self.named_activation_prohibited(card)))
                || !Self::object_activation_available_with_kinds(
                    &ready_object,
                    self.counter_amounts(card),
                    self.current_kinds(card),
                    || CharacteristicWorld::activation_keywords(self, card),
                    self.state.players[actor.index()].life,
                    &readiness,
                    0,
                    actor,
                    self.loyalty_timing(card, ability, actor, sorcery),
                )
            {
                continue;
            }
            if self.visit_activation_counter_payments(
                actor,
                ability,
                self.counter_amounts(card),
                |counter_payment| {
                    if let Some(production) = descriptor.mana_production() {
                        let packet = self.borrowed_activation_packet(actor, *descriptor);
                        return self.visit_activation_mana_outputs(
                            production,
                            card,
                            descriptor.definition,
                            descriptor.ability,
                            |production| {
                                self.mana_packet_fundable(
                                    &packet,
                                    counter_payment.clone(),
                                    production,
                                ) && emit(descriptor.mana_action(
                                    counter_payment.clone(),
                                    None,
                                    production,
                                ))
                            },
                        );
                    }
                    if descriptor.inherited.is_some() {
                        let packet = self.borrowed_activation_packet(actor, *descriptor);
                        let targets_available = ability.declares_target_value_x()
                            || ability.modes().map_or_else(
                                || self.item_targets_available(&packet, self.target_spec(&packet)),
                                |modes| {
                                    modes.iter().enumerate().any(|(index, mode)| {
                                        self.item_mode_eligible(&packet, index)
                                            && self.modal_choice_unused(&packet, index)
                                            && self.item_targets_available(&packet, mode.targets)
                                    })
                                },
                            );
                        return targets_available
                            && self.initial_activation_packet_target_fundable(
                                &packet,
                                &counter_payment,
                            )
                            && emit(Action::AnnounceInheritedAbility {
                                source: descriptor.source,
                                slot: descriptor.slot,
                                counter_payment,
                            });
                    }
                    let fundable = self
                        .activation_mana_costs(card, descriptor.definition, descriptor.ability, 0)
                        .any(|cost| {
                            self.initial_activation_target_fundable(
                                FundingGoal {
                                    actor,
                                    required: cost.colored,
                                    generic: cost.generic,
                                    reserve: None,
                                    source: Some((
                                        descriptor.source,
                                        descriptor.ability,
                                        counter_payment.clone(),
                                        descriptor.definition,
                                        Some(descriptor.slot),
                                    )),
                                },
                                ability.targets,
                            )
                        });
                    let modes = ability.modes();
                    let targets_available = ability.declares_target_value_x()
                        || modes.map_or_else(
                            || self.targets_available(ability.targets, actor, descriptor.source),
                            |modes| {
                                let packet = self.borrowed_activation_packet(actor, *descriptor);
                                modes.iter().enumerate().any(|(mode_index, mode)| {
                                    self.modal_choice_unused(&packet, mode_index)
                                        && self.targets_available(
                                            mode.targets,
                                            actor,
                                            descriptor.source,
                                        )
                                })
                            },
                        );
                    if !fundable || !targets_available {
                        return false;
                    }
                    emit(if descriptor.zone == Zone::Graveyard {
                        Action::AnnounceGraveyardBorrowedAbility {
                            card,
                            graveyard: descriptor.donor,
                            ability: descriptor.ability,
                            counter_payment,
                        }
                    } else {
                        Action::AnnounceBorrowedAbility {
                            card,
                            exiled: descriptor.donor,
                            ability: descriptor.ability,
                            counter_payment,
                        }
                    })
                },
            ) {
                return true;
            }
        }
        false
    }
}
