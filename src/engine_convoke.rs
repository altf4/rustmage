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
    pub(super) fn apply(self, colors: u8, cost: &mut ManaCost) -> bool {
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
pub(super) fn contributions() -> [ConvokeContribution; 6] {
    [
        ConvokeContribution::Generic,
        ConvokeContribution::White,
        ConvokeContribution::Blue,
        ConvokeContribution::Black,
        ConvokeContribution::Red,
        ConvokeContribution::Green,
    ]
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConvokePaymentView {
    pub object: ObjectRef,
    pub contribution: ConvokeContribution,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConvokeSelection {
    pub selected: Vec<ConvokePaymentView>,
    pub improvised: Vec<ObjectRef>,
    pub remaining: ManaCost,
}
impl ConvokeSelection {
    pub(super) fn objects(&self) -> Vec<ObjectRef> {
        self.selected
            .iter()
            .map(|s| s.object)
            .chain(self.improvised.iter().copied())
            .collect()
    }
    pub(super) fn contains(&self, r: ObjectRef) -> bool {
        self.selected.iter().any(|s| s.object == r) || self.improvised.contains(&r)
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.selected.len() * std::mem::size_of::<ConvokePaymentView>()
            + self.improvised.len() * std::mem::size_of::<ObjectRef>()
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
    pub fn convoke_choice(&self) -> Option<ConvokeChoiceView> {
        if !matches!(
            self.state.pending,
            Pending::Convoke { .. } | Pending::Improvise { .. }
        ) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        let f = a.convoke.as_ref()?;
        if !a.item.source_card.has_convoke() {
            return None;
        }
        let pool = self.available_mana(a.item.controller, self.purpose_for_item(&a.item));
        let can_pay = self.mana_payment_domain().is_some()
            && payable_in(
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
        self.begin_contributors(false);
    }
    pub(super) fn begin_improvise(&mut self) {
        self.begin_contributors(true);
    }
    fn begin_contributors(&mut self, artifact: bool) {
        let a = self.state.announcing.as_mut().unwrap();
        assert!(a.item.kind == StackKind::Spell && a.convoke.is_none());
        a.initialize_cast_costs(artifact);
        a.convoke = Some(Box::new(ConvokeSelection {
            remaining: a.total_cost.expect("locked total"),
            selected: Vec::new(),
            improvised: Vec::new(),
        }));
        self.state.pending = if let Some(c) = &mut a.cast_costs {
            c.mana_closed = true;
            Pending::CastCosts {
                player: a.item.controller,
            }
        } else if artifact {
            Pending::Improvise {
                player: a.item.controller,
            }
        } else {
            Pending::Convoke {
                player: a.item.controller,
            }
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
        // Selecting contributors reserves the whole mana-payment group. Commit
        // all their taps together when that cost is paid, before any later cost.
    }
    pub(super) fn visit_convoke_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let f = a.convoke.as_ref().unwrap();
        if !self.tap_contributors_valid(a) {
            return false;
        }
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
        for id in 0..self.state.objects.len() {
            let object = self.object_ref(id as CardId);
            if f.contains(object) {
                continue;
            }
            if a.item.source_card.has_convoke() && self.convoke_eligible(a.item.controller, object)
            {
                for contribution in contributions() {
                    if self.contributor_fundable(object, Some(contribution))
                        && emit(Action::ChooseConvoker {
                            object,
                            contribution,
                        })
                    {
                        return true;
                    }
                }
            }
            if self.item_improvise(&a.item)
                && improvise::eligible(self, a.item.controller, object)
                && self.contributor_fundable(object, None)
                && emit(Action::ChooseImproviser { object })
            {
                return true;
            }
        }
        false
    }
    fn contributor_fundable(
        &self,
        object: ObjectRef,
        contribution: Option<ConvokeContribution>,
    ) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let f = a.convoke.as_ref().unwrap();
        if a.cast_costs.is_some() {
            let mut choice = self.cast_cost_world(a);
            return (if let Some(c) = contribution {
                choice.project_convoker(object, c)
            } else {
                choice.project_improviser(object)
            }) && choice.cost_completion();
        }
        let mut cost = f.remaining;
        let colors = ability_world::CharacteristicWorld::colors(self, object.card);
        if !contribution
            .unwrap_or(ConvokeContribution::Generic)
            .apply(colors, &mut cost)
        {
            return false;
        }
        let mut selected = f.objects();
        selected.push(object);
        let masks = improvise::masks(
            self,
            a.item.controller,
            a.item.source_card.has_convoke(),
            self.item_improvise(&a.item),
            &selected,
        );
        convoke::payable_in(
            self,
            a.item.controller,
            self.purpose_for_item(&a.item),
            self.available_mana(a.item.controller, self.purpose_for_item(&a.item))
                .map(u128::from),
            cost.colored,
            cost.generic,
            &masks,
        )
    }
}
