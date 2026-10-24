//! Warp is an alternative cast, a counterable next-end-step exile, then a paid
//! permission for the exact card exiled by that ability (CR702.185/608.3g).
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct WarpedCard {
    pub object: ObjectRef,
    pub owner: Player,
    /// The turn of actual exile, which must end before this permission is usable.
    pub exiled_turn: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedWarpExileView {
    pub id: u64,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedWarp {
    pub view: DelayedWarpExileView,
    keywords: Keywords,
    tapped: bool,
    facts: DamageFacts,
    attachment: Option<ObjectRef>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct WarpState {
    pub delayed: Vec<DelayedWarp>,
    pub cards: Vec<WarpedCard>,
}
impl WarpState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.delayed.len() * std::mem::size_of::<DelayedWarp>()
            + self.cards.len() * std::mem::size_of::<WarpedCard>()
    }
}
impl Game {
    pub(super) fn warp_cast_fundable(&self, actor: Player, card: CardId, cost: ManaCost) -> bool {
        let plan = match self.state.objects[card as usize].card.effect() {
            Effect::Spell(plan, _) | Effect::Aura(plan) => plan,
            _ => TargetPlan::NONE,
        };
        self.cast_targets_available(plan, actor, self.object_ref(card))
            && self.spell_can_fund(actor, card, cost)
    }

    /// Called by the actual atomic entry commit, before its ordinary ETB observers.
    /// A countered spell never reaches this path. The exact entering incarnation
    /// remains the delayed ability's subject even when its source later leaves.
    pub(super) fn register_warp_exile(&mut self, event: &ZoneChange, controller: Player) {
        let source = self.object_view(event.after.card);
        let registration = DelayedWarp {
            view: DelayedWarpExileView {
                id: self.next_stack_id(),
                source: event.after,
                source_card: source.card,
                controller,
            },
            keywords: source.keywords,
            tapped: source.tapped,
            facts: self.damage_facts(event.after.card),
            attachment: source.attached_to,
        };
        let state = self.state.warp.get_or_insert_with(Default::default);
        state
            .delayed
            .try_reserve(1)
            .capacity("Warp delayed allocation exhausted");
        state.delayed.push(registration);
    }
    pub(super) fn queue_warp_exiles(&mut self) {
        let Some(state) = self.state.warp.as_mut() else {
            return;
        };
        let delayed = std::mem::take(&mut state.delayed);
        if state.cards.is_empty() {
            self.state.warp = None;
        }
        self.state
            .pending_triggers
            .try_reserve(delayed.len())
            .capacity("Warp trigger allocation exhausted");
        for d in delayed {
            let view = d.view;
            let id = self.next_stack_id();
            self.state.pending_triggers.push(SpellView {
                source_move: None,
                borrowed_slot: None,
                target_controller_lki: None,
                gift: None,
                reflexive_subject: None,
                delayed_warp_exile: Some(Box::new(view)),
                return_link: None,
                crew: None,
                cast_event: None,
                copy_event: None,
                captured_artifact_copy: None,
                source_copy_recipe: None,
                target_allocations: None,
                mana_copy_sources: None,
                is_copy: false,
                delayed_sacrifice: None,
                x: None,
                attachment_trigger: None,
                cast: None,
                entering_object: None,
                evidence_collected: false,
                id,
                kind: StackKind::DelayedWarpExile,
                source_card: view.source_card,
                source_keywords: d.keywords,
                source_tapped: d.tapped,
                source_attachment: d.attachment,
                source_transform: None,
                source_facts: d.facts,
                related_stack: None,
                damage_event: None,
                damage_received: None,
                creature_combat_event: None,
                discard_event: None,
                draw_event: None,
                life_event: None,
                source_counter_lki: None,
                sacrificed_toughness: None,
                card: view.source.card,
                object: view.source,
                targets: Vec::new(),
                mode: None,
                controller: view.controller,
            });
        }
    }
    pub(super) fn exile_warped_permanent(&mut self, item: &SpellView) {
        let subject = item
            .delayed_warp_exile
            .as_ref()
            .expect("Warp delayed context")
            .source;
        let Some(object) = self
            .state
            .objects
            .get(subject.card as usize)
            .filter(|o| o.zone == Zone::Battlefield && o.generation == subject.generation)
        else {
            return;
        };
        let owner = object.owner;
        let is_card = !object.is_token();
        let event = self.move_object(subject.card, Zone::Exile);
        if is_card && event.moved && event.destination == Zone::Exile {
            let state = self.state.warp.get_or_insert_with(Default::default);
            state
                .cards
                .try_reserve(1)
                .capacity("Warp permission allocation exhausted");
            let at = state
                .cards
                .partition_point(|p| p.object.card < event.after.card);
            state.cards.insert(
                at,
                WarpedCard {
                    object: event.after,
                    owner,
                    exiled_turn: self.state.turn,
                },
            );
        }
    }
    pub(super) fn can_cast_warped(&self, player: Player, card: CardId) -> bool {
        let Some(state) = &self.state.warp else {
            return false;
        };
        let Ok(i) = state.cards.binary_search_by_key(&card, |p| p.object.card) else {
            return false;
        };
        let p = state.cards[i];
        self.state.objects.get(card as usize).is_some_and(|o| {
            p.owner == player
                && o.owner == player
                && o.zone == Zone::Exile
                && o.generation == p.object.generation
                && self.state.turn > p.exiled_turn
                && !o.card.is_land()
                && !o.is_token()
                && o.card.is_supported()
        })
    }
    pub(super) fn remove_warped(&mut self, object: ObjectRef) {
        if let Some(state) = self.state.warp.as_mut() {
            state.cards.retain(|p| p.object != object);
            if state.cards.is_empty() && state.delayed.is_empty() {
                self.state.warp = None;
            }
        }
    }
}
