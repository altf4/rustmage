//! Discrete X declarations for ordinary nonmana costs and their typed targets.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn affine_x(value: AmountValue) -> Option<(u128, u128)> {
    match value {
        AmountValue::ChosenX => Some((1, 0)),
        AmountValue::Fixed(n) => Some((0, u128::from(n))),
        AmountValue::Sum(values) => values.iter().try_fold((0u128, 0u128), |(a, b), value| {
            let (x, constant) = affine_x(*value)?;
            Some((a.checked_add(x)?, b.checked_add(constant)?))
        }),
        _ => None,
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn nonmana_x(&self) -> bool {
        matches!(self.item.kind, StackKind::ActivatedAbility { ability }
            if self.item.source.expect("activation source").card.activation(ability).nonmana_x())
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn nonmana_x_intervals(&self, a: &Announcement) -> Vec<XInterval> {
        let Some((TargetPlan::Single(spec), _)) = a.item.program() else {
            return Vec::new();
        };
        let TargetRule::ManaValueExactly { base, amount, .. } = spec.rule else {
            return Vec::new();
        };
        let Some((factor, offset)) = affine_x(amount).filter(|(factor, _)| *factor > 0) else {
            return Vec::new();
        };
        // Targets are chosen before costs. Their public values bound all possible
        // declarations, including costs enabled later by a mana child.
        let mut values = Vec::new();
        self.visit_targets(*base, a.item.controller, a.item.target_source(), |target| {
            if let Target::Card(card) = target {
                let value = u128::from(self.object_mana_value(card));
                if let Some(delta) = value.checked_sub(offset)
                    && delta % factor == 0
                    && let Ok(x) = u32::try_from(delta / factor)
                {
                    values.push(x);
                }
            }
            false
        });
        values.sort_unstable();
        values.dedup();
        let mut result: Vec<XInterval> = Vec::new();
        for value in values {
            let mut proposal = a.clone();
            proposal.item.x = Some(value);
            proposal.plan = self.target_spec(&proposal.item);
            if !proposal
                .cost_options()
                .any(|cost| self.announcement_cost_fundable(&proposal, cost))
            {
                continue;
            }
            if let Some(last) = result.last_mut()
                && last.maximum.checked_add(1) == Some(value)
            {
                last.maximum = value;
            } else {
                result.push(XInterval {
                    minimum: value,
                    maximum: value,
                });
            }
        }
        result
    }
}
