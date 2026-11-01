//! Rare failed proposals caused by another player's mandatory entry choices.
//! The borrowed funding solver never owns this checkpoint or clones a Game.
//! Failed proposals offer selective legal mana-ability reversal (CR733.1),
//! preserving irreversible library actions and the completed events that remain.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CastRollback {
    pub(super) before: State,
    pub(super) player: Player,
    pub(super) spell: CardId,
    foreign_choice: bool,
    bytes: usize,
    pub(super) events: Vec<cast_rollback_mana::ManaEvent>,
    pub(super) replay: Option<Box<cast_rollback_mana::ManaReplay>>,
    pub(super) capture: Option<Box<cast_mana_recording::ManaCapture>>,
    pub(super) public_hand_cards: Vec<CardId>,
    pub(super) obscured_hand_cards: Vec<CardId>,
    pub(super) public_face_down: Vec<ObjectRef>,
    pub(super) obscured_face_down: Vec<ObjectRef>,
}
impl CastRollback {
    fn new(before: State, player: Player, spell: CardId) -> Self {
        // A wrapper reuses the exact initialized-state accounting without cloning
        // the already owned checkpoint or allocating a second game tree.
        let checkpoint = Game {
            state: before,
            revision: 0,
        };
        let bytes =
            checkpoint.state_bytes() + std::mem::size_of::<Self>() - std::mem::size_of::<State>();
        Self {
            before: checkpoint.state,
            player,
            spell,
            foreign_choice: false,
            bytes,
            events: Vec::new(),
            replay: None,
            capture: None,
            public_hand_cards: Vec::new(),
            obscured_hand_cards: Vec::new(),
            public_face_down: Vec::new(),
            obscured_face_down: Vec::new(),
        }
    }
    pub(super) fn bytes(&self) -> usize {
        self.bytes
            + (self.public_hand_cards.len() + self.obscured_hand_cards.len())
                * std::mem::size_of::<CardId>()
            + (self.public_face_down.len() + self.obscured_face_down.len())
                * std::mem::size_of::<ObjectRef>()
            + self.events.iter().map(|e| e.bytes()).sum::<usize>()
            + self.replay.as_ref().map_or(0, |r| r.bytes())
            + self.capture.as_ref().map_or(0, |r| r.bytes())
    }
}
impl Game {
    // Record public knowledge while the checkpoint incarnation is still in hand.
    pub(super) fn remember_public_hand(&mut self, reference: ObjectRef) {
        if let Some(rollback) = &mut self.state.cast_rollback
            && rollback
                .before
                .objects
                .get(reference.card as usize)
                .is_some_and(|o| o.zone == Zone::Hand && o.generation == reference.generation)
            && !rollback.obscured_hand_cards.contains(&reference.card)
            && !rollback.public_hand_cards.contains(&reference.card)
        {
            rollback.public_hand_cards.push(reference.card);
        }
        if !self.state.known_hand.contains(&reference) {
            self.state.known_hand.push(reference);
        }
    }
    pub(super) fn remember_revealed_disguise(&mut self, id: CardId) {
        if let Some(rollback) = &mut self.state.cast_rollback
            && rollback
                .before
                .objects
                .get(id as usize)
                .is_some_and(|o| matches!(o.zone, Zone::Hand | Zone::Library))
            && !rollback.obscured_hand_cards.contains(&id)
            && !rollback.public_hand_cards.contains(&id)
        {
            rollback.public_hand_cards.push(id);
        }
    }
    pub(super) fn restore_public_hand_knowledge(
        &mut self,
        current: &State,
        cards: &[CardId],
        obscured: &[CardId],
    ) {
        self.state
            .known_hand
            .retain(|reference| !obscured.contains(&reference.card));
        let mut public = cards.to_vec();
        for reference in &current.known_hand {
            if !obscured.contains(&reference.card)
                && self
                    .state
                    .objects
                    .get(reference.card as usize)
                    .is_some_and(|o| o.zone == Zone::Hand && o.generation == reference.generation)
                && !public.contains(&reference.card)
            {
                public.push(reference.card);
            }
        }
        for id in public {
            if obscured.contains(&id) {
                continue;
            }
            let Some(object) = self.state.objects.get(id as usize) else {
                continue;
            };
            let owner = object.owner;
            if object.zone == Zone::Hand {
                let reference = self.object_ref(id);
                if !self.state.known_hand.contains(&reference) {
                    self.state.known_hand.push(reference);
                }
            } else if object.zone == Zone::Library
                && self.state.players[owner.index()].library.last() == Some(&id)
                // A hidden library movement could have obscured the exact card.
                && current.objects.get(id as usize).is_some_and(|object| object.zone != Zone::Library)
            {
                self.remember_library_top(owner, self.object_ref(id));
            }
        }
    }
    // Face-down exile is marked after the zone move, so sample completed
    // actions rather than assuming an exile destination is public at commit.
    pub(super) fn capture_public_hand_exiles(&mut self) {
        let Some(rollback) = self.state.cast_rollback.as_ref() else {
            return;
        };
        let public = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let original = rollback.before.objects.get(id)?;
                (original.zone == Zone::Hand
                    && !rollback.obscured_hand_cards.contains(&(id as CardId))
                    && object.zone == Zone::Exile
                    && self
                        .can_view_hideaway(original.owner.other(), self.object_ref(id as CardId)))
                .then_some(id as CardId)
            })
            .collect::<Vec<_>>();
        let rollback = self.state.cast_rollback.as_mut().unwrap();
        for id in public {
            if !rollback.public_hand_cards.contains(&id) {
                rollback.public_hand_cards.push(id);
            }
        }
    }
    // Once an original hand card enters a hidden library, its physical ID is
    // no longer public evidence that later objects are the same card.
    pub(super) fn obscure_rollback_hand_card(&mut self, id: CardId) {
        let Some(rollback) = self.state.cast_rollback.as_mut() else {
            return;
        };
        if rollback
            .before
            .objects
            .get(id as usize)
            .is_some_and(|o| matches!(o.zone, Zone::Hand | Zone::Library))
        {
            rollback.public_hand_cards.retain(|&known| known != id);
            if !rollback.obscured_hand_cards.contains(&id) {
                rollback.obscured_hand_cards.push(id);
            }
        }
    }
    pub(super) fn begin_attack_rollback(&mut self, actor: Player) {
        assert!(self.state.cast_rollback.is_none());
        self.state.cast_rollback = Some(Box::new(CastRollback::new(
            self.state.clone(),
            actor,
            CardId::MAX,
        )));
    }
    pub(super) fn prepare_cast_rollback(&mut self, actor: Player, action: &Action) {
        self.capture_public_hand_exiles();
        if let Some(rollback) = &mut self.state.cast_rollback {
            if actor != rollback.player
                && matches!(
                    action,
                    Action::ChooseEntryBehold { .. }
                        | Action::ChooseEntryValue { .. }
                        | Action::ChooseEntryLife { .. }
                        | Action::ChooseAuraAttachment { .. }
                        | Action::ChooseEntryReplacement { .. }
                        | Action::ChooseEntryTimestamp { .. }
                        | Action::ChooseReplacement { .. }
                        | Action::ChooseDrawReplacement { .. }
                        | Action::ChooseSelfCounterReplacement { .. }
                        | Action::ChooseZoneReplacement { .. }
                )
            {
                rollback.foreign_choice = true;
            }
            return;
        }
        if let Action::BeginFaceUp { object, .. } = *action {
            let mut rollback = CastRollback::new(self.state.clone(), actor, CardId::MAX);
            rollback.public_face_down.push(object);
            self.state.cast_rollback = Some(Box::new(rollback));
            return;
        }
        let card = match *action {
            Action::AnnounceCast { card }
            | Action::AnnounceExile { card }
            | Action::AnnounceAirbend { card, .. }
            | Action::AnnounceExileLife { card, .. }
            | Action::AnnounceExileFree { card, .. }
            | Action::AnnounceFlashSurcharge { card }
            | Action::AnnounceWarp { card }
            | Action::AnnounceDisguise { card }
            | Action::AnnounceDisguiseGraveyard { card, .. }
            | Action::AnnounceSneak { card, .. }
            | Action::AnnounceAdventure { card }
            | Action::AnnounceOmen { card }
            | Action::AnnounceBack { card }
            | Action::AnnouncePlotted { card }
            | Action::AnnounceHarmonize { card }
            | Action::AnnounceMayhem { card }
            | Action::AnnounceFlashback { card }
            | Action::AnnounceGrantedFlashback { card, .. }
            | Action::AnnounceFree { card, .. }
            | Action::AnnounceFreeFromPermission { card, .. }
            | Action::AnnounceCounterGraveyard { card }
            | Action::Cast { card, .. }
            | Action::BeginCast { card, .. } => card,
            Action::CastExiledDuringResolution { object, .. } => object.card,
            Action::PlayGraveyardPermanent { card, .. }
                if !self.state.objects[card as usize].card.is_land() =>
            {
                card
            }
            _ => return,
        };
        // Foreign entry choices currently arise from immediate linked returns.
        // Allocate no checkpoint when no opponent-owned return is possible.
        if !self.state.exile_links.iter().any(|link| {
            link.return_zone == Zone::Battlefield
                && self.object_ref(link.exiled.card) == link.exiled
                && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                && self.state.objects[link.exiled.card as usize].owner != actor
        }) {
            return;
        }
        self.state.cast_rollback =
            Some(Box::new(CastRollback::new(self.state.clone(), actor, card)));
    }
    pub(super) fn mark_foreign_cast_choice(&mut self) {
        if let Some(r) = &mut self.state.cast_rollback {
            r.foreign_choice = true;
        }
    }
    pub(super) fn foreign_cast_choice_pending(&self) -> bool {
        self.state
            .cast_rollback
            .as_ref()
            .is_some_and(|r| r.foreign_choice)
    }
    pub(super) fn finish_cast_rollback(&mut self, fundable: bool) {
        self.capture_public_hand_exiles();
        if !fundable {
            let Some(rollback) = self.state.cast_rollback.take() else {
                return;
            };
            if !rollback.events.is_empty() {
                let current = self.state.clone();
                assert!(self.start_mana_rollback(rollback, current));
                return;
            }
            let current = std::mem::replace(&mut self.state, rollback.before);
            self.complete_cast_rollback_restore(
                &current,
                rollback.spell,
                &rollback.public_hand_cards,
                &rollback.obscured_hand_cards,
                &rollback.public_face_down,
                &rollback.obscured_face_down,
            );
            return;
        }
        if let Some(r) = &mut self.state.cast_rollback {
            r.foreign_choice = false;
        }
    }
    pub(super) fn complete_cast_rollback_restore(
        &mut self,
        current: &State,
        spell: CardId,
        public_hand_cards: &[CardId],
        obscured_hand_cards: &[CardId],
        public_face_down: &[ObjectRef],
        obscured_face_down: &[ObjectRef],
    ) {
        self.preserve_irreversible_cast_library_changes(current, spell);
        self.restore_public_hand_knowledge(current, public_hand_cards, obscured_hand_cards);
        self.restore_public_face_down(current, public_face_down, obscured_face_down);
        // Undone proposal triggers disappear; retain the original priority holder.
        if self.state.resolving.is_none() && self.state.pending == Pending::Priority {
            self.state_based_actions();
            self.prepare_triggers();
        }
    }
    pub(super) fn clear_finished_cast_rollback(&mut self) {
        if self.state.cast_rollback.is_none()
            || matches!(self.state.pending, Pending::ManaRollback { .. })
        {
            return;
        }
        let casting = self
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.item.kind == StackKind::Spell)
            || self
                .state
                .mana_announcement_stack
                .as_ref()
                .is_some_and(|frames| {
                    frames
                        .iter()
                        .any(|f| f.announcement.item.kind == StackKind::Spell)
                });
        if (!casting && self.state.attack_payment.is_none() && self.face_up_payment().is_none())
            || self.outcome().is_some()
        {
            self.state.cast_rollback = None;
        }
    }
    fn preserve_irreversible_cast_library_changes(&mut self, current: &State, spell: CardId) {
        let changed = current
            .players
            .iter()
            .zip(&self.state.players)
            .any(|(a, b)| {
                a.library
                    .iter()
                    .filter(|&&id| id != spell)
                    .ne(b.library.iter().filter(|&&id| id != spell))
            })
            || current.objects.iter().enumerate().any(|(id, o)| {
                o.zone == Zone::Library
                    && self
                        .state
                        .objects
                        .get(id)
                        .is_none_or(|old| old.zone != o.zone || old.generation != o.generation)
            });
        if !changed {
            return;
        }
        // Supported irreversible cost movements are self-shuffles from hand or
        // battlefield. Replay only their silent zone commits, using the normal
        // exact-reference cleanup; never dispatch the undone cast's observers.
        let retained =
            current
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, o)| {
                    (o.zone == Zone::Library
                        && self.state.objects.get(id).is_some_and(|old| {
                            old.zone != o.zone || old.generation != o.generation
                        }))
                    .then_some((id as CardId, o.clone()))
                })
                .collect::<Vec<_>>();
        let restored_spell = self
            .state
            .players
            .iter()
            .enumerate()
            .find_map(|(p, player)| {
                player
                    .library
                    .iter()
                    .position(|&id| id == spell)
                    .map(|at| (p, player.library.len() - 1 - at))
            });
        let before: Vec<_> = retained
            .iter()
            .map(|(id, _)| self.object_view(*id))
            .collect();
        self.capture_event_object_facts(before.iter());
        self.capture_source_quantities(before.iter());
        for (id, object) in retained {
            let before = self.object_view(id);
            if before.zone != Zone::Library {
                self.commit_planned_zone_move(
                    before,
                    zone_replacements::Plan {
                        destination: Zone::Library,
                        prevented: false,
                        shuffle: false,
                        reflexive: None,
                    },
                    false,
                    Counters::default(),
                    EntryIdentity::default(),
                    ZoneObservers::default(),
                );
            }
            let temporary = self.object_ref(id);
            self.clear_copy_recipe(temporary);
            self.state.objects[id as usize] = object;
            let reference = self.object_ref(id);
            let counters = live_counters::amounts(
                &current.named_counters,
                reference,
                current.objects[id as usize].counters,
            )
            .to_owned();
            self.set_counters(id, counters);
            if let Some(recipe) = current.copy_recipes.as_ref().and_then(|rows| {
                rows.iter()
                    .find(|r| r.object == reference)
                    .map(|r| r.recipe)
            }) {
                self.set_copy_recipe(reference, recipe);
            }
        }
        for (player, now) in self.state.players.iter_mut().zip(&current.players) {
            player.library = now.library.clone();
        }
        // Library-to-stack casting can reverse even when a later shuffle cannot.
        // Preserve the restored spell's original library position (currently top).
        if let Some((p, distance_from_top)) = restored_spell {
            let library = &mut self.state.players[p].library;
            if !library.contains(&spell) {
                let at = library.len().saturating_sub(distance_from_top);
                library.insert(at, spell);
            }
        }
        self.state.rng = current.rng;
        zone_replacements::preserve_irreversible_reveals(
            &mut self.state.zone_rules,
            current.zone_rules.as_deref(),
        );
        self.refresh_controllers();
        self.prune_linked_exiles();
    }
}
