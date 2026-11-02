//! Earlier-layer count expressions; deliberately no derived-stat recursion.
use super::*;

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
            && kinds(id, o) & filter.exclude == 0
            && filter.subtype.is_none_or(|s| subtypes(id).has(o.card, s))
            && (!filter.basic || (kinds(id, o) & 2 != 0 && o.card.is_basic_land()))
    }
    pub(super) fn stat_value(&self, value: StatValue, source: ObjectRef, owner: Player) -> i128 {
        self.stat_value_without_hand(value, source, owner, None)
    }
    /// CR601.2a preflight removes only the exact proposed Hand spell. Plot and
    /// activation queries retain their real current objects until costs commit.
    pub(super) fn stat_value_without_hand(
        &self,
        value: StatValue,
        source: ObjectRef,
        owner: Player,
        departing: Option<CardId>,
    ) -> i128 {
        self.stat_value_with(
            value,
            source,
            owner,
            &|| {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(move |(id, o)| departing != Some(*id as CardId) || o.zone != Zone::Hand)
                    .map(|(id, o)| (id as CardId, o))
            },
            &|id| self.granted_subtypes(id),
            &|id, o| self.object_kinds(id, o),
            &|id| self.object_name(id),
            &|id| ability_world::CharacteristicWorld::colors(self, id),
            &|id| ability_world::CharacteristicWorld::legendary(self, id),
            &|id| ability_world::CharacteristicWorld::mana_value(self, id),
            &|id, kind| self.counter_count(id, kind),
            &|id| self.counter_amounts(id).has_any(),
            self.state.players[owner.index()].life,
            self.state.players[owner.other().index()].life,
            self.state.attack_declared
                && self.state.attackers.iter().any(|a| a.card == source.card),
            self.state.players[owner.index()].enduring_story,
            self.state.players[owner.index()].life_lost_this_turn > 0,
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
    ) -> i128
    where
        F: Fn() -> I,
        I: Iterator<Item = (CardId, &'a Object)>,
    {
        match value {
            StatValue::SourceHasNotDealtCombatDamage => objects()
                .find(|(id, object)| *id == source.card && object.generation == source.generation)
                .map_or(0, |(_, object)| {
                    i128::from(!object.dealt_combat_damage_this_incarnation)
                }),
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
            StatValue::SourceCounters(kind) => objects()
                .find(|(id, object)| *id == source.card && object.generation == source.generation)
                .map_or(0, |_| i128::from(counter_count(source.card, kind))),
            StatValue::LegendaryCreaturesYouControl => objects()
                .filter(|(id, object)| {
                    object.zone == Zone::Battlefield
                        && object.controller == owner
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
                ) == 0,
            ),
            StatValue::SourceEquipmentCount => {
                if !objects().any(|(id, object)| {
                    id == source.card
                        && object.generation == source.generation
                        && object.zone == Zone::Battlefield
                }) {
                    return 0;
                }
                objects()
                    .filter(|(id, object)| {
                        object.zone == Zone::Battlefield
                            && object.attached_to == Some(source)
                            && subtypes(*id).has(object.card, "Equipment")
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
            StatValue::NonlandEntriesThisTurn => {
                i128::from(self.state.players[owner.index()].nonland_permanents_entered_this_turn)
            }
            StatValue::ControlledCreatureDiedThisTurn => {
                i128::from(self.state.players[owner.index()].controlled_creature_died_this_turn)
            }
            StatValue::CardLeftGraveyardThisTurn => {
                i128::from(self.state.players[owner.index()].card_left_graveyard_this_turn)
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
            StatValue::SourceModified => i128::from(
                objects()
                    .find(|(id, o)| *id == source.card && o.generation == source.generation)
                    .is_some_and(|(_, o)| {
                        Self::object_modified_with(
                            source,
                            o,
                            has_counters(source.card),
                            objects().map(|(_, o)| o),
                        )
                    }),
            ),
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
                .try_fold(0i128, |total, (id, object)| {
                    // Face-down cards in exile have no mana cost, even when
                    // their owner is permitted to look at the underlying face.
                    let hidden = object.zone == Zone::Exile
                        && self.state.linked_choices.as_ref().is_some_and(|choices| {
                            choices.hidden.iter().any(|(_, reference, _)| {
                                reference.card == id && reference.generation == object.generation
                            })
                        });
                    total.checked_add(if hidden {
                        0
                    } else {
                        i128::from(mana_value(id))
                    })
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
                    )
                })
                .try_fold(0i128, i128::checked_add)
                .capacity("stat sum exhausted"),
        }
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
            AmountValue::OpponentCreaturesExiledThisTurn => u128::from(
                self.state.players[owner.other().index()].controlled_creatures_exiled_this_turn,
            ),
            AmountValue::ControllerHandDeficit(n) => {
                u128::from(n).saturating_sub(self.hand_count(owner) as u128)
            }
            AmountValue::SourceCounterQuantity(kind) => {
                u128::from(self.source_counter_quantity_live(source, kind).unwrap_or(0))
            }
            AmountValue::PaidSourceCounterQuantity(_)
            | AmountValue::TargetPlayerHandSize
            | AmountValue::TargetPlayerLibraryToGraveyardThisTurn
            | AmountValue::TargetPlayerCreatureCount
            | AmountValue::MilledCreatures
            | AmountValue::MovedCount
            | AmountValue::ChosenX
            | AmountValue::ReflexiveTargetLimit
            | AmountValue::ManaSpent
            | AmountValue::CastSpellManaValue
            | AmountValue::CastSpellManaSymbols(_)
            | AmountValue::SpellColors
            | AmountValue::ColorsSpent
            | AmountValue::SacrificedPower
            | AmountValue::ReflexiveSacrificedPower
            | AmountValue::SacrificeCostPower
            | AmountValue::TappedCostCreaturePower
            | AmountValue::TargetCreaturePower
            | AmountValue::SourceLastKnownPower
            | AmountValue::SacrificedToughness
            | AmountValue::SourcePower
            | AmountValue::SourceToughness
            | AmountValue::SourceEquipmentCount
            | AmountValue::LifeEventAmount
            | AmountValue::DiscardEventAmount
            | AmountValue::DamageEventAmount
            | AmountValue::CreatureCombatDamageAmount
            | AmountValue::DamageReceivedAmount => 0, // Off-stack expressions have no cast parameter.
            AmountValue::Stat(value) => self.stat_value(value, source, owner).max(0) as u128,
            AmountValue::CardsDiscardedThisTurn => {
                u128::from(self.state.players[owner.index()].cards_discarded_this_turn)
            }
            AmountValue::LifeGainedThisTurn => {
                self.state.players[owner.index()].life_gained_this_turn
            }
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
