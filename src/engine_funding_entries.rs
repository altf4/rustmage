//! Cold owned facts for immediate entries inside a declared cost transaction.
use super::*;
use ability_world::CharacteristicWorld;

#[derive(Clone)]
pub(super) struct PaymentContinuation<'g> {
    head: &'g Announcement,
    outer: &'g [mana_announcements::ManaAnnouncement],
    quality: Option<&'g mana_lots::Quality>,
}
impl PaymentContinuation<'_> {
    pub(super) fn selects(&self, id: CardId) -> bool {
        self.head
            .additional
            .is_some_and(|selected| selected.card == id)
            || self.outer.iter().any(|frame| {
                frame
                    .announcement
                    .additional
                    .is_some_and(|selected| selected.card == id)
            })
    }
}
#[derive(Clone)]
pub(super) struct EntryProjection {
    /// Opt-in restoration origins; these are copy changes, never entry history.
    pub copy_origins: Vec<(ObjectRef, Option<u64>)>,
    pub manifested: Vec<(ObjectRef, Card)>,
    pub static_sources: Vec<CardId>,
    pub attachment_sources: Vec<CardId>,
    pub roles: roles::RoleState,
    pub hand_size: hand_size::HandSizeState,
    pub controls: continuous_control::ControlState,
    pub entry_values: Vec<entry_choices::ChoiceRow>,
    pub order: Vec<ObjectRef>,
    /// Every committed entry, independent of whether it provides a timestamped
    /// continuous effect. Power-up and public entry facts use this exact history.
    pub entered: Vec<ObjectRef>,
    pub battle_protectors: Vec<(ObjectRef, Player)>,
    pub consumed: Vec<ObjectRef>,
    pub counter: Option<replacements::CounterPreview>,
}
impl EntryProjection {
    fn from_world(w: &FundingState<'_>) -> Self {
        let mut hand_size = w
            .game
            .state
            .hand_size
            .as_deref()
            .cloned()
            .unwrap_or_default();
        hand_size.sources.retain(|row| {
            w.reference(row.source.card) == row.source
                && w.object(row.source.card).zone == Zone::Battlefield
        });
        Self {
            copy_origins: Vec::new(),
            manifested: w
                .game
                .state
                .manifest
                .as_ref()
                .map_or_else(Vec::new, |state| {
                    state
                        .permanents
                        .iter()
                        .filter(|row| {
                            w.reference(row.object.card) == row.object
                                && w.object(row.object.card).zone == Zone::Battlefield
                        })
                        .map(|row| (row.object, row.underlying))
                        .collect()
                }),
            static_sources: w.static_sources().to_vec(),
            attachment_sources: w.attachment_sources().to_vec(),
            roles: w.roles().cloned().unwrap_or_default(),
            hand_size,
            controls: w.controls().cloned().unwrap_or_default(),
            entry_values: w.entry_values().to_vec(),
            order: Vec::new(),
            entered: Vec::new(),
            battle_protectors: Vec::new(),
            consumed: Vec::new(),
            counter: None,
        }
    }
    fn register(&mut self, source: ObjectRef, card: Card) -> u64 {
        self.order.push(source);
        let timestamp = self.roles.next_timestamp();
        self.register_at(source, card, timestamp)
    }
    pub(super) fn register_at(&mut self, source: ObjectRef, card: Card, timestamp: u64) -> u64 {
        if !self.static_sources.contains(&source.card) {
            self.static_sources.push(source.card);
        }
        if card.attachment_bonus().is_some() && !self.attachment_sources.contains(&source.card) {
            self.attachment_sources.push(source.card);
        }
        if let Some(spec) = card.base_stat_setter() {
            self.roles.base_stats.push(BaseStatsEffectView {
                recipient: source,
                source,
                power: spec.power,
                toughness: spec.toughness,
                timestamp,
                static_source: true,
                permanent: false,
            });
        }
        if !card.live_static_effects().is_empty() {
            self.roles.live_static.push(LiveStaticSourceView {
                source,
                timestamp,
                effects: card.live_static_effects(),
            });
        }
        if card.has_layer6_provider() {
            use crate::cards::StaticAbilityFragment as F;
            self.roles
                .ability_timestamps
                .push(AbilitySourceTimestampView { source, timestamp });
            if card.static_ability_groups().iter().any(|g| {
                g.fragments
                    .iter()
                    .any(|f| matches!(f, F::RemoveAttachedAbilities))
            }) {
                self.roles.ability_removal_sources.push(source);
            }
            if card.static_ability_groups().iter().any(|g| {
                g.fragments
                    .iter()
                    .any(|f| matches!(f, F::AttachedCharacteristics(_)))
            }) {
                self.roles.characteristic_sources.push(source);
            }
        }
        timestamp
    }
}
impl FundingState<'_> {
    pub(super) fn prepare_entry_projection(&mut self) {
        if self.entry_projection.is_none() {
            self.entry_projection = Some(Box::new(EntryProjection::from_world(self)));
            self.power_sensitive = true;
            for (id, object) in self.game.state.objects.iter().enumerate() {
                if !self.resources.iter().any(|r| r.id == id as CardId) {
                    self.resources.push(Resource {
                        id: id as CardId,
                        object: object.clone(),
                        attacking: self
                            .game
                            .state
                            .attackers
                            .iter()
                            .any(|r| r.card == id as CardId),
                    });
                }
            }
        }
    }
    /// Current links are immutable rules records. Exact virtual incarnations decide
    /// whether each immediate zone-change effect still has an object to return.
    pub(super) fn visit_linked_returns(&self, emit: &mut dyn FnMut(&Self) -> bool) -> bool {
        if self.game.state.exile_links.is_empty() {
            return emit(self);
        }
        let links = self
            .game
            .state
            .exile_links
            .iter()
            .filter(|link| {
                let source = self.object(link.source.card);
                let exiled = self.object(link.exiled.card);
                (source.zone != Zone::Battlefield
                    || self.reference(link.source.card) != link.source)
                    && exiled.zone == Zone::Exile
                    && self.reference(link.exiled.card) == link.exiled
                    && !exiled.is_token()
                    && self
                        .entry_projection
                        .as_ref()
                        .is_none_or(|p| !p.consumed.contains(&link.exiled))
            })
            .collect::<Vec<_>>();
        if links.is_empty() {
            return emit(self);
        }
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        let moves = links
            .into_iter()
            .map(|link| {
                entries::EntryMove::from_view(
                    query.object_view(link.exiled.card),
                    link.return_zone,
                    false,
                )
            })
            .collect::<Vec<_>>();
        if moves.is_empty() {
            return emit(self);
        }
        let mut next = self.clone();
        next.prepare_entry_projection();
        next.entry_projection
            .as_mut()
            .unwrap()
            .consumed
            .extend(moves.iter().map(|m| m.before.reference));
        next.visit_entry_moves(moves, true, None, emit)
    }
}

impl FundingState<'_> {
    /// Evaluate one hypothetical entrant with current continuous effects, its
    /// own static effects, and entry-characteristic effects. Other simultaneous
    /// entrants are deliberately absent for replacement-effect assessment.
    pub(super) fn preview_entry_view(&self, movement: &entries::EntryMove) -> Option<ObjectView> {
        self.with_entry_preview(movement, |world| {
            characteristic_query::CharacteristicQuery::in_world(world)
                .object_view(movement.before.id)
        })
    }

    /// Replacement selection uses existing effects and the entrant's own
    /// static abilities, but not an effect created simultaneously by its return.
    pub(super) fn entry_replacement_has_subtype(
        &self,
        movement: &entries::EntryMove,
        subtype: &str,
    ) -> bool {
        self.with_entry_preview(&movement.replacement_preview(), |world| {
            let id = movement.before.id;
            world.subtypes(id).has(world.object(id).card, subtype)
        })
        .unwrap_or(false)
    }

    fn with_entry_preview<T>(
        &self,
        movement: &entries::EntryMove,
        mut inspect: impl FnMut(&Self) -> T,
    ) -> Option<T> {
        if movement.skipped || movement.destination != Zone::Battlefield {
            return None;
        }
        let mut seed = self.clone();
        if !seed.resources.iter().any(|r| r.id == movement.before.id) {
            seed.resources.push(Resource {
                id: movement.before.id,
                attacking: false,
                object: seed.game.state.objects[movement.before.id as usize].clone(),
            });
        }
        let order: Vec<usize> = if Self::entry_has_timestamp(movement) {
            vec![0]
        } else {
            vec![]
        };
        let mut result = None;
        seed.commit_projected_entries(vec![movement.clone()], &order, &mut |world| {
            result = Some(inspect(world));
            true
        });
        result
    }
    pub(super) fn entry_guard(&self, guard: StepGuard, movement: &entries::EntryMove) -> bool {
        let player = movement.controller;
        let source = movement.before.reference;
        let state = &self.game.state.players[player.index()];
        let count = |zone, owner| {
            self.resources
                .iter()
                .filter(|r| {
                    r.object.zone == zone
                        && if zone == Zone::Battlefield {
                            r.object.controller == owner
                        } else {
                            r.object.owner == owner
                        }
                })
                .count()
        };
        match guard {
            StepGuard::SourceUnprepared => {
                let object = ability_world::CharacteristicWorld::object(self, source.card);
                object.zone == Zone::Battlefield
                    && self.reference(source.card) == source
                    && !self.game.is_prepared(source)
            }
            StepGuard::ClassLevelAtLeast(level) => {
                let object = ability_world::CharacteristicWorld::object(self, source.card);
                object.zone == Zone::Battlefield
                    && self.reference(source.card) == source
                    && object.class_level >= level
            }
            StepGuard::UnsolvedLifeGainedAtLeast(n) => {
                !self.game.case_solved(source) && state.life_gained_this_turn >= n
            }
            StepGuard::Always => true,
            StepGuard::ArtifactEnteredThisTurn => state.turn_facts.artifact_entered,
            StepGuard::HeroEnteredOrAttackedThisTurn => {
                state.turn_facts.hero_entered || state.turn_facts.hero_attacked
            }
            StepGuard::AttachedControllerTurn => {
                steps::attached_controller_turn(self, source, self.active)
            }
            StepGuard::YourTurn => self.active == player,
            StepGuard::GainedLife => self.gained_life(player),
            StepGuard::ScriedOrSurveilledThisTurn => state.scried_or_surveilled_this_turn,
            StepGuard::LoyaltyAbilityActivatedThisTurn => state.loyalty_ability_activated_this_turn,
            StepGuard::OpponentNoncombatDamageLastTurn => {
                self.game.state.players[player.other().index()].noncombat_damage_last_turn
            }
            StepGuard::GainedAndLostLife => self.gained_life(player) && self.lost_life(player),
            StepGuard::Attacked => state.attacked_this_turn,
            StepGuard::CreatureDied => self.creatures_died > 0,
            StepGuard::OpponentLostLife => self.lost_life(player.other()),
            StepGuard::CommittedCrimeThisTurn => state.committed_crime_this_turn,
            StepGuard::CardExiledThisTurn => {
                self.game.state.cards_exiled_this_turn
                    || self.resources.iter().any(|r| {
                        r.object.zone == Zone::Exile
                            && (self.game.state.objects[r.id as usize].zone != Zone::Exile
                                || self.game.object_ref(r.id) != self.reference(r.id))
                    })
            }
            StepGuard::OpponentHasMoreCardsInHand => {
                count(Zone::Hand, player.other()) > count(Zone::Hand, player)
            }
            StepGuard::OpponentControlsMoreCreatures => {
                let creatures = |p| {
                    self.resources
                        .iter()
                        .filter(|r| {
                            r.object.zone == Zone::Battlefield
                                && r.object.controller == p
                                && CharacteristicWorld::kinds(self, r.id) & 1 != 0
                        })
                        .count()
                };
                creatures(player.other()) > creatures(player)
            }
            StepGuard::SourceTapped => {
                if self.reference(source.card) == source
                    && self.object(source.card).zone == Zone::Battlefield
                {
                    self.object(source.card).tapped
                } else {
                    movement.before.tapped
                }
            }
            StepGuard::SourceReceivedCounterThisTurn => counter_placement_history::any_placed(
                &self.counter_placement_history,
                source,
                player,
            ),
            StepGuard::SourceCreature => {
                self.reference(source.card) == source
                    && self.object(source.card).zone == Zone::Battlefield
                    && CharacteristicWorld::kinds(self, source.card) & 1 != 0
            }
            StepGuard::SourcePowerAtLeast(n) => {
                self.reference(source.card) == source
                    && self.object(source.card).zone == Zone::Battlefield
                    && self.power(source.card) >= i128::from(n)
            }
            StepGuard::SourceToughnessAtMost(n) => {
                self.reference(source.card) == source
                    && self.object(source.card).zone == Zone::Battlefield
                    && characteristic_query::CharacteristicQuery::in_world(self)
                        .stats(source.card)
                        .toughness
                        <= i128::from(n)
            }
            StepGuard::Outlaw => self.resources.iter().any(|r| {
                r.object.zone == Zone::Battlefield
                    && r.object.controller == player
                    && ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
                        .iter()
                        .any(|s| self.subtypes(r.id).has(r.object.card, s))
            }),
            StepGuard::Expression {
                value,
                minimum,
                maximum,
            } => {
                let v = self.value(value, source.card);
                minimum.is_none_or(|n| v >= n) && maximum.is_none_or(|n| v <= n)
            }
            StepGuard::Threshold {
                value,
                minimum,
                maximum,
            } => {
                let creatures = self
                    .resources
                    .iter()
                    .filter(|r| {
                        r.object.zone == Zone::Battlefield
                            && r.object.controller == player
                            && CharacteristicWorld::kinds(self, r.id) & 1 != 0
                    })
                    .collect::<Vec<_>>();
                let v = match value {
                    StepValue::Life => Some(self.life(player)),
                    StepValue::Hand => Some(count(Zone::Hand, player) as i128),
                    StepValue::SpellsCast => Some(i128::from(state.spells_cast_this_turn)),
                    StepValue::Creatures => Some(creatures.len() as i128),
                    StepValue::TappedCreatures => {
                        Some(creatures.iter().filter(|r| r.object.tapped).count() as i128)
                    }
                    StepValue::EnchantedCreatures => Some(
                        creatures
                            .iter()
                            .filter(|r| {
                                self.attachment_sources().iter().any(|&id| {
                                    self.object(id).zone == Zone::Battlefield
                                        && self.object(id).card.aura().is_some()
                                        && self.object(id).attached_to == Some(self.reference(r.id))
                                })
                            })
                            .count() as i128,
                    ),
                    StepValue::Lands => Some(
                        self.resources
                            .iter()
                            .filter(|r| {
                                r.object.zone == Zone::Battlefield
                                    && r.object.controller == player
                                    && CharacteristicWorld::kinds(self, r.id) & 2 != 0
                            })
                            .count() as i128,
                    ),
                    StepValue::GreatestCreaturePower => {
                        creatures.iter().map(|r| self.power(r.id)).max()
                    }
                    StepValue::GreatestOtherCreaturePower => creatures
                        .iter()
                        .filter(|r| self.reference(r.id) != source)
                        .map(|r| self.power(r.id))
                        .max(),
                    StepValue::TotalCreaturePower => Some(
                        creatures
                            .iter()
                            .map(|r| self.power(r.id))
                            .try_fold(0i128, i128::checked_add)
                            .capacity("entry power exhausted"),
                    ),
                    StepValue::TotalCreatureToughness => Some(
                        creatures
                            .iter()
                            .map(|r| {
                                characteristic_query::CharacteristicQuery::in_world(self)
                                    .stats(r.id)
                                    .toughness
                            })
                            .try_fold(0i128, i128::checked_add)
                            .capacity("entry toughness exhausted"),
                    ),
                };
                v.is_some_and(|v| minimum.is_none_or(|n| v >= n) && maximum.is_none_or(|n| v <= n))
            }
        }
    }
    fn entry_counters(&self, m: &entries::EntryMove) -> Counters {
        if m.destination != Zone::Battlefield {
            return Counters::default();
        }
        let card = m.before.card;
        let mut counters = card.enters_counters();
        if m.battle_protector.is_none() {
            counters.remove(CounterKind::Defense, card.printed_defense());
        }
        if let Some((kind, max)) = card.x_complement_entry_counters() {
            counters.add(kind, u64::from(max.saturating_sub(m.x)));
        }
        for rule in card.entry_counter_rules() {
            if !self.entry_guard(rule.guard, m) {
                continue;
            }
            let amount = match rule.amount {
                EntryCounterAmount::ControlledCreatureDeaths => {
                    self.controlled_creatures_died[m.controller.index()]
                }
                EntryCounterAmount::GreatestOtherCreaturePower => self
                    .resources
                    .iter()
                    .filter(|r| {
                        r.object.zone == Zone::Battlefield
                            && r.object.controller == m.controller
                            && r.id != m.before.id
                            && CharacteristicWorld::kinds(self, r.id) & 1 != 0
                    })
                    .map(|r| self.power(r.id))
                    .max()
                    .unwrap_or(0)
                    .max(0)
                    .try_into()
                    .capacity("entry power exhausted"),
                EntryCounterAmount::ColorsSpent => m
                    .cast_payment
                    .map_or(0, |p| (0..5).filter(|&i| p[i] > 0).count() as u64),
                EntryCounterAmount::IfNotCastOrZero(n) => {
                    if m.cast_payment.is_none_or(|p| p == [0; 6]) {
                        u64::from(n)
                    } else {
                        0
                    }
                }
                EntryCounterAmount::Fixed(n) => u64::from(n),
                EntryCounterAmount::Stat(v) => u64::try_from(self.value(v, m.before.id).max(0))
                    .capacity("entry counter amount exhausted"),
            };
            counters.add(CounterKind::PlusOne, amount);
        }
        if m.cast_origin.is_some() {
            for &(kind, n) in card.cast_entry_counters() {
                counters.add(kind, n)
            }
        }
        if m.cast_origin == Some(Zone::Hand) {
            for &(kind, n) in card.hand_cast_entry_counters() {
                counters.add(kind, n)
            }
        }
        if let Some((kind, n)) = m.effect_counter {
            counters.add(kind, n)
        }
        if card.x_entry_counters() {
            counters.add(CounterKind::PlusOne, u64::from(m.x))
        }
        if m.kicked {
            counters.add(
                CounterKind::PlusOne,
                u64::from(card.kicked_entry_counters()),
            )
        }
        counters
    }
    fn entry_tapped(&self, m: &entries::EntryMove) -> bool {
        let owner = m.controller;
        let land_count = || {
            self.resources
                .iter()
                .filter(|r| {
                    r.id != m.before.id
                        && r.object.zone == Zone::Battlefield
                        && r.object.controller == owner
                        && CharacteristicWorld::kinds(self, r.id) & 2 != 0
                })
                .count()
        };
        match m.before.card.entry_tapped() {
            EntryTapped::Never | EntryTapped::PayLife(_) | EntryTapped::UnlessBehold(_) => false,
            EntryTapped::Always => true,
            EntryTapped::ChosenXAtMost(n) => m.x <= n,
            EntryTapped::OpponentsTurn => owner != self.active,
            EntryTapped::DuringFirstThreeTurns => {
                owner != self.active
                    || !(1..=3).contains(&self.game.state.players[owner.index()].turns_started)
            }
            EntryTapped::UnlessLowLife => [Player::P0, Player::P1]
                .into_iter()
                .all(|p| self.life(p) > 13),
            EntryTapped::UnlessAtMostTwo => land_count() > 2,
            EntryTapped::UnlessAtLeastTwo => land_count() < 2,
            EntryTapped::UnlessBasic => !self.resources.iter().any(|r| {
                r.object.zone == Zone::Battlefield
                    && r.object.controller == owner
                    && r.object.card.is_basic_land()
                    && CharacteristicWorld::kinds(self, r.id) & 2 != 0
            }),
            EntryTapped::UnlessControlsEitherSubtype(first, second) => {
                !self.resources.iter().any(|r| {
                    r.object.zone == Zone::Battlefield
                        && r.object.controller == owner
                        && (self.subtypes(r.id).has(r.object.card, first)
                            || self.subtypes(r.id).has(r.object.card, second))
                })
            }
            EntryTapped::UnlessControlsSubtype(subtype) => !self.resources.iter().any(|r| {
                r.object.zone == Zone::Battlefield
                    && r.object.controller == owner
                    && self.subtypes(r.id).has(r.object.card, subtype)
            }),
            EntryTapped::UnlessPlaneswalker => !self.resources.iter().any(|r| {
                r.object.zone == Zone::Battlefield
                    && r.object.controller == owner
                    && CharacteristicWorld::kinds(self, r.id) & 64 != 0
            }),
            EntryTapped::UnlessPlainsOrIsland => !self.resources.iter().any(|r| {
                r.object.zone == Zone::Battlefield
                    && r.object.controller == owner
                    && (self.subtypes(r.id).has(r.object.card, "Plains")
                        || self.subtypes(r.id).has(r.object.card, "Island"))
            }),
        }
    }
    fn initialize_entry_tap(&self, m: &mut entries::EntryMove) -> Vec<entry_tap::EntryTapEffect> {
        let mut effects = Vec::new();
        let kinds = if m.ghost_vacuum_after.is_some() {
            self.preview_entry_view(m)
                .map_or_else(|| m.entry_kinds(), |view| view.kinds)
        } else {
            m.entry_kinds()
        };
        for &id in self.static_sources() {
            let o = self.object(id);
            if o.zone == Zone::Battlefield
                && o.controller == m.controller
                && o.card.controlled_entry_untapped() & kinds != 0
                && self.fragment_available(id, |f| {
                    matches!(
                        f,
                        crate::cards::StaticAbilityFragment::ControlledEntryUntapped
                    )
                })
            {
                effects.push(entry_tap::EntryTapEffect {
                    source: self.reference(id),
                    intrinsic: false,
                    kind: entry_tap::EntryTapKind::Untapped,
                });
            }
        }
        let opponent = self
            .static_sources()
            .iter()
            .copied()
            .filter(|&id| {
                let o = self.object(id);
                o.zone == Zone::Battlefield
                    && o.controller != m.controller
                    && o.card.opponent_entry_tapped() & kinds != 0
                    && self.printed_abilities_available(id)
            })
            .collect::<Vec<_>>();
        if effects.is_empty()
            && !matches!(m.before.card.entry_tapped(), EntryTapped::UnlessBehold(_))
        {
            m.tapped |= self.entry_tapped(m) || !opponent.is_empty();
            return effects;
        }
        for id in opponent {
            effects.push(entry_tap::EntryTapEffect {
                source: self.reference(id),
                intrinsic: false,
                kind: entry_tap::EntryTapKind::Tapped,
            });
        }
        if self.entry_tapped(m) {
            effects.push(entry_tap::EntryTapEffect {
                source: m.before.reference,
                intrinsic: true,
                kind: entry_tap::EntryTapKind::Tapped,
            });
        }
        if let EntryTapped::PayLife(n) = m.before.card.entry_tapped() {
            effects.push(entry_tap::EntryTapEffect {
                source: m.before.reference,
                intrinsic: true,
                kind: entry_tap::EntryTapKind::PayLife(n),
            });
        }
        if m.manifest_original.is_none()
            && let EntryTapped::UnlessBehold(subtype) = m.before.card.entry_tapped()
        {
            effects.push(entry_tap::EntryTapEffect {
                source: m.before.reference,
                intrinsic: true,
                kind: entry_tap::EntryTapKind::Behold(subtype),
            });
        }
        effects
    }
    pub(super) fn visit_entry_moves(
        &self,
        mut moves: Vec<entries::EntryMove>,
        initial: bool,
        actual: Option<&entries::EntryBatch>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let mut taps = actual.map_or_else(Vec::new, |b| b.tap_events[b.tap_next..].to_vec());
        if initial {
            for (index, m) in moves.iter_mut().enumerate() {
                m.choose_battle_protector();
                if m.before.kinds & crate::cards::BATTLE_KIND != 0
                    && let Some(view) = self.preview_entry_view(m)
                {
                    m.choose_projected_battle_protector(view.kinds, view.has_subtype("Siege"));
                }
                m.counters = self.entry_counters(m);
                if m.destination == Zone::Battlefield {
                    let effects = self.initialize_entry_tap(m);
                    if !effects.is_empty() {
                        taps.push(entry_tap::EntryTapEvent {
                            index,
                            effects,
                            payment: None,
                            behold: None,
                        });
                    }
                }
            }
        }
        self.visit_entry_taps(moves, taps, 0, actual, emit)
    }
    fn visit_entry_taps(
        &self,
        moves: Vec<entries::EntryMove>,
        taps: Vec<entry_tap::EntryTapEvent>,
        at: usize,
        actual: Option<&entries::EntryBatch>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let Some(event) = taps.get(at) else {
            return self.visit_entry_values(moves, 0, actual, emit);
        };
        if moves[event.index].choice.is_none()
            && moves[event.index].before.card.entry_choice() == Some(EntryChoiceKind::BasicLandType)
        {
            for kind in crate::cards::BasicLandType::ALL {
                let mut next = moves.clone();
                next[event.index].choice = Some(EntryChoiceValue::BasicLandType(kind));
                if self.visit_entry_taps(next, taps.clone(), at, actual, emit) {
                    return true;
                }
            }
            return false;
        }
        if let Some(subtype) = event.behold {
            let excluded = moves
                .iter()
                .filter(|m| m.destination == Zone::Battlefield && !m.skipped)
                .map(|m| m.before.reference)
                .collect::<Vec<_>>();
            let candidates = hand_choices::behold_candidates(
                self,
                moves[event.index].controller,
                subtype,
                &excluded,
            );
            for pay in [false, true] {
                if pay && candidates.is_empty() {
                    continue;
                }
                let mut next = moves.clone();
                next[event.index].tapped |= !pay;
                let mut events = taps.clone();
                events[at].behold = None;
                if self.visit_entry_taps(next, events, at, actual, emit) {
                    return true;
                }
            }
            return false;
        }
        if let Some(cost) = event.payment {
            for pay in [false, true] {
                let m = &moves[event.index];
                if pay && self.life(m.controller) < i128::from(cost) {
                    continue;
                }
                let mut w = self.clone();
                if pay {
                    w.pay_projected_life(m.controller, cost, &moves)
                }
                let mut next = moves.clone();
                next[event.index].life_paid = Some(pay);
                next[event.index].tapped |= !pay;
                let mut events = taps.clone();
                events[at].payment = None;
                if w.visit_entry_taps(next, events, at, actual, emit) {
                    return true;
                }
            }
            return false;
        }
        if event.effects.is_empty() {
            return self.visit_entry_taps(moves, taps, at + 1, actual, emit);
        }
        for index in 0..event.effects.len() {
            let mut next = moves.clone();
            let mut events = taps.clone();
            let effect = events[at].effects.remove(index);
            match effect.kind {
                entry_tap::EntryTapKind::Tapped => next[event.index].tapped = true,
                entry_tap::EntryTapKind::Untapped => next[event.index].tapped = false,
                entry_tap::EntryTapKind::PayLife(n) => events[at].payment = Some(n),
                entry_tap::EntryTapKind::Behold(subtype) => events[at].behold = Some(subtype),
            }
            if self.visit_entry_taps(next, events, at, actual, emit) {
                return true;
            }
        }
        false
    }
    pub(super) fn pay_projected_life(
        &mut self,
        player: Player,
        n: u32,
        entering: &[entries::EntryMove],
    ) {
        if n == 0 {
            return;
        }
        let applies = self.life_replacement_possible
            && self.static_sources().iter().copied().any(|id| {
                let source = self.object(id);
                source.zone == Zone::Battlefield
                    && source.controller == player
                    && self.printed_abilities_available(id)
                    && source.card.static_ability_groups().iter().any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(
                                fragment,
                                StaticAbilityFragment::ReplaceLifePaymentWithLibraryExile
                            )
                        })
                    })
            });
        if applies {
            let cards = self.game.state.players[player.index()]
                .library
                .iter()
                .rev()
                .copied()
                .filter(|&id| {
                    let object = self.object(id);
                    object.zone == Zone::Library
                        && object.is_card()
                        && !entering.iter().any(|m| {
                            !m.skipped
                                && m.destination == Zone::Battlefield
                                && m.before.zone == Zone::Library
                                && m.before.reference == self.reference(id)
                        })
                })
                .take(n as usize)
                .map(|id| self.reference(id))
                .collect::<Vec<_>>();
            if cards.len() == n as usize {
                let mut result = None;
                self.cost_departures(&cards, Zone::Exile, &mut |world| {
                    result = Some(world.clone());
                    true
                });
                if let Some(world) = result {
                    *self = world;
                    return;
                }
            }
        }
        self.life_lost[player.index()] |= n > 0;
        if player == self.actor {
            self.life -= i128::from(n)
        } else {
            self.other_life -= i128::from(n)
        }
    }
    fn aura_candidate(&self, m: &entries::EntryMove, target: ObjectRef) -> bool {
        let Some(spec) = m.before.card.aura() else {
            return false;
        };
        let o = self.object(target.card);
        o.zone == Zone::Battlefield
            && self.reference(target.card) == target
            && !characteristic_query::CharacteristicQuery::in_world(self)
                .keywords(target.card)
                .contains(Keywords::PROTECTION_EVERYTHING)
            && if spec.kinds == 0 {
                CharacteristicWorld::kinds(self, target.card) & (1 | 2 | 16 | 32 | 64 | 256) != 0
            } else {
                CharacteristicWorld::kinds(self, target.card) & u16::from(spec.kinds) != 0
            }
            && ability_world::aura_artifact_or_non_aura_enchantment(
                self,
                m.before.card,
                target.card,
            )
            && match spec.relation {
                0 => o.controller == m.controller,
                1 => o.controller != m.controller,
                _ => true,
            }
    }
    fn visit_entry_values(
        &self,
        moves: Vec<entries::EntryMove>,
        at: usize,
        actual: Option<&entries::EntryBatch>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let Some(m) = moves.get(at) else {
            return self.visit_entry_counter_rows(moves, actual, emit);
        };
        if m.skipped || m.destination != Zone::Battlefield {
            return self.visit_entry_values(moves, at + 1, actual, emit);
        }
        if let Some(kind) = m.before.card.entry_choice()
            && m.choice.is_none()
        {
            let mut choices = Vec::new();
            match kind {
                crate::cards::EntryChoiceKind::ManaValueParity => choices.extend(
                    [ManaValueParity::Odd, ManaValueParity::Even]
                        .into_iter()
                        .map(EntryChoiceValue::ManaValueParity),
                ),
                crate::cards::EntryChoiceKind::Color => choices.extend(
                    crate::cards::ChoiceColor::ALL
                        .into_iter()
                        .map(EntryChoiceValue::Color),
                ),
                crate::cards::EntryChoiceKind::BasicLandType => choices.extend(
                    crate::cards::BasicLandType::ALL
                        .into_iter()
                        .map(EntryChoiceValue::BasicLandType),
                ),
                crate::cards::EntryChoiceKind::CreatureType => {
                    choices.extend(crate::CREATURE_TYPES.iter().map(|name| {
                        EntryChoiceValue::CreatureType(
                            crate::cards::CreatureType::from_name(name).unwrap(),
                        )
                    }))
                }
                crate::cards::EntryChoiceKind::LandCardName => {
                    choices.extend(crate::CardName::lands().map(EntryChoiceValue::CardName));
                }
                crate::cards::EntryChoiceKind::CardNameAfterOpponentHand => {
                    // Names absent from this world are equivalent for cost completion.
                    let mut unmatched = false;
                    for name in crate::CardName::all() {
                        let present = self.resources.iter().any(|r| name.matches(self.name(r.id)));
                        if present || !unmatched {
                            choices.push(EntryChoiceValue::CardName(name));
                            unmatched |= !present;
                        }
                    }
                }
            }
            for value in choices {
                let mut next = moves.clone();
                let row = &mut next[at];
                row.choice = Some(value);
                if let (Some(kind), EntryChoiceValue::CreatureType(typ)) =
                    (row.before.card.chosen_entry_counters(), value)
                {
                    let n = self
                        .resources
                        .iter()
                        .filter(|r| {
                            r.object.zone == Zone::Battlefield
                                && r.object.controller == row.controller
                                && CharacteristicWorld::kinds(self, r.id) & 1 != 0
                                && self.subtypes(r.id).has(r.object.card, typ.name())
                        })
                        .count() as u64;
                    row.counters.add(kind, n)
                }
                if self.visit_entry_values(next, at, actual, emit) {
                    return true;
                }
            }
            return false;
        }
        if m.life_paid.is_none()
            && let EntryTapped::PayLife(cost) = m.before.card.entry_tapped()
        {
            for pay in [false, true] {
                if pay && self.life(m.controller) < i128::from(cost) {
                    continue;
                }
                let mut w = self.clone();
                if pay {
                    w.pay_projected_life(m.controller, cost, &moves)
                }
                let mut next = moves.clone();
                next[at].life_paid = Some(pay);
                next[at].tapped |= !pay;
                if w.visit_entry_values(next, at, actual, emit) {
                    return true;
                }
            }
            return false;
        }
        if !m.artifact_return
            && !m.enchantment_return
            && m.before.card.aura().is_some()
            && m.attachment.is_none()
        {
            let mut any = false;
            for r in &self.resources {
                let target = self.reference(r.id);
                if self.aura_candidate(m, target) {
                    any = true;
                    let mut next = moves.clone();
                    next[at].attachment = Some(target);
                    if self.visit_entry_values(next, at + 1, actual, emit) {
                        return true;
                    }
                }
            }
            if !any {
                let mut next = moves.clone();
                if m.before.zone == Zone::Stack {
                    next[at].destination = Zone::Graveyard
                } else {
                    next[at].skipped = true
                }
                return self.visit_entry_values(next, at + 1, actual, emit);
            }
            return false;
        }
        self.visit_entry_values(moves, at + 1, actual, emit)
    }
}

impl FundingState<'_> {
    fn visit_entry_counter_rows(
        &self,
        mut moves: Vec<entries::EntryMove>,
        actual: Option<&entries::EntryBatch>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        if actual.is_none_or(|b| !b.entry_bonuses_ready) {
            for &id in self.static_sources() {
                let provider = self.object(id);
                if provider.zone != Zone::Battlefield {
                    continue;
                }
                for group in provider.card.static_ability_groups() {
                    for fragment in group.fragments {
                        let crate::cards::StaticAbilityFragment::OtherSubtypeEntryCounters(subtype) =
                            fragment
                        else {
                            continue;
                        };
                        if !self.fragment_available(id,|f|matches!(f,crate::cards::StaticAbilityFragment::OtherSubtypeEntryCounters(s)if s==subtype)){continue}
                        let count = self
                            .resources
                            .iter()
                            .filter(|r| {
                                r.object.zone == Zone::Battlefield
                                    && r.object.controller == provider.controller
                                    && self.subtypes(r.id).has(r.object.card, subtype)
                            })
                            .count() as u64;
                        for m in &mut moves {
                            if !m.skipped
                                && m.destination == Zone::Battlefield
                                && m.controller == provider.controller
                                && m.before.reference != self.reference(id)
                                && self.entry_replacement_has_subtype(m, subtype)
                            {
                                m.counters.add(CounterKind::PlusOne, count)
                            }
                        }
                    }
                }
            }
        }
        let mut order = (0..moves.len()).collect::<Vec<_>>();
        order.sort_by_key(|&i| (moves[i].controller != self.active, i));
        self.visit_entry_counters(moves, &order, 0, actual, emit)
    }
    fn visit_entry_counters(
        &self,
        moves: Vec<entries::EntryMove>,
        order: &[usize],
        at: usize,
        actual: Option<&entries::EntryBatch>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let Some(&index) = order.get(at) else {
            return self.visit_entry_timestamps(
                moves,
                actual.map_or_else(Vec::new, |b| b.timestamp_order.clone()),
                emit,
            );
        };
        let m = &moves[index];
        let current = self
            .entry_projection
            .as_ref()
            .and_then(|p| p.counter.as_ref())
            .filter(|p| p.index == index);
        let processed = actual.is_some_and(|b| b.counter_order[..b.counter_next].contains(&index))
            && current.is_none();
        if m.skipped || m.destination != Zone::Battlefield || m.counters.is_empty() || processed {
            return self.visit_entry_counters(moves, order, at + 1, actual, emit);
        }
        let (placement, mut used) = current.map_or_else(
            || {
                (
                    replacements::CounterPlacement {
                        placed_by: m.controller,
                        object: m.before.reference,
                        card: m.before.card,
                        owner: m.before.owner,
                        controller: m.controller,
                        protector: m.battle_protector,
                        subtypes: if m.ghost_vacuum_after.is_some() {
                            self.preview_entry_view(&m.replacement_preview())
                                .map_or_else(
                                    || Game::entry_replacement_subtypes(m),
                                    |view| view.granted_subtypes,
                                )
                        } else {
                            Game::entry_replacement_subtypes(m)
                        },
                        kind: if m.ghost_vacuum_after.is_some() {
                            self.preview_entry_view(&m.replacement_preview())
                                .map_or_else(|| m.entry_kinds(), |view| view.kinds)
                        } else {
                            m.entry_kinds()
                        },
                        power: if m.ghost_vacuum_after.is_some()
                            || replacements::entry_power_sensitive(
                                self,
                                m.controller,
                                m.before.card,
                            ) {
                            self.preview_entry_view(&m.replacement_preview())
                                .map_or(m.before.power, |view| view.power)
                        } else {
                            m.before.power
                        },
                        amount: m.counters.clone(),
                        effect: true,
                        entry_index: Some(index),
                        private: false,
                    },
                    Vec::new(),
                )
            },
            |p| (p.placement.clone(), p.used.clone()),
        );
        let refresh = |placement: &replacements::CounterPlacement| {
            let mut refreshed = placement.clone();
            if m.ghost_vacuum_after.is_some()
                || replacements::entry_power_sensitive(self, m.controller, m.before.card)
            {
                let mut movement = m.clone();
                movement.counters = placement.amount.clone();
                if let Some(view) = self.preview_entry_view(&movement.replacement_preview()) {
                    refreshed.subtypes = view.granted_subtypes;
                    refreshed.kind = view.kinds;
                    refreshed.power = view.power;
                }
            }
            refreshed
        };
        replacements::counter_outcomes(self, &placement, &mut used, &refresh, &mut |amount| {
            let mut next = moves.clone();
            next[index].counters = amount.clone();
            self.visit_entry_counters(next, order, at + 1, actual, emit)
        })
    }
    fn visit_entry_timestamps(
        &self,
        moves: Vec<entries::EntryMove>,
        prefix: Vec<usize>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let mut rest = moves
            .iter()
            .enumerate()
            .filter(|(i, m)| {
                !prefix.contains(i)
                    && !m.skipped
                    && m.destination == Zone::Battlefield
                    && (Self::entry_has_timestamp(m)
                        || (moves.iter().any(|m| {
                            Game::hand_entry(m)
                                && m.before.card.hand_size_setting().flatten().is_some()
                        }) && Game::hand_entry(m)))
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        rest.sort_by_key(|&i| (moves[i].controller != self.active, i));
        let Some(&first) = rest.first() else {
            return self.commit_projected_entries(moves, &prefix, emit);
        };
        let controller = moves[first].controller;
        for index in rest
            .into_iter()
            .filter(|&i| moves[i].controller == controller)
        {
            let mut next = prefix.clone();
            next.push(index);
            if self.visit_entry_timestamps(moves.clone(), next, emit) {
                return true;
            }
        }
        false
    }
    fn entry_has_timestamp(m: &entries::EntryMove) -> bool {
        let card = m.before.card;
        m.artifact_return
            || m.enchantment_return
            || m.ghost_vacuum_after.is_some()
            || m.food_return.is_some()
            || m.return_modification.is_some()
            || CounterAmounts::from(&m.counters)
                .keyword_amounts()
                .iter()
                .any(|n| *n > 0)
            || card.base_stat_setter().is_some()
            || !card.live_static_effects().is_empty()
            || card.role().is_some()
            || card.has_layer6_provider()
    }
    pub(super) fn commit_projected_entries(
        &self,
        moves: Vec<entries::EntryMove>,
        order: &[usize],
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        let mut next = self.clone();
        next.prepare_entry_projection();
        for m in &moves {
            if !m.skipped {
                next.remember_payment_departure(m.before.id);
            }
        }
        for m in &moves {
            if m.skipped {
                continue;
            }
            let face_down_physical = next.projected_face_down_physical(m.before.reference);
            let row = next
                .resources
                .iter_mut()
                .find(|r| r.id == m.before.id)
                .expect("entry resource retained");
            let o = &mut row.object;
            next.game
                .restore_projected_copy_identity(m.before.reference, o);
            o.card = if m.destination == Zone::Battlefield && m.manifest_original.is_some() {
                m.before.card
            } else if let Some(physical) = face_down_physical
                && m.before.zone == Zone::Battlefield
                && m.destination != Zone::Battlefield
            {
                physical.front_face()
            } else if m.destination == Zone::Battlefield
                && (m.source_before.is_some() || m.before.zone == Zone::Stack)
            {
                m.before.card
            } else if !o.is_token() && !o.is_spell_copy() {
                o.card.front_face()
            } else {
                o.card
            };
            o.zone = m.destination;
            o.generation = o
                .generation
                .checked_add(1)
                .capacity("virtual generation exhausted");
            o.controller = if m.destination == Zone::Battlefield {
                m.controller
            } else {
                o.owner
            };
            o.summoning_sick = m.destination == Zone::Battlefield;
            o.flashback = false;
            o.cast_for_warp = m.destination == Zone::Battlefield
                && m.warp_controller.is_some()
                && m.cast_origin.is_some();
            o.gained_keywords = Keywords::NONE;
            o.growth = Stats::default();
            o.counters = m.counters.hot();
            o.used_abilities = 0;
            o.loyalty_activated_this_turn = false;
            o.class_level = 1;
            o.damage = 0;
            o.damaged_this_turn = false;
            o.dealt_damage_this_turn = false;
            o.dealt_combat_damage_this_incarnation = false;
            o.deathtouch_since_sba = false;
            o.attached_to = m.attachment;
            o.tapped = m.tapped;
            row.attacking = false;
            let source = ObjectRef {
                card: row.id,
                generation: o.generation,
            };
            let manifested = &mut next.entry_projection.as_mut().unwrap().manifested;
            manifested.retain(|(object, _)| *object != m.before.reference);
            if m.destination == Zone::Battlefield
                && let Some(physical) = m.manifest_original
            {
                manifested.push((source, physical));
            }
            live_counters::set(&mut next.named_counters, source, m.counters.named.clone());
            if m.destination == Zone::Battlefield {
                next.entry_projection.as_mut().unwrap().entered.push(source);
                if let Some(protector) = m.battle_protector {
                    next.entry_projection
                        .as_mut()
                        .unwrap()
                        .battle_protectors
                        .push((source, protector));
                }
                if m.counters.has_any() {
                    counter_placement_history::record(
                        &mut next.counter_placement_history,
                        source,
                        m.controller,
                        m.counters.plus_one > 0,
                    );
                }
                let p = next.entry_projection.as_mut().unwrap();
                if !p.static_sources.contains(&row.id) {
                    p.static_sources.push(row.id)
                }
                if o.card.attachment_bonus().is_some() && !p.attachment_sources.contains(&row.id) {
                    p.attachment_sources.push(row.id)
                }
                if let Some(value) = m.choice {
                    p.entry_values.push(entry_choices::ChoiceRow {
                        object: source,
                        value,
                    });
                }
            }
            next.project_copy_departure(m.before.reference, source);
        }
        for &i in order {
            let m = &moves[i];
            if m.skipped || m.destination != Zone::Battlefield {
                continue;
            }
            let source = next.reference(m.before.id);
            let p = next.entry_projection.as_mut().unwrap();
            if !Self::entry_has_timestamp(m) {
                continue;
            }
            let timestamp = p.register(source, m.before.card);
            keyword_counters::sync_effects(
                &mut p.roles,
                source,
                [0; CounterKind::KEYWORDS.len()],
                CounterAmounts::from(&m.counters).keyword_amounts(),
                Some(timestamp),
            );
            if m.artifact_return {
                let timestamp = p.roles.next_timestamp();
                p.roles
                    .animations
                    .push(characteristic_layers::artifact_noncreature_form(
                        source, timestamp,
                    ));
            }
            if m.enchantment_return {
                let timestamp = p.roles.next_timestamp();
                p.roles
                    .animations
                    .push(characteristic_layers::enchantment_noncreature_form(
                        source, timestamp,
                    ));
            }
            if m.food_return.is_some() {
                let timestamp = p.roles.next_timestamp();
                p.roles
                    .animations
                    .push(characteristic_layers::food_form(source, timestamp));
            }
            if let Some((kind, origin)) = m.return_modification
                && let Some((types, keywords)) = return_links::modification(kind)
            {
                let timestamp = p.roles.next_timestamp();
                p.roles
                    .creature_types
                    .push(subtypes::CreatureTypesEffectView {
                        recipient: source,
                        source: origin,
                        types,
                        additional: true,
                        timestamp,
                    });
                if keywords != Keywords::NONE {
                    p.roles.keyword_effects.push(KeywordEffectView {
                        recipient: source,
                        origin: KeywordEffectOrigin::Resolution { source: origin },
                        keywords,
                        rules: Keywords::NONE,
                        timestamp,
                        expires_turn: None,
                    });
                }
            }
            if let Some(recipient) = m.attachment
                && m.before.card.role().is_some()
            {
                p.roles.attached.push(RoleTimestampView {
                    aura: source,
                    recipient,
                    timestamp,
                });
            }
        }
        // CR611.2e: surrounding characteristic effects exist as entrants enter,
        // after entrant static sources have received their timestamps.
        let ghost_timestamp = moves
            .iter()
            .any(|m| {
                !m.skipped && m.destination == Zone::Battlefield && m.ghost_vacuum_after.is_some()
            })
            .then(|| {
                next.entry_projection
                    .as_mut()
                    .unwrap()
                    .roles
                    .next_timestamp()
            });
        for m in &moves {
            if !m.skipped
                && m.destination == Zone::Battlefield
                && let Some(origin) = m.ghost_vacuum_after
            {
                let recipient = next.reference(m.before.id);
                let p = next.entry_projection.as_mut().unwrap();
                let timestamp = ghost_timestamp.expect("ghost entries have a timestamp");
                p.roles
                    .creature_types
                    .push(subtypes::CreatureTypesEffectView {
                        recipient,
                        source: origin,
                        types: &["Spirit"],
                        additional: true,
                        timestamp,
                    });
                p.roles.base_stats.push(base_stats::BaseStatsEffectView {
                    recipient,
                    source: origin,
                    power: Some(1),
                    toughness: Some(1),
                    timestamp,
                    static_source: false,
                    permanent: true,
                });
            }
        }
        for m in &moves {
            if !m.skipped
                && m.destination == Zone::Battlefield
                && let Some((origin, keywords)) = m.keywords_after
            {
                let recipient = next.reference(m.before.id);
                let turn = next.game.state.turn;
                let p = next.entry_projection.as_mut().unwrap();
                let timestamp = p.roles.next_timestamp();
                p.roles.keyword_effects.push(KeywordEffectView {
                    recipient,
                    origin: KeywordEffectOrigin::Resolution { source: origin },
                    keywords: keywords.without(keywords.resolution_rules()),
                    rules: keywords.resolution_rules(),
                    timestamp,
                    expires_turn: Some(turn),
                });
                let row = next
                    .resources
                    .iter_mut()
                    .find(|r| r.id == recipient.card)
                    .unwrap();
                row.object.gained_keywords = row.object.gained_keywords.union(keywords);
            }
        }
        for m in &moves {
            if !m.skipped {
                next.entry_projection
                    .as_mut()
                    .unwrap()
                    .hand_size
                    .forget(m.before.reference);
            }
        }
        for i in Game::hand_entry_order_for(&moves, order, next.active) {
            let m = &moves[i];
            let source = next.reference(m.before.id);
            next.entry_projection
                .as_mut()
                .unwrap()
                .hand_size
                .register(source, m.before.card);
        }
        for m in &moves {
            if m.skipped || m.destination != Zone::Battlefield || !m.before.card.controls_attached()
            {
                continue;
            }
            if let Some(target) = m.attachment {
                let controller = next.object(target.card).controller;
                let p = next.entry_projection.as_mut().unwrap();
                if !p.controls.bases.iter().any(|r| r.object == target) {
                    p.controls.bases.push(continuous_control::ControlBase {
                        object: target,
                        controller,
                    });
                }
            }
        }
        for m in &moves {
            if !m.skipped
                && m.destination == Zone::Battlefield
                && let Some(source) = m.blank_with_haste_after
            {
                let recipient = next.reference(m.before.id);
                let p = next.entry_projection.as_mut().unwrap();
                let timestamp = p.roles.next_timestamp();
                p.roles
                    .resolved_ability_modifications
                    .push(ResolvedAbilityModificationView {
                        recipient,
                        source,
                        keywords: Keywords::HASTE,
                        timestamp,
                    });
            }
        }
        next.entry_projection.as_mut().unwrap().counter = None;
        next.refresh_control_projection();
        next.prune_source_linked_projection();
        next.refresh_enduring_projection();
        for m in &moves {
            if m.skipped || m.destination != Zone::Battlefield {
                continue;
            }
            let Some(recipient) = m.attach_equipment_to else {
                continue;
            };
            let source = next.reference(m.before.id);
            if !equipment::can_attach(&next, source, recipient) {
                continue;
            }
            next.resources
                .iter_mut()
                .find(|r| r.id == source.card)
                .unwrap()
                .object
                .attached_to = Some(recipient);
            let p = next.entry_projection.as_mut().unwrap();
            let timestamp = p.roles.next_timestamp();
            for row in &mut p.roles.live_static {
                if row.source == source {
                    row.timestamp = timestamp;
                }
            }
            for row in &mut p.roles.ability_timestamps {
                if row.source == source {
                    row.timestamp = timestamp;
                }
            }
            for row in &mut p.roles.base_stats {
                if row.static_source && row.source == source {
                    row.timestamp = timestamp;
                }
            }
            p.hand_size.assign(source);
        }
        next.refresh_control_projection();
        next.refresh_enduring_projection();
        emit(&next)
    }
}

impl Game {
    pub(in crate::engine) fn cast_entry_action_fundable(&self, action: &Action) -> bool {
        // A completed legal mana ability must finish its mandatory entry choices
        // even when a foreign decision has already made its parent cast fail.
        if self.foreign_cast_choice_pending()
            && self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.capture.is_some())
        {
            return true;
        }
        if !matches!(
            action,
            Action::ChooseEntryBehold { .. }
                | Action::ChooseEntryValue { .. }
                | Action::ChooseEntryLife { .. }
                | Action::ChooseAuraAttachment { .. }
                | Action::ChooseEntryReplacement { .. }
                | Action::ChooseEntryTimestamp { .. }
                | Action::ChooseReplacement { .. }
        ) {
            return true;
        }
        let caster = self
            .state
            .mana_announcement_stack
            .as_ref()
            .and_then(|v| v.first())
            .map(|f| f.announcement.item.controller)
            .or_else(|| self.state.announcing.as_ref().map(|a| a.item.controller));
        if self.semantic_actor() != caster {
            return true;
        }
        self.projected_cast_entries_fundable(Some(action))
    }
    /// Complete the current casting attempt after an entry decision, including
    /// outstanding entry choices and every suspended mana-payment ancestor.
    #[allow(
        dead_code,
        reason = "Consumed by the integrating foreign-entry rollback hook"
    )]
    pub(in crate::engine) fn current_cast_costs_fundable(&self) -> bool {
        self.projected_cast_entries_fundable(None)
    }
    fn projected_cast_entries_fundable(&self, action: Option<&Action>) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return true;
        };
        let transaction = a.cast_costs.as_ref().is_some_and(|c| c.mana_closed);
        let nested = self
            .state
            .mana_announcement_stack
            .as_deref()
            .and_then(|frames| frames.split_last())
            .filter(|_| a.production.is_some());
        let (mut world, goal) = if transaction {
            (self.cast_cost_world(a), None)
        } else if let Some((parent, outer)) = nested {
            let goal = self.funding_goal(&parent.announcement);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                parent.announcement.item.kind == StackKind::Spell
                    && parent
                        .announcement
                        .item
                        .source
                        .expect("cost announcement has object source")
                        .card
                        .has_convoke(),
            );
            world.set_purpose(self.purpose_for_item(&parent.announcement.item));
            world.set_payment_context(&parent.announcement);
            world.set_funding_continuation(
                &parent.announcement,
                outer,
                parent.quality.as_ref().map(|(_, q)| q),
            );
            (world, Some(goal))
        } else if a.cast_costs.is_some()
            || a.item
                .cast
                .is_some_and(|cast| cast.method == CastMethod::Disguise)
        {
            let goal = self.funding_goal(a);
            let mut world = FundingState::from_game_with_convoke(
                self,
                &goal,
                a.item.kind == StackKind::Spell
                    && a.item
                        .source
                        .expect("cost announcement has object source")
                        .card
                        .has_convoke(),
            );
            world.set_purpose(self.purpose_for_item(&a.item));
            world.set_payment_context(a);
            (world, Some(goal))
        } else {
            return true;
        };
        let mut complete = |w: &FundingState<'_>| {
            if let Some(goal) = &goal {
                let mut next = w.clone();
                if let Some(production) = a.production {
                    for (pool, amount) in next.mana.iter_mut().zip(production) {
                        *pool += u128::from(amount);
                    }
                    next.mana[3] = next.mana[3]
                        .checked_add(u128::from(a.item.mana_tap_bonus.unwrap_or(0)))
                        .capacity("mana representation exhausted");
                    if let Some((_, quality)) = self
                        .state
                        .mana_lots
                        .as_ref()
                        .and_then(|m| m.announcement_quality.as_ref())
                    {
                        mana_lots::add_lot(&mut next.lots, *quality, production.map(u128::from));
                    }
                }
                next.reachable(goal, &mut HashSet::new())
            } else {
                w.cost_completion()
            }
        };
        let Some(mut batch) = self.state.entry_batch.as_deref().cloned() else {
            return complete(&world);
        };
        world.prepare_entry_projection();
        let counter_choice = if let Some(Action::ChooseReplacement { source, ability }) = action {
            Some((*source, *ability))
        } else {
            None
        };
        world.entry_projection.as_mut().unwrap().counter =
            self.projected_entry_counter(counter_choice);
        match action {
            Some(Action::ChooseEntryValue { value }) => {
                let i = self
                    .current_entry_value_index()
                    .expect("active entry choice");
                let m = &mut batch.moves[i];
                m.choice = Some(*value);
                if let (Some(kind), EntryChoiceValue::CreatureType(typ)) =
                    (m.before.card.chosen_entry_counters(), *value)
                {
                    let count = world
                        .resources
                        .iter()
                        .filter(|r| {
                            r.object.zone == Zone::Battlefield
                                && r.object.controller == m.controller
                                && CharacteristicWorld::kinds(&world, r.id) & 1 != 0
                                && world.subtypes(r.id).has(r.object.card, typ.name())
                        })
                        .count() as u64;
                    m.counters.add(kind, count);
                }
            }
            Some(Action::ChooseAuraAttachment { object }) => {
                batch.moves[batch.choices[batch.next]].attachment = Some(*object)
            }
            Some(Action::ChooseEntryLife { pay }) => {
                let payment = batch
                    .tap_events
                    .get(batch.tap_next)
                    .and_then(|e| e.payment.map(|cost| (e.index, cost)));
                let (index, cost) = payment.unwrap_or_else(|| {
                    let i = batch.choices[batch.next];
                    let EntryTapped::PayLife(cost) = batch.moves[i].before.card.entry_tapped()
                    else {
                        unreachable!()
                    };
                    (i, cost)
                });
                let entering = batch.moves.clone();
                let m = &mut batch.moves[index];
                m.life_paid = Some(*pay);
                m.tapped |= !*pay;
                if *pay {
                    world.pay_projected_life(m.controller, cost, &entering)
                }
                if payment.is_some() {
                    batch.tap_events[batch.tap_next].payment = None;
                }
            }
            Some(Action::ChooseEntryBehold { object }) => {
                let event = &mut batch.tap_events[batch.tap_next];
                if event.behold.take().is_none() {
                    return false;
                }
                if object.is_none() {
                    batch.moves[event.index].tapped = true;
                }
            }
            Some(Action::ChooseEntryReplacement { source, intrinsic }) => {
                let event = &mut batch.tap_events[batch.tap_next];
                let Some(i) = event
                    .effects
                    .iter()
                    .position(|e| e.source == *source && e.intrinsic == *intrinsic)
                else {
                    return false;
                };
                let effect = event.effects.remove(i);
                match effect.kind {
                    entry_tap::EntryTapKind::Tapped => batch.moves[event.index].tapped = true,
                    entry_tap::EntryTapKind::Untapped => batch.moves[event.index].tapped = false,
                    entry_tap::EntryTapKind::PayLife(cost) => event.payment = Some(cost),
                    entry_tap::EntryTapKind::Behold(subtype) => event.behold = Some(subtype),
                }
            }
            Some(Action::ChooseEntryTimestamp { object }) => {
                let Some(i) = batch
                    .moves
                    .iter()
                    .position(|m| m.before.reference == *object)
                else {
                    return false;
                };
                batch.timestamp_order.push(i);
            }
            Some(Action::ChooseReplacement { .. }) => {
                if world.entry_projection.as_ref().unwrap().counter.is_none() {
                    return true;
                }
            }
            None => {}
            _ => return true,
        }
        world.visit_entry_moves(batch.moves.clone(), false, Some(&batch), &mut complete)
    }
}

impl<'g> FundingState<'g> {
    pub(super) fn visit_applied(
        &self,
        step: &FundingMove,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        if step.external.is_none()
            && self.cast_costs.is_none()
            && self.continuation.is_none()
            && self.entry_projection.is_none()
            && self.game.state.exile_links.is_empty()
            && self.game.state.zone_rules.is_none()
            && { true }
            && self.game.state.death_exile_replacements.is_none()
            && self.counter_count(self.resources[step.resource].id, CounterKind::Finality) == 0
        {
            let mut next = self.clone();
            next.apply(step, false);
            return emit(&next);
        }
        let external = if let Some(external) = step.external {
            let source = &self.resources[step.resource];
            let card = step.frozen.as_ref().map_or(source.object.card, |f| f.0);
            let ability = step
                .borrowed
                .map_or_else(|| card.activation(step.ability.unwrap()), |b| b.rules());
            let Some(spec) = ability.cost.additional else {
                return false;
            };
            if external.kind != spec.kind
                || !self.cost_object(spec, external.object, self.reference(source.id))
            {
                return false;
            }
            Some((external, spec))
        } else {
            None
        };
        let source = &self.resources[step.resource];
        let reference = self.reference(source.id);
        let mut next = self.clone();
        // Keep the post-payment counter snapshot until replacement planning.
        // Paying away the final Finality counter must differ from sacrificing
        // a permanent which still has one after the counter cost.
        next.apply(step, true);
        let departed = source.object.zone == Zone::Battlefield
            && next.object(source.id).zone != Zone::Battlefield;
        let destination = next.object(source.id).zone;
        if departed {
            next.resources[step.resource].object.zone = Zone::Battlefield;
        }
        // The staged source remains present until a replacement plan commits.
        next.refresh_control_projection();
        next.prune_source_linked_projection();
        next.refresh_enduring_projection();
        if let Some((external, _)) = external {
            // Selection belongs to this incarnation. Never silently substitute a
            // returned card or allow an object already spent by another cost.
            if next.reference(external.object.card) != external.object
                || next.resources[external.resource].object.zone
                    != self.resources[external.resource].object.zone
                || departed && external.object == reference
            {
                return false;
            }
        }
        next.visit_mana_cost_departures(reference, departed.then_some(destination), external, emit)
    }
    pub(super) fn visit_mana_cost_departures(
        &self,
        source: ObjectRef,
        source_departure: Option<Zone>,
        external: Option<(ExternalManaCost, AdditionalCost)>,
        emit: &mut dyn FnMut(&Self) -> bool,
    ) -> bool {
        // Match pay_activation followed by pay_additional_cost. These are
        // ordered cost components; a source's departure can remove a replacement
        // or change control before the selected permanent's departure.
        let mut after_source = |world: &Self| {
            if let Some((selected, spec)) = external {
                if !world.cost_object(spec, selected.object, source) {
                    return false;
                }
                world.cost_departures(&[selected.object], selected.destination(), emit)
            } else {
                emit(world)
            }
        };
        if let Some(destination) = source_departure {
            self.cost_departures(&[source], destination, &mut after_source)
        } else if external.is_some() {
            after_source(self)
        } else {
            self.visit_linked_returns(&mut after_source)
        }
    }
    pub(super) fn set_funding_continuation(
        &mut self,
        head: &'g Announcement,
        outer: &'g [mana_announcements::ManaAnnouncement],
        quality: Option<&'g mana_lots::Quality>,
    ) {
        self.continuation = (head.production.is_some() && !outer.is_empty()).then(|| {
            Box::new(PaymentContinuation {
                head,
                outer,
                quality,
            })
        });
    }
    pub(super) fn complete_funding_continuation(&self, goal: &FundingGoal) -> bool {
        let Some(current) = self.continuation.as_deref() else {
            return self.cost_completion();
        };
        let a = current.head;
        let Some(production) = a.production else {
            return self.cost_completion();
        };
        let Some((parent, outer)) = current.outer.split_last() else {
            return false;
        };
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return false;
        };
        let Some(resource) = self.resources.iter().position(|r| {
            r.id == a
                .item
                .source
                .expect("cost announcement has object source")
                .object
                .card
        }) else {
            return false;
        };
        let parent_goal = self.game.funding_goal(&parent.announcement);
        let purpose = self.payment_purpose(goal);
        let mut found = false;
        let pool = mana_lots::available(self.mana, &self.lots, purpose);
        funding_payments(pool, goal.required, goal.generic, |payment| {
            if found {
                return;
            }
            mana_lots::visit_allocations(self.mana, &self.lots, purpose, payment, |allocation| {
                if found {
                    return;
                }
                let external = a.additional.and_then(|object| {
                    self.resources
                        .iter()
                        .position(|r| {
                            r.id == object.card && r.object.generation == object.generation
                        })
                        .map(|resource| ExternalManaCost {
                            resource,
                            object,
                            kind: a.additional_cost().expect("selected mana cost").kind,
                        })
                });
                if a.additional.is_some() && external.is_none() {
                    return;
                }
                let step = FundingMove {
                    resource,
                    ability: Some(ability),
                    grantor: None,
                    borrowed: None,
                    payment,
                    counters: a.counter_payment.clone(),
                    production,
                    quality: current.quality.copied(),
                    allocation,
                    external,
                    frozen: Some(Box::new((
                        a.item
                            .source
                            .expect("cost announcement has object source")
                            .card,
                        a.item.borrowed_slot,
                    ))),
                };
                found = self.visit_applied(&step, &mut |w| {
                    let mut next = w.clone();
                    next.set_funding_continuation(
                        &parent.announcement,
                        outer,
                        parent.quality.as_ref().map(|(_, q)| q),
                    );
                    next.set_purpose(self.game.purpose_for_item(&parent.announcement.item));
                    next.convoke = parent.announcement.item.kind == StackKind::Spell
                        && parent
                            .announcement
                            .item
                            .source
                            .expect("cost announcement has object source")
                            .card
                            .has_convoke();
                    next.set_payment_context(&parent.announcement);
                    next.reachable_with_evidence(
                        &parent_goal,
                        parent.announcement.evidence.as_ref().map(|e| e.threshold),
                        &mut HashSet::new(),
                    )
                });
            });
        });
        found
    }
}
