//! Earlier-layer count expressions; deliberately no derived-stat recursion.
use super::*;

impl Game {
    fn count_matches(
        &self,
        filter: CountFilter,
        source: ObjectRef,
        owner: Player,
        id: CardId,
    ) -> bool {
        let o = &self.state.objects[id as usize];
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
            && (!filter.other || self.object_ref(id) != source)
            && (!filter.cards_only || !o.card.is_token())
            && (filter.kind == 0 || o.card.kind_mask() & filter.kind != 0)
            && o.card.kind_mask() & filter.exclude == 0
            && filter.subtype.is_none_or(|s| o.card.has_subtype(s))
            && (!filter.basic || o.card.is_basic_land())
    }
    pub(super) fn stat_value(&self, value: StatValue, source: ObjectRef, owner: Player) -> i128 {
        match value {
            StatValue::ControllerLife => self.state.players[owner.index()].life,
            StatValue::Colors(filter) | StatValue::CardTypes(filter) => {
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|&(id, _)| self.count_matches(filter, source, owner, id as CardId))
                    .fold(0u16, |mask, (_, object)| {
                        mask | if matches!(value, StatValue::Colors(_)) {
                            u16::from(object.card.colors())
                        } else {
                            object.card.kind_mask()
                        }
                    })
                    .count_ones() as i128
            }
            StatValue::Count(filter) => i128::try_from(
                (0..self.state.objects.len())
                    .filter(|&id| self.count_matches(filter, source, owner, id as CardId))
                    .count(),
            )
            .capacity("stat count exhausted"),
            StatValue::DistinctNames(filter) => {
                let mut names: Vec<_> = self
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|&(id, _)| self.count_matches(filter, source, owner, id as CardId))
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
            StatValue::CountUnion(filters) => (0..self.state.objects.len())
                .filter(|&id| {
                    filters
                        .iter()
                        .any(|&filter| self.count_matches(filter, source, owner, id as CardId))
                })
                .count() as i128,
            StatValue::GreatestManaValue(filter) => i128::from(
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|&(id, _)| self.count_matches(filter, source, owner, id as CardId))
                    .map(|(_, o)| o.card.fixed_mana_value())
                    .max()
                    .unwrap_or(0),
            ),
            StatValue::GraveyardCardTypes => self
                .state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Graveyard && !o.card.is_token())
                .fold(0u16, |mask, o| mask | o.card.kind_mask())
                .count_ones() as i128,
            StatValue::Sum(values) => values
                .iter()
                .map(|&value| self.stat_value(value, source, owner))
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
            AmountValue::Stat(value) => self.stat_value(value, source, owner).max(0) as u128,
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
