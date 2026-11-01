//! Declared target shares stay attached to their exact occurrence through resolution.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TargetAllocation {
    pub target: BoundTarget,
    pub amount: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TargetAmountChoiceView {
    pub target: BoundTarget,
    pub remaining: u32,
    pub minimum: u32,
    pub maximum: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn division_total(&self) -> Option<u32> {
        self.program()?
            .1
            .iter()
            .find_map(|instruction| match instruction {
                Instruction::DamageDivided { total }
                | Instruction::DistributeCounters { total, .. } => Some(u32::from(*total)),
                Instruction::DistributeCountersX { .. } => self.x,
                _ => None,
            })
    }
    pub(super) fn allocate_single_target(&mut self) {
        if self.targets.len() == 1
            && let Some(total) = self.division_total()
        {
            self.target_allocations = Some(Box::new(vec![TargetAllocation {
                target: self.targets[0],
                amount: total,
            }]));
        }
    }
    pub(super) fn retarget_allocation(&mut self, index: usize, new: BoundTarget) {
        if let Some(rows) = &mut self.target_allocations {
            rows[index].target = new;
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn target_amount_choice(&self) -> Option<TargetAmountChoiceView> {
        if !matches!(self.state.pending, Pending::TargetAmount { .. }) {
            return None;
        }
        let item = &self.state.announcing.as_ref()?.item;
        let total = item.division_total()?;
        let selected = item
            .target_allocations
            .as_deref()
            .map_or(&[][..], |v| v.as_slice());
        let used: u32 = selected.iter().map(|row| row.amount).sum();
        let target = *item.targets.get(selected.len())?;
        let others =
            u32::try_from(item.targets.len() - selected.len() - 1).expect("bounded targets");
        let remaining = total.checked_sub(used).expect("bounded allocation");
        Some(TargetAmountChoiceView {
            target,
            remaining,
            minimum: if others == 0 { remaining } else { 1 },
            maximum: remaining - others,
        })
    }
    pub(super) fn start_target_amounts(&mut self) -> bool {
        let item = &self.state.announcing.as_ref().unwrap().item;
        if item.division_total().is_some()
            && item.target_allocations.as_deref().map_or(0, |v| v.len()) < item.targets.len()
        {
            self.state.pending = Pending::TargetAmount {
                player: item.controller,
            };
            true
        } else {
            false
        }
    }
    pub(super) fn choose_target_amount(&mut self, amount: u32) {
        let choice = self.target_amount_choice().expect("announced division");
        let a = self.state.announcing.as_mut().unwrap();
        a.item
            .target_allocations
            .get_or_insert_with(Default::default)
            .push(TargetAllocation {
                target: choice.target,
                amount,
            });
        self.state.pending = Pending::Targets {
            player: a.item.controller,
        };
        self.finish_announcement();
    }
    pub(super) fn damage_divided(&mut self, item: &SpellView) {
        let source = self.program_damage_source(item);
        let events: Vec<_> = item
            .target_allocations
            .as_ref()
            .into_iter()
            .flat_map(|rows| rows.iter())
            .filter(|row| item.targets.contains(&row.target) && self.same_target_object(row.target))
            .map(|row| DamageEvent {
                source,
                target: row.target.target,
                amount: i128::from(row.amount),
                combat: false,
                preventable: true,
            })
            .collect();
        self.apply_damage_batch(&events);
    }
}
