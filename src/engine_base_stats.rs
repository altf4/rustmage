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
    pub permanent: bool,
}
/// A layer-7c modifier lasting until the specified player's next actual turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct UntilNextTurnStatsView {
    pub recipient: ObjectRef,
    pub controller: Player,
    pub power: i32,
    pub toughness: i32,
}
/// Sparse facts only for exact entering subjects read by resolved expressions.
/// Departure updates use the frozen pre-transition view, including simultaneous moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ResolvedStatSubjectView {
    pub object: ObjectRef,
    pub departure_power: Option<i128>,
}
impl Game {
    pub(super) fn target_stats_until_next_turn(
        &mut self,
        item: &SpellView,
        power: i32,
        toughness: i32,
    ) {
        let recipients = self
            .keyword_subjects(item, Subject::Targets)
            .into_iter()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_ref(id))
            .collect::<Vec<_>>();
        if recipients.is_empty() {
            return;
        }
        let roles = self.state.roles.get_or_insert_default();
        roles
            .until_next_turn_stats
            .extend(
                recipients
                    .into_iter()
                    .map(|recipient| UntilNextTurnStatsView {
                        recipient,
                        controller: item.controller,
                        power,
                        toughness,
                    }),
            );
    }
    pub(super) fn expire_next_turn_stats(&mut self, player: Player) {
        if let Some(roles) = &mut self.state.roles {
            roles
                .until_next_turn_stats
                .retain(|row| row.controller != player);
        }
    }
    pub(super) fn set_base_stats_from_source(&mut self, item: &SpellView, subject: Subject) {
        let Some(source) = item.source else { return; };
        let stats = if self.on_battlefield(source.object.card)
            && self.object_ref(source.object.card) == source.object
        {
            if self.is_creature(source.object.card) {
                self.stats(source.object.card)
            } else {
                Stats::default()
            }
        } else {
            Stats {
                power: if source.facts.is_creature() {
                    source.facts.power
                } else {
                    0
                },
                toughness: item
                    .source_quantity_lki
                    .as_ref()
                    .and_then(|facts| facts.toughness)
                    .expect("source base-setting program opts into toughness LKI"),
            }
        };
        // Both signed axes are sampled before creating the fixed layer-7b effect.
        // Ordinary amount helpers clamp negatives, which would be wrong here.
        self.set_base_stats(item, subject, Some(stats.power), Some(stats.toughness));
    }
    pub(super) fn bind_stat_subject(&mut self, object: ObjectRef, program: &[Instruction]) {
        let reads_power = program.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::SetResolvedBaseStats {
                    power: Some(ResolvedStatValue::EnteringPower),
                    ..
                } | Instruction::SetResolvedBaseStats {
                    toughness: Some(ResolvedStatValue::EnteringPower),
                    ..
                } | Instruction::GrowResolved {
                    value: ResolvedStatValue::EnteringPower,
                    ..
                }
            )
        });
        if !reads_power {
            return;
        }
        let facts = &mut self
            .state
            .roles
            .get_or_insert_with(Default::default)
            .stat_subjects;
        if !facts.iter().any(|fact| fact.object == object) {
            facts.push(ResolvedStatSubjectView {
                object,
                departure_power: None,
            });
        }
    }
    pub(super) fn capture_stat_subject_departure(&mut self, before: &ObjectView) {
        if let Some(state) = &mut self.state.roles {
            for fact in &mut state.stat_subjects {
                if fact.object == before.reference && fact.departure_power.is_none() {
                    fact.departure_power = Some(before.power);
                }
            }
        }
    }
    pub(super) fn resolved_stat_value(
        &self,
        item: &SpellView,
        value: ResolvedStatValue,
    ) -> Option<i128> {
        match value {
            ResolvedStatValue::Stat(value) => {
                Some(self.stat_value(value, item.source?.object, item.controller))
            }
            ResolvedStatValue::Threshold {
                value,
                minimum,
                yes,
                no,
            } => Some(
                if self.stat_value(value, item.source?.object, item.controller) >= minimum {
                    yes
                } else {
                    no
                },
            ),
            ResolvedStatValue::EnteringPower => {
                let subject = item.entering_object?;
                let fact = self
                    .state
                    .roles
                    .as_ref()?
                    .stat_subjects
                    .iter()
                    .find(|fact| fact.object == subject)?;
                if let Some(power) = fact.departure_power {
                    Some(power)
                } else if self.on_battlefield(subject.card)
                    && self.object_ref(subject.card) == subject
                {
                    Some(self.object_view(subject.card).power)
                } else {
                    None
                }
            }
        }
    }
    pub(super) fn set_resolved_base_stats(
        &mut self,
        item: &SpellView,
        subject: Subject,
        power: Option<ResolvedStatValue>,
        toughness: Option<ResolvedStatValue>,
    ) {
        let p = power.map(|value| self.resolved_stat_value(item, value));
        let t = toughness.map(|value| self.resolved_stat_value(item, value));
        if p.is_some_and(|value| value.is_none()) || t.is_some_and(|value| value.is_none()) {
            return;
        }
        self.set_base_stats(item, subject, p.flatten(), t.flatten());
    }
    pub(super) fn register_static_base_stats(&mut self, id: CardId) {
        let Some(spec) = self.state.objects[id as usize].card.base_stat_setter() else {
            return;
        };
        let object = self.object_ref(id);
        let timestamp = self.static_source_timestamp(object);
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
                permanent: false,
            });
    }
    pub(super) fn set_base_stats(
        &mut self,
        item: &SpellView,
        subject: Subject,
        power: Option<i128>,
        toughness: Option<i128>,
    ) {
        let Some(source) = item.source else { return; };
        let recipients: Vec<_> = match subject {
            Subject::Source => (self.on_battlefield(source.id)
                && self.object_ref(source.id) == source.object)
                .then_some(source.object)
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
                source: source.object,
                power,
                toughness,
                timestamp,
                static_source: false,
                permanent: false,
            });
        }
    }
    pub(super) fn forget_base_stats(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.base_stats.retain(|effect| effect.recipient != object);
            state
                .until_next_turn_stats
                .retain(|effect| effect.recipient != object);
        }
    }
    pub(super) fn expire_base_stats(&mut self) {
        if let Some(state) = &mut self.state.roles {
            state
                .base_stats
                .retain(|effect| effect.static_source || effect.permanent);
            // Legal cleanup cannot begin with an unresolved stack or announcement.
            state.stat_subjects.clear();
        }
    }
}
