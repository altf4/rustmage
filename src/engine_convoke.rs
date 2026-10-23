//! Intrinsic convoke: exact matching, a closed mana window, and atomic tap costs.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvokeContribution {
    Generic,
    White,
    Blue,
    Black,
    Red,
    Green,
}
impl ConvokeContribution {
    fn color(self) -> Option<usize> {
        match self {
            Self::Generic => None,
            Self::White => Some(0),
            Self::Blue => Some(1),
            Self::Black => Some(2),
            Self::Red => Some(3),
            Self::Green => Some(4),
        }
    }
    fn apply(self, colors: u8, cost: &mut ManaCost) -> bool {
        if let Some(i) = self.color() {
            if colors & (1 << i) == 0 || cost.colored[i] == 0 {
                return false;
            }
            cost.colored[i] -= 1;
        } else {
            if cost.generic == 0 {
                return false;
            }
            cost.generic -= 1;
        }
        true
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConvokePaymentView {
    pub object: ObjectRef,
    pub contribution: ConvokeContribution,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConvokeSelection {
    pub selected: Vec<ConvokePaymentView>,
    pub remaining: ManaCost,
}
impl ConvokeSelection {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.selected.len() * std::mem::size_of::<ConvokePaymentView>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConvokeChoiceView {
    pub spell: ObjectRef,
    pub selected: Vec<ConvokePaymentView>,
    pub remaining: ManaCost,
    pub can_pay: bool,
}

/// Hall's theorem on the five-color bipartite matching problem. Only 32 masks
/// exist, so neither creature subsets nor numeric mana units are enumerated.
/// Real mana covers matching requirements first; exchanges preserve feasibility.
/// Colorless creatures pay generic only; required {C} always consumes real mana.
pub(super) fn payable(pool: [u128; 6], required: Mana, generic: u32, masks: &[u64; 32]) -> bool {
    if pool[5] < u128::from(required[5]) {
        return false;
    }
    let deficits: [u64; 5] = std::array::from_fn(|i| {
        u64::try_from(u128::from(required[i]).saturating_sub(pool[i])).unwrap()
    });
    let mut subset = *masks;
    for bit in 0..5 {
        for mask in 0..32 {
            if mask & (1 << bit) != 0 {
                subset[mask] += subset[mask ^ (1 << bit)];
            }
        }
    }
    let total = subset[31];
    let mut needs = [0u64; 32];
    for mask in 1usize..32 {
        let i = mask.trailing_zeros() as usize;
        needs[mask] = needs[mask ^ (1 << i)] + deficits[i];
        if needs[mask] > total - subset[31 ^ mask] {
            return false;
        }
    }
    let mut left = u128::from(generic).saturating_sub(u128::from(total - needs[31]));
    for i in 0..6 {
        let spare = pool[i].saturating_sub(u128::from(required[i]));
        if spare >= left {
            return true;
        }
        left -= spare;
    }
    left == 0
}
/// Mana-type substitution applies to mana, never to a creature's convoke color.
/// Treat actual mana as universal matching resources; Hall deficits still enforce
/// colored creature contributions and {C} still cannot be paid by convoking.
pub(super) fn payable_in<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    purpose: mana_lots::Purpose,
    pool: [u128; 6],
    required: Mana,
    generic: u32,
    masks: &[u64; 32],
) -> bool {
    if !cast_origins::spends_any_type(world, player, purpose) {
        return payable(pool, required, generic, masks);
    }
    let mana = pool.into_iter().fold(0u128, u128::saturating_add);
    let mut subset = *masks;
    for bit in 0..5 {
        for mask in 0..32 {
            if mask & (1 << bit) != 0 {
                subset[mask] += subset[mask ^ (1 << bit)];
            }
        }
    }
    let creatures = u128::from(subset[31]);
    let total = required.into_iter().map(u128::from).sum::<u128>() + u128::from(generic);
    if mana.saturating_add(creatures) < total || mana < u128::from(required[5]) {
        return false;
    }
    let mut needs = [0u64; 32];
    for mask in 1usize..32 {
        let i = mask.trailing_zeros() as usize;
        needs[mask] = needs[mask ^ (1 << i)] + u64::from(required[i]);
        let unmatched =
            u128::from(needs[mask]).saturating_sub(u128::from(subset[31] - subset[31 ^ mask]));
        if unmatched + u128::from(required[5]) > mana {
            return false;
        }
    }
    true
}

impl Game {
    pub(super) fn convoke_eligible(&self, p: Player, r: ObjectRef) -> bool {
        self.state.objects.get(r.card as usize).is_some_and(|o| {
            o.generation == r.generation
                && o.controller == p
                && o.zone == Zone::Battlefield
                && self.object_kinds(r.card, o) & 1 != 0
                && !o.tapped
        })
    }
    pub(super) fn convoke_masks(&self, p: Player, selected: &[ConvokePaymentView]) -> [u64; 32] {
        let mut masks = [0; 32];
        for (id, _) in self.state.objects.iter().enumerate() {
            let r = self.object_ref(id as CardId);
            if self.convoke_eligible(p, r) && !selected.iter().any(|s| s.object == r) {
                masks[usize::from(
                    ability_world::CharacteristicWorld::colors(self, id as CardId) & 31,
                )] += 1;
            }
        }
        masks
    }
    pub fn convoke_choice(&self) -> Option<ConvokeChoiceView> {
        if !matches!(self.state.pending, Pending::Convoke { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let f = a.convoke.as_ref()?;
        let pool = self.available_mana(a.item.controller, self.purpose_for_item(&a.item));
        let can_pay = payable_in(
            self,
            a.item.controller,
            self.purpose_for_item(&a.item),
            pool.map(u128::from),
            f.remaining.colored,
            f.remaining.generic,
            &[0; 32],
        );
        Some(ConvokeChoiceView {
            spell: a.item.object,
            selected: f.selected.clone(),
            remaining: f.remaining,
            can_pay,
        })
    }
    pub(super) fn begin_convoke(&mut self) {
        let a = self.state.announcing.as_mut().unwrap();
        assert!(
            a.item.kind == StackKind::Spell
                && a.item.source_card.has_convoke()
                && a.convoke.is_none()
        );
        a.convoke = Some(Box::new(ConvokeSelection {
            remaining: a.total_cost.expect("locked total"),
            selected: Vec::new(),
        }));
        self.state.pending = Pending::Convoke {
            player: a.item.controller,
        };
    }
    pub(super) fn choose_convoker(&mut self, object: ObjectRef, contribution: ConvokeContribution) {
        let colors = ability_world::CharacteristicWorld::colors(self, object.card);
        let f = self
            .state
            .announcing
            .as_mut()
            .unwrap()
            .convoke
            .as_mut()
            .unwrap();
        assert!(contribution.apply(colors, &mut f.remaining));
        f.selected.push(ConvokePaymentView {
            object,
            contribution,
        });
    }
    pub(super) fn visit_convoke_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let f = a.convoke.as_ref().unwrap();
        let p = a.item.controller;
        // Exact selected incarnations remain reserved until this atomic cost ends.
        if !f
            .selected
            .iter()
            .all(|s| self.convoke_eligible(p, s.object))
        {
            return false;
        }
        let pool = self.available_mana(p, self.purpose_for_item(&a.item));
        let mut done = false;
        if let Some(domain) = self.mana_payment_domain() {
            domain.representatives(|payment| {
                if !done {
                    done = emit(Action::PayCosts { payment });
                }
            });
        }
        if done {
            return true;
        }
        let masks = self.convoke_masks(p, &f.selected);
        for (id, _) in self.state.objects.iter().enumerate() {
            let object = self.object_ref(id as CardId);
            if !self.convoke_eligible(p, object) || f.selected.iter().any(|s| s.object == object) {
                continue;
            }
            let colors = ability_world::CharacteristicWorld::colors(self, id as CardId);
            let mut rest = masks;
            rest[usize::from(colors & 31)] -= 1;
            for contribution in [
                ConvokeContribution::Generic,
                ConvokeContribution::White,
                ConvokeContribution::Blue,
                ConvokeContribution::Black,
                ConvokeContribution::Red,
                ConvokeContribution::Green,
            ] {
                let mut cost = f.remaining;
                if contribution.apply(colors, &mut cost)
                    && payable_in(
                        self,
                        p,
                        self.purpose_for_item(&a.item),
                        pool.map(u128::from),
                        cost.colored,
                        cost.generic,
                        &rest,
                    )
                    && emit(Action::ChooseConvoker {
                        object,
                        contribution,
                    })
                {
                    return true;
                }
            }
        }
        false
    }
}
