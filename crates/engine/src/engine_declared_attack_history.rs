//! Public declaration facts are frozen only for subtype guards that actually
//! exist. Live attackers and objects put onto the battlefield attacking never
//! substitute for a historical declaration.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeclaredAttackHistoryView {
    pub object: ObjectRef,
    pub player: Player,
    /// Only queried public subtypes that were true at the declaration event.
    pub subtypes: Vec<&'static str>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug)]
pub(super) struct DeclaredAttackHistory {
    queried: Vec<&'static str>,
    pub rows: Vec<DeclaredAttackHistoryView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn collect(condition: TriggerCondition, result: &mut Vec<&'static str>) {
    fn guard(guard: StepGuard, result: &mut Vec<&'static str>) {
        if let StepGuard::DeclaredAttackSubtypesAll(types) = guard {
            for &typ in types {
                if !result.contains(&typ) {
                    result.push(typ);
                }
            }
        }
    }
    match condition {
        TriggerCondition::AtStep { guard: value, .. } => guard(value, result),
        TriggerCondition::When {
            guard: value,
            event,
        }
        | TriggerCondition::InterveningIf {
            guard: value,
            event,
        } => {
            guard(value, result);
            collect(*event, result);
        }
        TriggerCondition::InZone { event, .. }
        | TriggerCondition::Granted(event)
        | TriggerCondition::OncePerTurn(event)
        | TriggerCondition::DuringOpponentsTurn(event)
        | TriggerCondition::CastFromHand(event)
        | TriggerCondition::CastFromExile(event)
        | TriggerCondition::TreasureManaSpent(event)
        | TriggerCondition::ManaSpent { event, .. } => collect(*event, result),
        _ => (),
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn record_declared_attack_history(
        &mut self,
        player: Player,
        declared: &[AttackDeclaration],
    ) {
        if declared.is_empty() {
            return;
        }
        let mut queried = self
            .state
            .declared_attack_history
            .as_ref()
            .map_or_else(Vec::new, |h| h.queried.clone());
        // Public definitions opt in even when currently suppressed: restoration
        // during the combat must not query newly changed live characteristics.
        for object in self
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield)
        {
            for ability in object.card.triggers() {
                collect(ability.condition, &mut queried);
            }
        }
        for grant in self
            .state
            .temporary_abilities
            .as_deref()
            .into_iter()
            .flatten()
        {
            if self.on_battlefield(grant.recipient.card)
                && self.object_ref(grant.recipient.card) == grant.recipient
                && (grant.permanent || grant.expires_turn == self.state.turn)
            {
                collect(
                    grant.granting_card.triggers()[usize::from(grant.ability)].condition,
                    &mut queried,
                );
            }
        }
        if queried.is_empty() {
            return;
        }
        let rows = declared
            .iter()
            .map(|attack| DeclaredAttackHistoryView {
                object: self.object_ref(attack.card),
                player,
                subtypes: queried
                    .iter()
                    .copied()
                    .filter(|typ| self.has_subtype(attack.card, typ))
                    .collect(),
            })
            .collect::<Vec<_>>();
        let history = self.state.declared_attack_history.get_or_insert_with(|| {
            Box::new(DeclaredAttackHistory {
                queried: Vec::new(),
                rows: Vec::new(),
            })
        });
        history.queried = queried;
        history.rows.extend(rows);
    }
    pub(super) fn declared_attack_subtypes_all(&self, player: Player, subtypes: &[&str]) -> bool {
        subtypes.iter().all(|wanted| {
            self.state
                .declared_attack_history
                .as_ref()
                .is_some_and(|h| {
                    h.rows.iter().any(|row| {
                        row.player == player && row.subtypes.iter().any(|actual| actual == wanted)
                    })
                })
        })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Option<Box<DeclaredAttackHistory>>) -> usize {
    rows.as_ref().map_or(0, |history| {
        std::mem::size_of::<DeclaredAttackHistory>()
            + history.queried.len() * std::mem::size_of::<&str>()
            + history
                .rows
                .iter()
                .map(|row| {
                    std::mem::size_of::<DeclaredAttackHistoryView>()
                        + row.subtypes.len() * std::mem::size_of::<&str>()
                })
                .sum::<usize>()
    })
}
