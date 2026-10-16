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
            StatValue::Count(filter) => i128::try_from(
                (0..self.state.objects.len())
                    .filter(|&id| self.count_matches(filter, source, owner, id as CardId))
                    .count(),
            )
            .capacity("stat count exhausted"),
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
    pub(super) fn stat_expression(&self, expr: StatExpr, source: ObjectRef, owner: Player) -> i128 {
        self.stat_value(expr.value, source, owner)
            .checked_mul(expr.multiplier)
            .and_then(|v| v.checked_add(expr.offset))
            .capacity("stat expression exhausted")
    }
}
