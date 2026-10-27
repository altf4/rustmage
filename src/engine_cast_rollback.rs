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
        }
    }
    pub(super) fn bytes(&self) -> usize {
        self.bytes
            + self.events.iter().map(|e| e.bytes()).sum::<usize>()
            + self.replay.as_ref().map_or(0, |r| r.bytes())
            + self.capture.as_ref().map_or(0, |r| r.bytes())
    }
}
impl Game {
    pub(super) fn begin_attack_rollback(&mut self, actor: Player) {
        assert!(self.state.cast_rollback.is_none());
        self.state.cast_rollback = Some(Box::new(CastRollback::new(
            self.state.clone(),
            actor,
            CardId::MAX,
        )));
    }
    pub(super) fn prepare_cast_rollback(&mut self, actor: Player, action: &Action) {
        if let Some(rollback) = &mut self.state.cast_rollback {
            if actor != rollback.player
                && matches!(
                    action,
                    Action::ChooseEntryValue { .. }
                        | Action::ChooseEntryLife { .. }
                        | Action::ChooseAuraAttachment { .. }
                        | Action::ChooseEntryReplacement { .. }
                        | Action::ChooseEntryTimestamp { .. }
                        | Action::ChooseReplacement { .. }
                        | Action::ChooseZoneReplacement { .. }
                )
            {
                rollback.foreign_choice = true;
            }
            return;
        }
        let card = match *action {
            Action::AnnounceCast { card }
            | Action::AnnounceExile { card }
            | Action::AnnounceExileLife { card, .. }
            | Action::AnnounceFlashSurcharge { card }
            | Action::AnnounceWarp { card }
            | Action::AnnounceSneak { card, .. }
            | Action::AnnounceAdventure { card }
            | Action::AnnounceOmen { card }
            | Action::AnnounceBack { card }
            | Action::AnnouncePlotted { card }
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
            self.complete_cast_rollback_restore(&current, rollback.spell);
            return;
        }
        if let Some(r) = &mut self.state.cast_rollback {
            r.foreign_choice = false;
        }
    }
    pub(super) fn complete_cast_rollback_restore(&mut self, current: &State, spell: CardId) {
        self.preserve_irreversible_cast_library_changes(current, spell);
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
        if (!casting && self.state.attack_payment.is_none()) || self.outcome().is_some() {
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
                        shuffle: false,
                    },
                    false,
                    Counters::default(),
                    None,
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
