//! Owned spell and activation attempts, including nested mana announcements.
//! The borrowed funding solver never owns this checkpoint or clones a Game.
//! Failed proposals offer selective legal mana-ability reversal (CR733.1),
//! preserving irreversible library actions and the completed events that remain.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CastRollback {
    pub(super) before: State,
    /// Parents are taken out before the checkpoint is copied.
    pub(super) parent: Option<Box<CastRollback>>,
    pub(super) attempt_id: u64,
    pub(super) context: Option<u64>,
    pub(super) source: Option<ObjectRef>,
    pub(super) ability: Option<u16>,
    pub(super) turn: u64,
    pub(super) mana_closed: bool,
    pub(super) contributions_sealed: bool,
    pub(super) special: bool,
    pub(super) costs_complete: bool,
    pub(super) deferred_draws: Vec<attempts::DeferredAttemptDraw>,
    pub(super) failure: Option<AttemptFailureReason>,
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
            parent: None,
            attempt_id: 0,
            context: None,
            source: None,
            ability: None,
            turn: 0,
            mana_closed: false,
            contributions_sealed: false,
            special: false,
            costs_complete: false,
            deferred_draws: Vec::new(),
            failure: None,
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
            + self.parent.as_ref().map_or(0, |parent| parent.bytes())
            + self
                .deferred_draws
                .iter()
                .map(attempts::DeferredAttemptDraw::bytes)
                .sum::<usize>()
            + (self.public_hand_cards.len() + self.obscured_hand_cards.len())
                * std::mem::size_of::<CardId>()
            + (self.public_face_down.len() + self.obscured_face_down.len())
                * std::mem::size_of::<ObjectRef>()
            + self.events.iter().map(|e| e.bytes()).sum::<usize>()
            + self.replay.as_ref().map_or(0, |r| r.bytes())
            + self.capture.as_ref().map_or(0, |r| r.bytes())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    // Record public knowledge while the checkpoint incarnation is still in hand.
    pub(super) fn remember_public_hand(&mut self, reference: ObjectRef) {
        if self.defer_attempt_draw_reveal(reference) {
            return;
        }
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
    // A private choice may move any of the choosing player's hand cards.
    // Keeping a public reference for an unchosen card would disclose that choice.
    pub(super) fn obscure_private_hand_selection(&mut self, player: Player) {
        self.obscure_private_hand_selection_matching(player, |_| true);
    }

    pub(super) fn obscure_private_hand_selection_matching(
        &mut self,
        player: Player,
        matches: impl Fn(Card) -> bool,
    ) {
        if self.state.cast_rollback.is_some() {
            let candidates: Vec<_> = self
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, object)| {
                    (object.zone == Zone::Hand
                        && object.owner == player
                        && object.is_card()
                        && matches(object.card))
                    .then_some(id as CardId)
                })
                .collect();
            for id in candidates {
                self.obscure_rollback_hand_card(id);
            }
        }
        let objects = &self.state.objects;
        self.state.known_hand.retain(|reference| {
            objects.get(reference.card as usize).is_some_and(|object| {
                object.generation == reference.generation
                    && (object.owner != player || !matches(object.card))
            })
        });
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
    pub(super) fn begin_combat_rollback(&mut self, actor: Player) {
        assert!(self.state.cast_rollback.is_none());
        self.open_attempt_checkpoint(actor, CardId::MAX);
        self.state.cast_rollback.as_mut().unwrap().special = true;
    }
    pub(super) fn prepare_cast_rollback(&mut self, actor: Player, action: &Action) {
        self.capture_public_hand_exiles();
        if self.state.cast_rollback.is_some()
            && let Some(card) = self.attempt_action_card(action)
        {
            self.open_attempt_checkpoint(actor, card);
            self.capture_attempt_origin(action);
            return;
        }
        if let Some(rollback) = &mut self.state.cast_rollback {
            if actor != rollback.player
                && matches!(
                    action,
                    Action::ChooseEntryCopy { .. }
                        | Action::ChooseEntryExile { .. }
                        | Action::FinishEntryExile
                        | Action::ChooseEntryDevour { .. }
                        | Action::FinishEntryDevour
                        | Action::ChooseEntryRiot { .. }
                        | Action::ChooseEntryBehold { .. }
                        | Action::ChooseEntryValue { .. }
                        | Action::ChooseEntryLife { .. }
                        | Action::ChooseAuraAttachment { .. }
                        | Action::ChooseAuraPlayerAttachment { .. }
                        | Action::ChooseEntryReplacement { .. }
                        | Action::ChooseEntryTimestamp { .. }
                        | Action::ChooseReplacement { .. }
                        | Action::ChooseResolvedDamageReplacement { .. }
                        | Action::ChooseResolvedCounterReplacement { .. }
                        | Action::ChooseDrawReplacement { .. }
                        | Action::ChooseSelfCounterReplacement { .. }
                        | Action::ChooseZoneReplacement { .. }
                        | Action::ChooseMeldOrder { .. }
                )
            {
                rollback.foreign_choice = true;
            }
            return;
        }
        if let Action::BeginFaceUp { object, .. } | Action::BeginFaceUpX { object, .. } = *action {
            self.open_attempt_checkpoint(actor, CardId::MAX);
            let rollback = self.state.cast_rollback.as_mut().unwrap();
            rollback.special = true;
            rollback.public_face_down.push(object);
            return;
        }
        if matches!(
            action,
            Action::BeginRoomUnlock { .. }
                | Action::AnnounceForetell { .. }
                | Action::AnnouncePlot { .. }
                | Action::AnnounceLibraryTopPlot { .. }
        ) {
            self.open_attempt_checkpoint(actor, CardId::MAX);
            self.state.cast_rollback.as_mut().unwrap().special = true;
            return;
        }
        if let Some(card) = self.attempt_action_card(action) {
            self.open_attempt_checkpoint(actor, card);
            self.capture_attempt_origin(action);
            if card != CardId::MAX
                && self.state.objects[card as usize].zone == Zone::Hand
                && !matches!(
                    action,
                    Action::AnnounceDisguise { .. } | Action::AnnounceDisguiseGraveyard { .. }
                )
            {
                self.state
                    .cast_rollback
                    .as_mut()
                    .unwrap()
                    .public_hand_cards
                    .push(card);
            }
        }
    }
    fn attempt_action_card(&self, action: &Action) -> Option<CardId> {
        match *action {
            Action::AnnounceCast { card }
            | Action::AnnounceExile { card }
            | Action::AnnounceAirbend { card, .. }
            | Action::AnnounceExileLife { card, .. }
            | Action::AnnounceLibraryLife { card, .. }
            | Action::AnnounceExileFree { card, .. }
            | Action::AnnounceLinkedFree { card, .. }
            | Action::AnnounceLinkedExileCounterCost { card, .. }
            | Action::AnnounceFlashSurcharge { card }
            | Action::AnnounceGraveyardFlashSurcharge { card }
            | Action::AnnounceWarp { card }
            | Action::AnnounceGrantedWarp { card, .. }
            | Action::AnnounceDisguise { card }
            | Action::AnnounceDisguiseGraveyard { card, .. }
            | Action::AnnounceSneak { card, .. }
            | Action::AnnounceWebSlinging { card, .. }
            | Action::AnnounceBeamMeUp { card, .. }
            | Action::AnnounceAdventure { card }
            | Action::AnnounceOmen { card }
            | Action::AnnounceRoom { card, .. }
            | Action::AnnounceSplit { card, .. }
            | Action::AnnounceGraveyardRoom { card, .. }
            | Action::AnnounceBack { card }
            | Action::AnnounceGraveyardBack { card, .. }
            | Action::AnnouncePlotted { card }
            | Action::AnnouncePlottedFace { card, .. }
            | Action::AnnounceHarmonize { card }
            | Action::AnnounceGrantedHarmonize { card, .. }
            | Action::AnnounceMayhem { card }
            | Action::AnnounceGrantedMayhem { card, .. }
            | Action::AnnounceFlashback { card }
            | Action::AnnounceGrantedFlashback { card, .. }
            | Action::AnnounceGrantedZeroFlashback { card, .. }
            | Action::AnnounceGrantedFixedFlashback { card, .. }
            | Action::AnnounceFree { card, .. }
            | Action::AnnounceFreeCounterGraveyard { card, .. }
            | Action::AnnounceFreeGraveyardPermanent { card, .. }
            | Action::AnnounceFreeGraveyardAdditionalCosts { card, .. }
            | Action::AnnounceFreeFromPermission { card, .. }
            | Action::AnnounceCounterGraveyard { card }
            | Action::AnnounceGraveyardAdditionalCosts { card }
            | Action::AnnounceGraveyardFinality { card, .. }
            | Action::AnnounceSourceGraveyard { card, .. }
            | Action::Cast { card, .. }
            | Action::BeginCast { card, .. } => card,
            Action::AnnounceForetold { object, .. }
            | Action::AnnounceLinkedAlternative { object, .. }
            | Action::CastExiledDuringResolution { object, .. }
            | Action::CastFreeDuringResolution { object, .. }
            | Action::AnnounceTaxedExile { object, .. }
            | Action::AnnounceExileForZero { object, .. }
            | Action::AnnounceCastWithManaAlternative { object, .. }
            | Action::AnnounceCastWithEvidenceAlternative { object, .. }
            | Action::AnnounceLinkedPaid { object, .. }
            | Action::AnnounceLinkedWaterbend { object, .. }
            | Action::AnnounceGraveyardPaid { object, .. }
            | Action::AnnounceLibraryTop { object, .. }
            | Action::AnnounceSelfGraveyardWithCounter { object, .. }
            | Action::AnnounceGraveyardWithLife { object, .. } => object.card,
            Action::AnnounceSelectedOutsidePermanent { .. } => {
                self.state.outside_play.as_ref().unwrap().object.card
            }
            Action::PlayGraveyardPermanent { card, .. }
            | Action::PlaySourceGraveyardLand { card, .. }
                if !self.state.objects[card as usize].card.is_land() =>
            {
                card
            }
            Action::AnnounceActivate { .. }
            | Action::AnnounceMana { .. }
            | Action::AnnounceGrantedMana { .. }
            | Action::AnnounceGrantedManaTemplate { .. }
            | Action::AnnounceAttachedAbility { .. }
            | Action::AnnouncePlaneswalkerGrant { .. }
            | Action::AnnounceArtifactFood { .. }
            | Action::AnnounceCreatureGrant { .. }
            | Action::AnnounceRetainedCopyAbility { .. }
            | Action::AnnounceEmbalm { .. }
            | Action::AnnounceSourceGrant { .. }
            | Action::AnnounceSelfGrantedAbility { .. }
            | Action::AnnounceSelfGrantedMana { .. }
            | Action::AnnounceBorrowedAbility { .. }
            | Action::AnnounceBorrowedMana { .. }
            | Action::AnnounceGraveyardBorrowedAbility { .. }
            | Action::AnnounceGraveyardBorrowedMana { .. }
            | Action::AnnounceInheritedAbility { .. }
            | Action::AnnounceInheritedMana { .. }
            | Action::Activate { .. }
            | Action::ActivateMana { .. }
            | Action::ActivateBorrowedMana { .. }
            | Action::ActivateGraveyardBorrowedMana { .. }
            | Action::ActivateInheritedMana { .. }
            | Action::BeginActivate { .. } => CardId::MAX,
            _ => return None,
        }
        .into()
    }
    fn open_attempt_checkpoint(&mut self, actor: Player, spell: CardId) {
        let parent = self.state.cast_rollback.take();
        let mut rollback = CastRollback::new(self.state.clone(), actor, spell);
        rollback.parent = parent;
        rollback.turn = u64::from(self.state.turn);
        rollback.source = (spell != CardId::MAX).then(|| self.object_ref(spell));
        rollback.attempt_id = self
            .revision
            .checked_add(1)
            .capacity("attempt identity exhausted");
        self.state.cast_rollback = Some(Box::new(rollback));
        self.state.last_attempt_outcome = None;
    }
    fn capture_attempt_origin(&mut self, action: &Action) {
        let (source, ability) = match *action {
            Action::AnnounceActivate { card, ability, .. }
            | Action::AnnounceMana { card, ability, .. }
            | Action::AnnounceBorrowedAbility { card, ability, .. }
            | Action::AnnounceBorrowedMana { card, ability, .. }
            | Action::AnnounceGraveyardBorrowedAbility { card, ability, .. }
            | Action::AnnounceGraveyardBorrowedMana { card, ability, .. }
            | Action::Activate { card, ability, .. }
            | Action::ActivateMana { card, ability, .. }
            | Action::ActivateBorrowedMana { card, ability, .. }
            | Action::ActivateGraveyardBorrowedMana { card, ability, .. }
            | Action::BeginActivate { card, ability, .. } => {
                (Some(self.object_ref(card)), Some(ability))
            }
            Action::AnnounceGrantedMana { card, .. } => (Some(self.object_ref(card)), None),
            Action::AnnounceSourceGrant { source, ability }
            | Action::AnnounceGrantedManaTemplate {
                source, ability, ..
            }
            | Action::AnnouncePlaneswalkerGrant {
                source, ability, ..
            }
            | Action::AnnounceArtifactFood {
                source, ability, ..
            }
            | Action::AnnounceCreatureGrant {
                source, ability, ..
            } => (Some(source), Some(ability)),
            Action::AnnounceSelfGrantedAbility { source, .. }
            | Action::AnnounceSelfGrantedMana { source, .. }
            | Action::AnnounceRetainedCopyAbility { source }
            | Action::AnnounceEmbalm { source, .. }
            | Action::AnnounceInheritedAbility { source, .. }
            | Action::AnnounceInheritedMana { source, .. }
            | Action::ActivateInheritedMana { source, .. } => (Some(source), None),
            _ => return,
        };
        let attempt = self.state.cast_rollback.as_mut().unwrap();
        attempt.source = source;
        attempt.ability = ability;
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
            let mut rollback = rollback;
            let current = std::mem::replace(&mut self.state, rollback.before.clone());
            self.complete_cast_rollback_restore(
                &current,
                rollback.spell,
                &rollback.public_hand_cards,
                &rollback.obscured_hand_cards,
                &rollback.public_face_down,
                &rollback.obscured_face_down,
            );
            self.finish_attempt_recovery(&mut rollback, Vec::new());
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
        // A retained mana activation may have tapped a proposed blocker.
        // After reversing the declaration it must qualify for a new choice.
        if self.state.pending == Pending::Blockers
            && !self
                .state
                .selected_blocks
                .iter()
                .all(|&(blocker, attacker)| {
                    self.state.attackers.iter().any(|a| a.card == attacker)
                        && self.can_block_pair(blocker, attacker)
                })
        {
            self.state.selected_blocks.clear();
        }
        // Undone proposal triggers disappear; retain the original priority holder.
        if self.state.resolving.is_none() && self.state.pending == Pending::Priority {
            self.state_based_actions();
            self.prepare_triggers();
        }
    }
    pub(super) fn clear_finished_cast_rollback(&mut self) {
        let Some(rollback) = self.state.cast_rollback.as_ref() else {
            return;
        };
        if matches!(self.state.pending, Pending::ManaRollback { .. }) {
            return;
        }
        let before_contains = |id| {
            rollback
                .before
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.id == id)
                || rollback
                    .before
                    .mana_announcement_stack
                    .as_ref()
                    .is_some_and(|frames| {
                        frames.iter().any(|frame| frame.announcement.item.id == id)
                    })
        };
        let owned = self
            .state
            .announcing
            .as_ref()
            .into_iter()
            .chain(
                self.state
                    .mana_announcement_stack
                    .as_ref()
                    .into_iter()
                    .flat_map(|frames| frames.iter().map(|frame| &frame.announcement)),
            )
            .find(|a| !a.trigger && !before_contains(a.item.id))
            .map(|a| {
                (
                    a.item.id,
                    a.item.source.map(|source| source.object),
                    match a.item.kind {
                        StackKind::ActivatedAbility { ability } => Some(ability),
                        _ => None,
                    },
                )
            });
        if let Some((id, source, ability)) = owned {
            let attempt = self.state.cast_rollback.as_mut().unwrap();
            attempt.context = Some(id);
            attempt.source = attempt.source.or(source);
            attempt.ability = attempt.ability.or(ability);
            return;
        }
        // Special actions retain their historical checkpoints and controls.
        if rollback.special
            && self.outcome().is_none()
            && (self.state.combat_payment.is_some()
                || self.face_up_payment().is_some()
                || self.state.room_unlock_payment().is_some()
                || self.plot_payment().is_some())
        {
            return;
        }
        let mut rollback = self.state.cast_rollback.take().unwrap();
        if rollback.attempt_id != 0 {
            self.state.last_attempt_outcome = Some(AttemptOutcomeView {
                attempt_id: rollback.attempt_id,
                player: rollback.player,
                source: rollback.source,
                ability: rollback.ability,
                turn: rollback.turn,
                outcome: attempts::AttemptOutcome::Completed,
                reason: None,
            });
        }
        let events = std::mem::take(&mut rollback.events);
        let draws = std::mem::take(&mut rollback.deferred_draws);
        self.restore_attempt_parent(&mut rollback, events);
        self.release_attempt_draws(draws);
    }

    /// Move completed/retained child receipts to their actual parent owner.
    pub(super) fn restore_attempt_parent(
        &mut self,
        rollback: &mut CastRollback,
        events: Vec<cast_rollback_mana::ManaEvent>,
    ) {
        if let Some(mut parent) = rollback.parent.take() {
            for mut event in events {
                event.id =
                    u32::try_from(parent.events.len()).capacity("mana receipt sequence exhausted");
                parent.events.push(event);
            }
            for card in &rollback.public_hand_cards {
                if !parent.public_hand_cards.contains(card) {
                    parent.public_hand_cards.push(*card);
                }
            }
            for card in &rollback.obscured_hand_cards {
                parent.public_hand_cards.retain(|known| known != card);
                if !parent.obscured_hand_cards.contains(card) {
                    parent.obscured_hand_cards.push(*card);
                }
            }
            for object in &rollback.public_face_down {
                if !parent.public_face_down.contains(object) {
                    parent.public_face_down.push(*object);
                }
            }
            for object in &rollback.obscured_face_down {
                parent.public_face_down.retain(|known| known != object);
                if !parent.obscured_face_down.contains(object) {
                    parent.obscured_face_down.push(*object);
                }
            }
            self.state.cast_rollback = Some(parent);
        }
    }

    pub(super) fn finish_attempt_recovery(
        &mut self,
        rollback: &mut CastRollback,
        retained: Vec<cast_rollback_mana::ManaEvent>,
    ) {
        if let Some(reason) = rollback.failure {
            self.state.last_attempt_outcome = Some(AttemptOutcomeView {
                attempt_id: rollback.attempt_id,
                player: rollback.player,
                source: rollback.source,
                ability: rollback.ability,
                turn: rollback.turn,
                outcome: attempts::AttemptOutcome::Failed,
                reason: Some(reason),
            });
        }
        let draws = std::mem::take(&mut rollback.deferred_draws);
        self.restore_attempt_parent(rollback, retained);
        self.release_attempt_draws(draws);
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
        // Library movements and shuffles cannot be undone. Preserve both
        // directions, including cards actually drawn during this attempt, with
        // their frozen incarnations. The proposed spell itself is the exception.
        // Silent zone commits never redispatch the undone proposal's observers.
        let retained = current
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (id as CardId != spell
                    && self.state.objects.get(id).is_some_and(|old| {
                        (old.zone == Zone::Library || o.zone == Zone::Library)
                            && (old.zone != o.zone || old.generation != o.generation)
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
            if before.zone != object.zone {
                self.commit_planned_zone_move(
                    before,
                    zone_replacements::Plan::unreplaced(object.zone),
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
            player.cards_drawn_this_turn = now.cards_drawn_this_turn;
            player.drew_in_own_draw_step = now.drew_in_own_draw_step;
            player.attempted_empty_draw = now.attempted_empty_draw;
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
