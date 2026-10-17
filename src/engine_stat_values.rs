//! Earlier-layer count expressions; deliberately no derived-stat recursion.
use super::*;

impl Game {
    fn count_object_matches(
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
            && (!filter.cards_only || !o.card.is_token())
            && (filter.kind == 0 || kinds(id, o) & filter.kind != 0)
            && kinds(id, o) & filter.exclude == 0
            && filter
                .subtype
                .is_none_or(|s| o.card.has_subtype(s) || subtypes(id).contains(s))
            && (!filter.basic || o.card.is_basic_land())
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
            self.state.players[owner.index()].life,
            self.state.attack_declared
                && self.state.attackers.iter().any(|a| a.card == source.card),
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
        life: i128,
        attacking: bool,
    ) -> i128
    where
        F: Fn() -> I,
        I: Iterator<Item = (CardId, &'a Object)>,
    {
        match value {
            StatValue::Fixed(n) => i128::from(n),
            StatValue::TappedCount(filter) => objects()
                .filter(|&(id, o)| {
                    o.tapped
                        && Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                })
                .count() as i128,
            StatValue::Not(value) => i128::from(
                self.stat_value_with(
                    *value, source, owner, objects, subtypes, kinds, life, attacking,
                ) == 0,
            ),
            StatValue::ControllerHandSize => objects()
                .filter(|(_, o)| o.owner == owner && o.zone == Zone::Hand)
                .count() as i128,
            StatValue::NonlandEntriesThisTurn => {
                i128::from(self.state.players[owner.index()].nonland_permanents_entered_this_turn)
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
                        && (o.card.has_subtype(subtype) || subtypes(*id).contains(subtype))
                        && self.state.attackers.iter().any(|a| a.card == *id)
                })
                .count() as i128,
            StatValue::OtherNamedCreatures(name) => objects()
                .filter(|(id, o)| {
                    o.controller == owner
                        && o.zone == Zone::Battlefield
                        && kinds(*id, o) & 1 != 0
                        && o.card.name() == name
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
                        Self::object_modified_with(source, o, objects().map(|(_, o)| o))
                    }),
            ),
            StatValue::Tokens(filter) => objects()
                .filter(|&(id, o)| {
                    o.card.is_token()
                        && Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                })
                .count() as i128,
            StatValue::AtLeast { value, minimum } => i128::from(
                self.stat_value_with(
                    *value, source, owner, objects, subtypes, kinds, life, attacking,
                ) >= i128::from(minimum),
            ),
            StatValue::SpellsCastThisTurn => {
                i128::from(self.state.players[owner.index()].spells_cast_this_turn)
            }
            StatValue::LifeGainOccurredThisTurn => {
                i128::from(self.state.players[owner.index()].life_gained_this_turn > 0)
            }
            StatValue::LifeGainedThisTurn => {
                i128::try_from(self.state.players[owner.index()].life_gained_this_turn)
                    .capacity("life gain count representation exhausted")
            }
            StatValue::ControllerLife => life,
            StatValue::Colors(filter) | StatValue::CardTypes(filter) => objects()
                .filter(|&(id, o)| {
                    Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                })
                .fold(0u16, |mask, (id, object)| {
                    mask | if matches!(value, StatValue::Colors(_)) {
                        u16::from(object.card.colors())
                    } else {
                        kinds(id, object)
                    }
                })
                .count_ones()
                as i128,
            StatValue::Count(filter) => i128::try_from(
                objects()
                    .filter(|&(id, o)| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    })
                    .count(),
            )
            .capacity("stat count exhausted"),
            StatValue::DistinctNames(filter) => {
                let mut names: Vec<_> = objects()
                    .filter(|&(id, o)| {
                        Self::count_object_matches(filter, source, owner, id, o, subtypes, kinds)
                    })
                    .map(|(_, object)| {
                        object
                            .card
                            .token_info()
                            .map_or_else(|| object.card.name(), |info| info.name)
                    })
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
                    .map(|(_, o)| o.card.fixed_mana_value())
                    .max()
                    .unwrap_or(0),
            ),
            StatValue::GraveyardCardTypes => objects()
                .filter(|(_, o)| o.zone == Zone::Graveyard && !o.card.is_token())
                .fold(0u16, |mask, (_, o)| mask | o.card.kind_mask())
                .count_ones() as i128,
            StatValue::Sum(values) => values
                .iter()
                .map(|&value| {
                    self.stat_value_with(
                        value, source, owner, objects, subtypes, kinds, life, attacking,
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
            AmountValue::Fixed(n) => u128::from(n),
            AmountValue::MilledCreatures
            | AmountValue::ChosenX
            | AmountValue::ManaSpent
            | AmountValue::ColorsSpent
            | AmountValue::SourcePower => 0, // Off-stack expressions have no cast parameter.
            AmountValue::Stat(value) => self.stat_value(value, source, owner).max(0) as u128,
            AmountValue::CardsDiscardedThisTurn => {
                u128::from(self.state.players[owner.index()].cards_discarded_this_turn)
            }
            AmountValue::LifeGainedThisTurn => {
                self.state.players[owner.index()].life_gained_this_turn
            }
            AmountValue::Multiple { value, factor } => self
                .amount_value(*value, source, owner)
                .checked_mul(u128::from(factor))
                .capacity("effect amount representation exhausted"),
        }
    }
    pub(super) fn stat_expression_without_hand(
        &self,
        expr: StatExpr,
        source: ObjectRef,
        owner: Player,
        departing: Option<CardId>,
    ) -> i128 {
        self.stat_value_without_hand(expr.value, source, owner, departing)
            .checked_mul(expr.multiplier)
            .and_then(|v| v.checked_add(expr.offset))
            .capacity("stat expression exhausted")
    }
}
