//! Exact-object layer-7b effects share the Role timestamp clock (CR613.4/613.7).
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct BaseStatsEffectView {
    pub recipient: ObjectRef,
    pub source: ObjectRef,
    pub power: Option<i128>,
    pub toughness: Option<i128>,
    pub timestamp: u64,
    pub static_source: bool,
}
impl Game {
    pub(super) fn register_static_base_stats(&mut self, id: CardId) {
        let Some(spec) = self.state.objects[id as usize].card.base_stat_setter() else {
            return;
        };
        let object = self.object_ref(id);
        let timestamp = self.next_role_timestamp();
        self.state
            .roles
            .get_or_insert_with(Default::default)
            .base_stats
            .push(BaseStatsEffectView {
                recipient: object,
                source: object,
                power: spec.power,
                toughness: spec.toughness,
                timestamp,
                static_source: true,
            });
    }
    pub(super) fn set_base_stats(
        &mut self,
        item: &SpellView,
        subject: Subject,
        power: Option<i128>,
        toughness: Option<i128>,
    ) {
        let recipients: Vec<_> = match subject {
            Subject::Source => (self.on_battlefield(item.card)
                && self.object_ref(item.card) == item.object)
                .then_some(item.object)
                .into_iter()
                .collect(),
            Subject::Attached => self
                .attached_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Entering => self
                .entering_subject(item)
                .map(|id| self.object_ref(id))
                .into_iter()
                .collect(),
            Subject::Targets => item
                .targets
                .iter()
                .filter_map(|bound| {
                    if self.same_target_object(*bound)
                        && let Target::Card(id) = bound.target
                    {
                        Some(self.object_ref(id))
                    } else {
                        None
                    }
                })
                .collect(),
        };
        if recipients.is_empty() {
            return;
        }
        let timestamp = self.next_role_timestamp();
        let state = self.state.roles.get_or_insert_with(Default::default);
        for recipient in recipients {
            state.base_stats.push(BaseStatsEffectView {
                recipient,
                source: item.object,
                power,
                toughness,
                timestamp,
                static_source: false,
            });
        }
    }
    pub(super) fn forget_base_stats(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.base_stats.retain(|effect| effect.recipient != object);
        }
    }
    pub(super) fn expire_base_stats(&mut self) {
        if let Some(state) = &mut self.state.roles {
            state.base_stats.retain(|effect| effect.static_source);
        }
    }
    pub(super) fn apply_base_stat_layers(
        &self,
        id: CardId,
        mut stats: Stats,
        departing: Option<CardId>,
    ) -> Stats {
        let object = self.object_ref(id);
        let cursed = self.cursed_role_timestamp(object);
        let mut power_time = cursed;
        let mut toughness_time = cursed;
        if cursed.is_some() {
            stats = Stats {
                power: 1,
                toughness: 1,
            };
        }
        if let Some(state) = &self.state.roles {
            for effect in &state.base_stats {
                if effect.recipient != object {
                    continue;
                }
                if effect.static_source {
                    let source = &self.state.objects[effect.source.card as usize];
                    if source.zone != Zone::Battlefield
                        || self.object_ref(effect.source.card) != effect.source
                    {
                        continue;
                    }
                    let Some(spec) = source.card.base_stat_setter() else {
                        continue;
                    };
                    if spec.own_turn && self.state.active != source.controller {
                        continue;
                    }
                    if spec.condition.is_some_and(|condition| {
                        self.stat_value_without_hand(
                            condition,
                            effect.source,
                            source.controller,
                            departing,
                        ) == 0
                    }) {
                        continue;
                    }
                }
                if let Some(power) = effect.power
                    && power_time.is_none_or(|t| effect.timestamp > t)
                {
                    stats.power = power;
                    power_time = Some(effect.timestamp);
                }
                if let Some(toughness) = effect.toughness
                    && toughness_time.is_none_or(|t| effect.timestamp > t)
                {
                    stats.toughness = toughness;
                    toughness_time = Some(effect.timestamp);
                }
            }
        }
        stats
    }
}
