//! Earlier-layer count expressions; deliberately no derived-stat recursion.
use super::*;

impl Game {
    fn count_object_matches(
        filter: CountFilter,
        source: ObjectRef,
        owner: Player,
        id: CardId,
        o: &Object,
    ) -> bool {
        let zone = match filter.zone {
            CountZone::Battlefield => Zone::Battlefield,
            CountZone::Graveyard => Zone::Graveyard,
            CountZone::Exile => Zone::Exile,
        };
        o.zone == zone
            && match filter.relation {
                0 => o.owner == owner,
                1 => o.owner != owner,
                _ => true,
            }
            && (!filter.other
                || ObjectRef {
                    card: id,
                    generation: o.generation,
                } != source)
            && (!filter.cards_only || !o.card.is_token())
            && (filter.kind == 0 || o.card.kind_mask() & filter.kind != 0)
            && o.card.kind_mask() & filter.exclude == 0
            && filter.subtype.is_none_or(|s| o.card.has_subtype(s))
            && (!filter.basic || o.card.is_basic_land())
    }
    pub(super) fn stat_value(&self, value: StatValue, source: ObjectRef, owner: Player) -> i128 {
        self.stat_value_with(
            value,
            source,
            owner,
            &|| {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .map(|(id, o)| (id as CardId, o))
            },
            self.state.players[owner.index()].life,
            self.state.attack_declared
                && self.state.attackers.iter().any(|a| a.card == source.card),
        )
    }
    /// The same earlier-layer evaluator serves concrete state and a borrowed
    /// counterfactual object overlay. No derived keywords or stats are queried.
    pub(super) fn stat_value_with<'a, F, I>(
        &self,
        value: StatValue,
        source: ObjectRef,
        owner: Player,
        objects: &F,
        life: i128,
        attacking: bool,
    ) -> i128
    where
        F: Fn() -> I,
        I: Iterator<Item = (CardId, &'a Object)>,
    {
        match value {
            StatValue::Not(value) => i128::from(
                self.stat_value_with(*value, source, owner, objects, life, attacking) == 0,
            ),
            StatValue::CardsDrawnThisTurn => {
                i128::from(self.state.players[owner.index()].cards_drawn_this_turn)
            }
            StatValue::AttackingCreatures => objects()
                .filter(|(id, o)| {
                    o.owner == owner
                        && o.zone == Zone::Battlefield
                        && o.card.is_creature()
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
                    o.card.is_token() && Self::count_object_matches(filter, source, owner, id, o)
                })
                .count() as i128,
            StatValue::AtLeast { value, minimum } => i128::from(
                self.stat_value_with(*value, source, owner, objects, life, attacking)
                    >= i128::from(minimum),
            ),
            StatValue::SpellsCastThisTurn => {
                i128::from(self.state.players[owner.index()].spells_cast_this_turn)
            }
            StatValue::LifeGainOccurredThisTurn => {
                i128::from(self.state.players[owner.index()].life_gained_this_turn > 0)
            }
            StatValue::ControllerLife => life,
            StatValue::Colors(filter) | StatValue::CardTypes(filter) => objects()
                .filter(|&(id, o)| Self::count_object_matches(filter, source, owner, id, o))
                .fold(0u16, |mask, (_, object)| {
                    mask | if matches!(value, StatValue::Colors(_)) {
                        u16::from(object.card.colors())
                    } else {
                        object.card.kind_mask()
                    }
                })
                .count_ones()
                as i128,
            StatValue::Count(filter) => i128::try_from(
                objects()
                    .filter(|&(id, o)| Self::count_object_matches(filter, source, owner, id, o))
                    .count(),
            )
            .capacity("stat count exhausted"),
            StatValue::DistinctNames(filter) => {
                let mut names: Vec<_> = objects()
                    .filter(|&(id, o)| Self::count_object_matches(filter, source, owner, id, o))
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
                    filters
                        .iter()
                        .any(|&filter| Self::count_object_matches(filter, source, owner, id, o))
                })
                .count() as i128,
            StatValue::GreatestManaValue(filter) => i128::from(
                objects()
                    .filter(|&(id, o)| Self::count_object_matches(filter, source, owner, id, o))
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
                .map(|&value| self.stat_value_with(value, source, owner, objects, life, attacking))
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
            AmountValue::ChosenX => 0, // Off-stack expressions have no cast parameter.
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
    pub(super) fn stat_expression(&self, expr: StatExpr, source: ObjectRef, owner: Player) -> i128 {
        self.stat_value(expr.value, source, owner)
            .checked_mul(expr.multiplier)
            .and_then(|v| v.checked_add(expr.offset))
            .capacity("stat expression exhausted")
    }
}
