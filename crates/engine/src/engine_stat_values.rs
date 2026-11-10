//! Earlier-layer count expressions; deliberately no derived-stat recursion.
use super::*;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn count_object_matches(
        filter: CountFilter,
        source: ObjectRef,
        owner: Player,
        id: CardId,
        o: &Object,
        subtypes: &impl Fn(CardId) -> GrantedSubtypes,
        kinds: &impl Fn(CardId, &Object) -> u16,
    ) -> bool {
        let zone = match filter.zone {
            CountZone::Battlefield => Zone::Battlefield,
            CountZone::Graveyard => Zone::Graveyard,
            CountZone::Exile => Zone::Exile,
        };
        let related_player = if zone == Zone::Battlefield {
            o.controller
        } else {
            o.owner
        };
        o.zone == zone
            && match filter.relation {
                0 => related_player == owner,
                1 => related_player != owner,
                _ => true,
            }
            && (!filter.other
                || ObjectRef {
                    card: id,
                    generation: o.generation,
                } != source)
            && (!filter.cards_only || o.is_card())
            && (filter.kind == 0 || kinds(id, o) & filter.kind != 0)
            && (filter.exclude == 0 || kinds(id, o) & filter.exclude == 0)
            && filter.subtype.is_none_or(|s| subtypes(id).has(o.card, s))
            && (!filter.basic || (kinds(id, o) & 2 != 0 && o.card.is_basic_land()))
    }
    pub(super) fn stat_value(&self, value: StatValue, source: ObjectRef, owner: Player) -> i128 {
        self.stat_value_in_world(self, value, source, owner)
    }

    pub(super) fn stat_value_in_world<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        value: StatValue,
        source: ObjectRef,
        owner: Player,
    ) -> i128 {
        if value == StatValue::CreaturesDiedThisTurn {
            return i128::from(world.creatures_died());
        }
        if value == StatValue::CardLeftGraveyardThisTurn {
            return i128::from(world.card_left_graveyard(owner));
        }
        if value == StatValue::LifeGainOccurredThisTurn {
            return i128::from(world.gained_life(owner));
        }
        self.stat_value_with(
            value,
            source,
            owner,
            &|| {
                self.state.objects.iter().enumerate().map(|(id, _)| {
                    let id = id as CardId;
                    (id, world.object(id))
                })
            },
            &|id| world.subtypes(id),
            &|id, _| world.kinds(id),
            &|id| world.name(id),
            &|id| world.colors(id),
            &|id| world.legendary(id),
            &|id| world.mana_value(id),
            &|id, kind| world.counter_count(id, kind),
            &|id| world.counter_amounts(id).has_any(),
            world.life(owner),
            world.life(owner.other()),
            self.state.attack_declared && world.attacking(source.card),
            world.has_enduring_story(owner),
            world.lost_life(owner),
            world.card_left_graveyard(owner),
        )
    }
    /// The same earlier-layer evaluator serves concrete state and a borrowed
    /// counterfactual object overlay. No derived keywords or stats are queried.
    #[allow(clippy::too_many_arguments)] // explicit borrowed type and object overlays
    pub(super) fn stat_value_with<'a, F, I>(
        &self,
        value: StatValue,
        source: ObjectRef,
        owner: Player,
        objects: &F,
        subtypes: &impl Fn(CardId) -> GrantedSubtypes,
        kinds: &impl Fn(CardId, &Object) -> u16,
        names: &impl Fn(CardId) -> &'static str,
        colors: &impl Fn(CardId) -> u8,
        legendary: &impl Fn(CardId) -> bool,
        mana_value: &impl Fn(CardId) -> u64,
        counter_count: &impl Fn(CardId, CounterKind) -> u64,
        has_counters: &impl Fn(CardId) -> bool,
        life: i128,
        opponent_life: i128,
        attacking: bool,
        enduring_story: bool,
        life_lost: bool,
        graveyard_left: bool,
    ) -> i128
    where
        F: Fn() -> I,
        I: Iterator<Item = (CardId, &'a Object)>,
    {
        match value {
            StatValue::CreatureEntriesThisTurn { other } => {
                self.creature_entries_this_turn(owner, source, other)
            }
            StatValue::DeclaredSubtypeAttackersThisTurn(subtype) => {
                self.declared_subtype_attackers(owner, subtype) as i128
            }
            StatValue::ArtifactEnteredThisTurn => i128::from(
                self.state.players[owner.index()]
                    .turn_facts
                    .artifact_entered,
            ),
            StatValue::SourceClassLevelAtLeast(level) => {
                i128::from(objects().any(|(id, object)| {
                    id == source.card
                        && object.generation == source.generation
                        && object.zone == Zone::Battlefield
                        && object.class_level >= level
                }))
            }
            StatValue::SourceHasNotDealtCombatDamage => objects()
                .find(|(id, object)| *id == source.card && object.generation == source.generation)
                .map_or(0, |(_, object)| {
                    i128::from(!object.dealt_combat_damage_this_incarnation)
                }),
            StatValue::SourceHasNotDealtDamage => objects()
                .find(|(id, object)| *id == source.card && object.generation == source.generation)
                .map_or(0, |(_, object)| {
                    i128::from(!object.dealt_damage_this_incarnation)
                }),
            StatValue::MulticoloredPermanentsYouControl => objects()
                .filter(|(id, object)| {
                    object.zone == Zone::Battlefield
                        && object.controller == owner
                        && colors(*id).count_ones() >= 2
                })
                .count() as i128,
            StatValue::CounterBearingCreatures => objects()
                .filter(|(id, object)| {
                    object.zone == Zone::Battlefield
                        && object.controller == owner
                        && kinds(*id, object) & 1 != 0
                        && has_counters(*id)
                })
                .count() as i128,
            StatValue::ControlledCreaturesWithCounter(kind) => objects()
                .filter(|&(id, object)| {
                    object.zone == Zone::Battlefield
                        && object.controller == owner
                        && kinds(id, object) & 1 != 0
                        && counter_count(id, kind) > 0
                })
                .count() as i128,
            StatValue::DistinctCreatureCounterKinds => {
                // No allocation or source-owned counter copy, including sparse kinds.
                let mut present = [false; CounterKind::ALL.len()];
                for (id, object) in objects() {
                    if object.zone != Zone::Battlefield
                        || object.controller != owner
                        || kinds(id, object) & 1 == 0
                        || !has_counters(id)
                    {
                        continue;
                    }
                    for (index, kind) in CounterKind::ALL.into_iter().enumerate() {
                        present[index] |= counter_count(id, kind) != 0;
                    }
                }
                present.into_iter().filter(|present| *present).count() as i128
            }
            StatValue::DistinctControlledPlaneswalkerTypes => {
                let mut distinct = std::collections::BTreeSet::new();
                for (id, object) in objects() {
                    if object.zone != Zone::Battlefield
                        || object.controller != owner
                        || kinds(id, object) & 64 == 0
                    {
                        continue;
                    }
                    let live = subtypes(id);
                    for &subtype in subtypes::PLANESWALKER_TYPES {
                        if live.has(object.card, subtype) {
                            distinct.insert(subtype);
                        }
                    }
                }
                i128::try_from(distinct.len()).capacity("planeswalker type count exhausted")
            }
            StatValue::Fixed(n) => i128::from(n),
            StatValue::HasEnduringStory => i128::from(enduring_story),
            StatValue::SourceTotalCounters => objects()
                .find(|(id, object)| *id == source.card && object.generation == source.generation)
                .map_or(0, |_| {
                    CounterKind::ALL
                        .into_iter()
                        .map(|kind| i128::from(counter_count(source.card, kind)))
                        .sum()
                }),
            StatValue::Parity { value, odd } => i128::from(
                (self.stat_value_with(
                    *value,
                    source,
                    owner,
                    objects,
                    subtypes,
                    kinds,
                    names,
                    colors,
                    legendary,
                    mana_value,
                    counter_count,
                    has_counters,
                    life,
                    opponent_life,
                    attacking,
                    enduring_story,
                    life_lost,
                    graveyard_left,
                ) % 2
                    != 0)
                    == odd,
            ),
            StatValue::SourceCounters(kind) => objects()
                .find(|(id, object)| *id == source.card && object.generation == source.generation)
                .map_or(0, |_| i128::from(counter_count(source.card, kind))),
            StatValue::LegendaryCreaturesYouControl
            | StatValue::OtherLegendaryCreaturesYouControl => objects()
                .filter(|(id, object)| {
                    object.zone == Zone::Battlefield
                        && object.controller == owner
                        && (value == StatValue::LegendaryCreaturesYouControl
                            || ObjectRef {
                                card: *id,
                                generation: object.generation,
                            } != source)
                        && kinds(*id, object) & 1 != 0
                        && legendary(*id)
                })
                .count() as i128,
            StatValue::TappedCount(filter) => objects()
                .filter(|&(id, o)| {
                    o.tapped
                        && Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                })
                .count() as i128,
            StatValue::Not(value) => i128::from(
                self.stat_value_with(
                    *value,
                    source,
                    owner,
                    objects,
                    subtypes,
                    kinds,
                    names,
                    colors,
                    legendary,
                    mana_value,
                    counter_count,
                    has_counters,
                    life,
                    opponent_life,
                    attacking,
                    enduring_story,
                    life_lost,
                    graveyard_left,
                ) == 0,
            ),
            StatValue::SourceEquipmentCount | StatValue::SourceAuraOrEquipmentCount => {
                if !objects().any(|(id, object)| {
                    id == source.card
                        && object.generation == source.generation
                        && object.zone == Zone::Battlefield
                }) {
                    return 0;
                }
                objects()
                    .filter(|(id, object)| {
                        object.zone == Zone::Battlefield && object.attached_to == Some(source) && {
                            let types = subtypes(*id);
                            types.has(object.card, "Equipment")
                                || value == StatValue::SourceAuraOrEquipmentCount
                                    && types.has(object.card, "Aura")
                        }
                    })
                    .count() as i128
            }
            StatValue::ControllerDomain => {
                let mut mask = 0u8;
                for (id, object) in objects() {
                    if object.zone != Zone::Battlefield
                        || object.controller != owner
                        || kinds(id, object) & 2 == 0
                    {
                        continue;
                    }
                    let types = subtypes(id);
                    for (bit, name) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                        .iter()
                        .enumerate()
                    {
                        if types.has(object.card, name) {
                            mask |= 1 << bit;
                        }
                    }
                }
                i128::from(mask.count_ones())
            }
            StatValue::ControllerHandSize => objects()
                .filter(|(_, o)| o.owner == owner && o.zone == Zone::Hand && o.is_card())
                .count() as i128,
            StatValue::MountOrVehicleEntriesThisTurn => {
                i128::from(self.state.players[owner.index()].mount_or_vehicle_entries_this_turn)
            }
            StatValue::NonlandEntriesThisTurn => {
                i128::from(self.state.players[owner.index()].nonland_permanents_entered_this_turn)
            }
            StatValue::ControlledCreatureDiedThisTurn => {
                i128::from(self.state.players[owner.index()].controlled_creature_died_this_turn)
            }
            StatValue::CardLeftGraveyardThisTurn => i128::from(graveyard_left),
            StatValue::OwnExiledAdventurerCard => i128::from(objects().any(|(id, object)| {
                object.zone == Zone::Exile
                    && object.owner == owner
                    && self.is_adventurer_card(id, object)
            })),
            StatValue::GraveyardInstantSorceryAdventure => {
                adventure::graveyard_instant_sorcery_adventure_count(
                    owner,
                    objects().map(|(_, object)| object),
                ) as i128
            }
            StatValue::ControllerTurn => i128::from(self.state.active == owner),
            StatValue::ControllerEndStep => {
                i128::from(self.state.step == Step::End && self.state.active == owner)
            }
            StatValue::CardsDrawnThisTurn => {
                i128::from(self.state.players[owner.index()].cards_drawn_this_turn)
            }
            StatValue::AttackingCreatures => objects()
                .filter(|(id, o)| {
                    o.controller == owner
                        && o.zone == Zone::Battlefield
                        && kinds(*id, o) & 1 != 0
                        && self.state.attackers.iter().any(|a| a.card == *id)
                })
                .count() as i128,
            StatValue::AttackingSubtype(subtype) => objects()
                .filter(|(id, o)| {
                    o.controller == owner
                        && o.zone == Zone::Battlefield
                        && (subtypes(*id).has(o.card, subtype))
                        && self.state.attackers.iter().any(|a| a.card == *id)
                })
                .count() as i128,
            StatValue::OtherNamedCreatures(name) => objects()
                .filter(|(id, o)| {
                    o.controller == owner
                        && o.zone == Zone::Battlefield
                        && kinds(*id, o) & 1 != 0
                        && crate::card_names::same_name(names(*id), name)
                        && (*id != source.card || o.generation != source.generation)
                })
                .count() as i128,
            StatValue::AllAttackingCreatures => objects()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && kinds(*id, o) & 1 != 0
                        && self.state.attackers.iter().any(|a| a.card == *id)
                })
                .count() as i128,
            StatValue::SourceAttacking => i128::from(
                attacking
                    && objects().any(|(id, o)| {
                        id == source.card
                            && o.generation == source.generation
                            && o.zone == Zone::Battlefield
                    }),
            ),
            StatValue::SourceUntapped => i128::from(objects().any(|(id, object)| {
                id == source.card
                    && object.generation == source.generation
                    && object.zone == Zone::Battlefield
                    && !object.tapped
            })),
            StatValue::CreaturesAttackingYou => objects()
                .filter(|(id, object)| {
                    object.zone == Zone::Battlefield
                        && kinds(*id, object) & 1 != 0
                        && self.state.attackers.iter().any(|attack| {
                            attack.card == *id
                                && attack.defender == Some(CombatDefender::Player(owner))
                        })
                })
                .count() as i128,
            StatValue::SourceModified | StatValue::ControlledModifiedCreatures => {
                let modified = |id, object: &Object| {
                    Self::object_modified_with(
                        ObjectRef {
                            card: id,
                            generation: object.generation,
                        },
                        has_counters(id),
                        objects(),
                        |id, attachment| {
                            let kind = kinds(id, attachment);
                            (kind & 16 != 0 && subtypes(id).has(attachment.card, "Equipment"))
                                || (kind & 32 != 0
                                    && attachment.controller == object.controller
                                    && subtypes(id).has(attachment.card, "Aura"))
                        },
                    )
                };
                let modified_creature = |id, object: &Object| {
                    object.zone == Zone::Battlefield
                        && kinds(id, object) & 1 != 0
                        && modified(id, object)
                };
                if value == StatValue::SourceModified {
                    i128::from(objects().any(|(id, object)| {
                        id == source.card
                            && object.generation == source.generation
                            && modified_creature(id, object)
                    }))
                } else {
                    objects()
                        .filter(|&(id, object)| {
                            object.controller == owner && modified_creature(id, object)
                        })
                        .count() as i128
                }
            }
            StatValue::Tokens(filter) => objects()
                .filter(|&(id, o)| {
                    o.is_token()
                        && Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                })
                .count() as i128,
            StatValue::AtLeast { value, minimum } => i128::from(
                self.stat_value_with(
                    *value,
                    source,
                    owner,
                    objects,
                    subtypes,
                    kinds,
                    names,
                    colors,
                    legendary,
                    mana_value,
                    counter_count,
                    has_counters,
                    life,
                    opponent_life,
                    attacking,
                    enduring_story,
                    life_lost,
                    graveyard_left,
                ) >= i128::from(minimum),
            ),
            StatValue::PermanentsSacrificedThisTurn => {
                i128::from(self.state.players[owner.index()].permanents_sacrificed_this_turn)
            }
            StatValue::SpellsCastThisTurn => {
                i128::from(self.state.players[owner.index()].spells_cast_this_turn)
            }
            StatValue::CreatureSpellsCastThisTurn => {
                i128::from(self.state.players[owner.index()].creature_spells_cast_this_turn)
            }
            StatValue::NoncreatureSpellsCastThisTurn => {
                let player = &self.state.players[owner.index()];
                i128::from(
                    player
                        .spells_cast_this_turn
                        .checked_sub(player.creature_spells_cast_this_turn)
                        .expect("creature spell count cannot exceed total spell count"),
                )
            }
            StatValue::CreaturesDiedThisTurn => i128::from(self.state.creatures_died_this_turn),
            StatValue::LifeGainOccurredThisTurn => {
                i128::from(self.state.players[owner.index()].life_gained_this_turn > 0)
            }
            StatValue::OpponentNoncombatDamageThisTurn => {
                i128::from(self.state.players[owner.other().index()].noncombat_damage_this_turn)
            }
            StatValue::LinkedExileCardTypes => {
                let mut mask = 0u16;
                if let Some(links) = &self.state.linked_choices {
                    for link in links.links.iter().filter(|link| link.source == source) {
                        if let Some((id, object)) = objects().find(|(id, object)| {
                            *id == link.exiled.card
                                && object.generation == link.exiled.generation
                                && object.zone == Zone::Exile
                        }) {
                            mask |= kinds(id, object);
                        }
                    }
                }
                i128::from(mask.count_ones())
            }
            StatValue::LifeLostThisTurn => i128::from(life_lost),
            StatValue::LifeGainedThisTurn => {
                i128::try_from(self.state.players[owner.index()].life_gained_this_turn)
                    .capacity("life gain count representation exhausted")
            }
            StatValue::ControllerLife => life,
            StatValue::ControllerSpeed => i128::from(self.state.players[owner.index()].speed),
            StatValue::OpponentControlsMoreLands => {
                let mut counts = [0usize; 2];
                for (id, object) in objects() {
                    if object.zone == Zone::Battlefield && kinds(id, object) & 2 != 0 {
                        counts[object.controller.index()] += 1;
                    }
                }
                i128::from(counts[owner.other().index()] > counts[owner.index()])
            }
            StatValue::OpponentLifeAtMost(maximum) => {
                i128::from(opponent_life <= i128::from(maximum))
            }
            StatValue::ControllerLifeAboveStarting { minimum } => {
                i128::from(life >= STARTING_LIFE + i128::from(minimum))
            }
            StatValue::ControllerLifeAtMostHalfStarting => i128::from(life <= STARTING_LIFE / 2),
            StatValue::HasColor { filter, color } => i128::from(objects().any(|(id, o)| {
                Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    && colors(id) & (1u8 << color.index()) != 0
            })),
            StatValue::Colors(filter) | StatValue::CardTypes(filter) => objects()
                .filter(|&(id, o)| {
                    Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                })
                .fold(0u16, |mask, (id, object)| {
                    mask | if matches!(value, StatValue::Colors(_)) {
                        u16::from(colors(id))
                    } else {
                        kinds(id, object)
                    }
                })
                .count_ones()
                as i128,
            StatValue::NamedCount { filter, name } => objects()
                .filter(|&(id, o)| {
                    Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                        && crate::card_names::same_name(names(id), name)
                })
                .count() as i128,
            StatValue::CountAllKinds(filter) => i128::try_from(
                objects()
                    .filter(|&(id, o)| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                            && kinds(id, o) & filter.kind == filter.kind
                    })
                    .count(),
            )
            .capacity("stat count exhausted"),
            StatValue::Count(filter) => i128::try_from(
                objects()
                    .filter(|&(id, o)| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    })
                    .count(),
            )
            .capacity("stat count exhausted"),
            StatValue::DistinctManaValues(filter) => {
                let mut values: Vec<_> = objects()
                    .filter(|&(id, o)| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    })
                    .map(|(id, _)| mana_value(id))
                    .collect();
                values.sort_unstable();
                values.dedup();
                values.len() as i128
            }
            StatValue::DistinctNames(filter) => {
                let mut names: Vec<_> = objects()
                    .filter(|&(id, o)| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    })
                    .map(|(id, _)| crate::card_names::comparison_key(names(id)))
                    .filter(|name| !name.is_empty())
                    .collect();
                names.sort_unstable();
                names.dedup();
                names.len() as i128
            }
            StatValue::CountUnion(filters) => objects()
                .filter(|&(id, o)| {
                    filters.iter().any(|&filter| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    })
                })
                .count() as i128,
            StatValue::GreatestManaValue(filter) => i128::from(
                objects()
                    .filter(|&(id, o)| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    })
                    .map(|(id, _)| mana_value(id))
                    .max()
                    .unwrap_or(0),
            ),
            StatValue::TotalManaValue(filter) => objects()
                .filter(|&(id, o)| {
                    Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                })
                .try_fold(0i128, |total, (id, _)| {
                    total.checked_add(i128::from(mana_value(id)))
                })
                .capacity("total mana value exhausted"),
            StatValue::GraveyardCardTypes => objects()
                .filter(|(_, o)| o.zone == Zone::Graveyard && o.is_card())
                .fold(0u16, |mask, (_, o)| mask | o.card.kind_mask())
                .count_ones() as i128,
            StatValue::GroupsOf { value, size } => {
                self.stat_value_with(
                    *value,
                    source,
                    owner,
                    objects,
                    subtypes,
                    kinds,
                    names,
                    colors,
                    legendary,
                    mana_value,
                    counter_count,
                    has_counters,
                    life,
                    opponent_life,
                    attacking,
                    enduring_story,
                    life_lost,
                    graveyard_left,
                )
                .max(0)
                    / i128::from(size.get())
            }
            StatValue::Sum(values) => values
                .iter()
                .map(|&value| {
                    self.stat_value_with(
                        value,
                        source,
                        owner,
                        objects,
                        subtypes,
                        kinds,
                        names,
                        colors,
                        legendary,
                        mana_value,
                        counter_count,
                        has_counters,
                        life,
                        opponent_life,
                        attacking,
                        enduring_story,
                        life_lost,
                        graveyard_left,
                    )
                })
                .try_fold(0i128, i128::checked_add)
                .capacity("stat sum exhausted"),
        }
    }
    pub(super) fn nonbasic_land_count(&self, controller: Player) -> usize {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller == controller
                    && self.current_kinds(*id as CardId) & 2 != 0
                    && !object.card.is_basic_land()
            })
            .count()
    }

    pub(super) fn amount_value(
        &self,
        value: AmountValue,
        source: ObjectRef,
        owner: Player,
    ) -> u128 {
        match value {
            AmountValue::Sum(values) => values.iter().fold(0u128, |total, value| {
                total
                    .checked_add(self.amount_value(*value, source, owner))
                    .capacity("effect sum exhausted")
            }),
            AmountValue::Fixed(n) => u128::from(n),
            AmountValue::SourceDamageReceivedThisTurn => self.damage_received_this_turn(source),
            AmountValue::EquippedCreaturesYouControl => self.equipped_creature_count(owner),
            AmountValue::NonbasicLandsYouControl => self.nonbasic_land_count(owner) as u128,
            AmountValue::Devotion(color) => self
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, object)| {
                    let reference = self.object_ref(*id as CardId);
                    object.zone == Zone::Battlefield
                        && object.controller == owner
                        && !self.is_face_down(reference)
                        && !self
                            .copy_recipe(reference)
                            .is_some_and(|recipe| recipe.exceptions.embalm)
                })
                .map(|(_, object)| u128::from(object.card.mana_symbols_of_color(color)))
                .sum(),
            AmountValue::GreatestCreaturePower(filter) => {
                let query = self.characteristic_query();
                (0..self.state.objects.len())
                    .map(|id| id as CardId)
                    .filter(|&id| query.creature_matches(filter, owner, source, id))
                    .map(|id| query.stats(id).power)
                    .max()
                    .unwrap_or(0)
                    .max(0) as u128
            }
            AmountValue::GreatestCreatureCardPowerInGraveyard => {
                let query = self.characteristic_query();
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, object)| {
                        object.owner == owner
                            && object.zone == Zone::Graveyard
                            && object.is_card()
                            && object.card.is_creature()
                    })
                    .map(|(id, _)| query.stats(id as CardId).power)
                    .max()
                    .unwrap_or(0)
                    .max(0) as u128
            }
            AmountValue::GreatestCreatureToughness => {
                ability_world::CharacteristicWorld::greatest_creature_toughness(self, owner).max(0)
                    as u128
            }
            AmountValue::OpponentCreaturesExiledThisTurn => u128::from(
                self.state.players[owner.other().index()].controlled_creatures_exiled_this_turn,
            ),
            AmountValue::ControllerHandDeficit(n) => {
                u128::from(n).saturating_sub(self.hand_count(owner) as u128)
            }
            AmountValue::SourceTotalCounters => self
                .stat_value(StatValue::SourceTotalCounters, source, owner)
                .max(0) as u128,
            AmountValue::SourceCounterQuantity(kind) => {
                u128::from(self.source_counter_quantity_live(source, kind).unwrap_or(0))
            }
            AmountValue::PaidSourceCounterQuantity(_)
            | AmountValue::TargetPlayerHandSize
            | AmountValue::TargetPlayerLibraryToGraveyardThisTurn
            | AmountValue::TargetPlayerCreatureCount
            | AmountValue::MilledCreatures
            | AmountValue::MovedCount
            | AmountValue::RememberedCount
            | AmountValue::PaidAdditionalCostCount
            | AmountValue::PaidAdditionalCostSelectionCount
            | AmountValue::ChosenX
            | AmountValue::ReflexiveTargetLimit
            | AmountValue::ManaSpent
            | AmountValue::CastCardTypeCount
            | AmountValue::CastSpellManaValue
            | AmountValue::CastSpellManaSymbols(_)
            | AmountValue::SpellColors
            | AmountValue::ColorsSpent
            | AmountValue::SacrificedPower
            | AmountValue::ReflexiveSacrificedPower
            | AmountValue::SacrificeCostPower
            | AmountValue::SacrificeCostManaValueWithBonus(_)
            | AmountValue::ResolutionSacrificedManaValue
            | AmountValue::TappedCostCreaturePower
            | AmountValue::ChosenCostCreaturePower
            | AmountValue::TargetCreaturePower
            | AmountValue::SourceLastKnownPower
            | AmountValue::SacrificedToughness
            | AmountValue::SourcePower
            | AmountValue::SourceToughness
            | AmountValue::SourceEquipmentCount
            | AmountValue::DeclaredAttackerCount
            | AmountValue::EventObjectTotalCounters
            | AmountValue::LifeEventAmount
            | AmountValue::DiscardEventAmount
            | AmountValue::DamageEventAmount
            | AmountValue::CounterPlacementAmount
            | AmountValue::CreatureCombatDamageAmount
            | AmountValue::DepartureCounterCount(_)
            | AmountValue::DeathEventPower
            | AmountValue::DeathEventCounterTotal
            | AmountValue::DeathEventToughness
            | AmountValue::DamageReceivedAmount => 0, // Off-stack expressions have no cast parameter.
            AmountValue::Stat(value) => self.stat_value(value, source, owner).max(0) as u128,
            AmountValue::CardsDiscardedThisTurn => {
                u128::from(self.state.players[owner.index()].cards_discarded_this_turn)
            }
            AmountValue::LifeGainedThisTurn => {
                self.state.players[owner.index()].life_gained_this_turn
            }
            AmountValue::LifeLostThisTurn => self.state.players[owner.index()].life_lost_this_turn,
            AmountValue::Divided { value, divisor } => {
                self.amount_value(*value, source, owner) / u128::from(divisor.get())
            }
            AmountValue::Multiple { value, factor } => self
                .amount_value(*value, source, owner)
                .checked_mul(u128::from(factor))
                .capacity("effect amount representation exhausted"),
        }
    }
}
