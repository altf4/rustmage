//! Pay generic mana with controlled, untapped permanents.
use super::*;
use crate::cards::WaterbendAdditional;
use ability_world::CharacteristicWorld;

pub(super) fn generic_cost(generic: u32) -> ManaCost {
    ManaCost {
        generic,
        ..ManaCost::ZERO
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WaterbendChoiceView {
    pub source: ObjectRef,
    pub selected: Vec<ObjectRef>,
    pub remaining: ManaCost,
    pub can_pay: bool,
}

pub type ActivationTapPaymentView = WaterbendChoiceView;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TapPaymentKind {
    Waterbend(u64),
    Creatures,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TapPayment {
    pub kind: TapPaymentKind,
    reserved_source: Option<ObjectRef>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl TapPayment {
    pub(super) fn for_activation(
        ability: &ActivatedAbility,
        source: ObjectRef,
        x: Option<u32>,
    ) -> Option<Self> {
        let kind = match ability.program.first() {
            Some(Instruction::WaterbendCost(amount)) => {
                TapPaymentKind::Waterbend(u64::from(*amount))
            }
            Some(Instruction::WaterbendXCost { minimum }) => {
                TapPaymentKind::Waterbend(u64::from(x.unwrap_or(*minimum)))
            }
            Some(Instruction::TapCreaturesForActivationMana) => TapPaymentKind::Creatures,
            _ => return None,
        };
        Some(Self {
            kind,
            reserved_source: ability.cost.tap.then_some(source),
        })
    }

    pub(super) fn waterbend(amount: u64) -> Self {
        Self {
            kind: TapPaymentKind::Waterbend(amount),
            reserved_source: None,
        }
    }

    fn kinds(self) -> u16 {
        match self.kind {
            TapPaymentKind::Waterbend(_) => 1 | 16,
            TapPaymentKind::Creatures => 1,
        }
    }

    pub(super) fn cap(self) -> u64 {
        match self.kind {
            // Taxes may increase the locked cost, but not its waterbend component.
            TapPaymentKind::Waterbend(amount) => amount,
            TapPaymentKind::Creatures => u64::MAX,
        }
    }

    /// Only the reserved source's tap symbol needs haste. These contributions
    /// may come from creatures which have just entered under the payer's control.
    pub(super) fn eligible<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        player: Player,
        object: ObjectRef,
    ) -> bool {
        if self.reserved_source == Some(object)
            || world
                .game()
                .state
                .objects
                .get(object.card as usize)
                .is_none()
        {
            return false;
        }
        let permanent = world.object(object.card);
        world.reference(object.card) == object
            && permanent.zone == Zone::Battlefield
            && permanent.controller == player
            && !permanent.tapped
            && world.kinds(object.card) & self.kinds() != 0
    }

    pub(super) fn add_exclusive_masks<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        player: Player,
        masks: &mut [u64; 32],
        ordinary: (bool, bool),
        excluded: &[ObjectRef],
        already_used: usize,
    ) {
        let cap = self.cap().saturating_sub(already_used as u64);
        if cap == 0 {
            return;
        }
        let (convoke, improvise) = ordinary;
        let count = (0..world.game().state.objects.len())
            .map(|id| world.reference(id as CardId))
            .filter(|object| {
                if excluded.contains(object) || !self.eligible(world, player, *object) {
                    return false;
                }
                // The same object can already contribute one generic mana
                // through these unlimited methods. Count it only there.
                let kinds = world.kinds(object.card);
                !(convoke && kinds & 1 != 0 || improvise && kinds & 16 != 0)
            })
            .count() as u64;
        masks[0] += count.min(cap);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn spell_waterbend_additional(&self) -> Option<WaterbendAdditional> {
        self.item.cast?;
        self.item.source?.card.waterbend_additional()
    }

    pub(super) fn fixed_waterbend_addition(&self) -> u32 {
        match self.spell_waterbend_additional() {
            Some(WaterbendAdditional::Required(amount)) => amount,
            Some(WaterbendAdditional::Optional(amount))
                if self
                    .item
                    .cast
                    .is_some_and(|cast| cast.waterbend == Some(true)) =>
            {
                amount
            }
            _ => 0,
        }
    }

    pub(super) fn waterbend_alternative(&self) -> Option<ManaCost> {
        (self.item.cast?.method == CastMethod::LinkedExileWaterbend)
            .then(|| self.origin_alternative_cost.expect("linked waterbend base"))
    }

    pub(super) fn needs_waterbend_cost(&self) -> bool {
        self.pay_window
            && self.item.cast.is_some_and(|cast| cast.waterbend.is_none())
            && matches!(
                self.spell_waterbend_additional(),
                Some(WaterbendAdditional::Optional(_))
            )
    }

    pub(super) fn tap_payment(&self) -> Option<TapPayment> {
        if self.activation_cost_route != ActivationCostRoute::Printed {
            return None;
        }
        let source = self.item.source?;
        match self.item.kind {
            StackKind::ActivatedAbility { ability } => TapPayment::for_activation(
                source.card.activation(ability),
                source.object,
                self.item.x,
            ),
            StackKind::Spell => {
                let addition = match self.spell_waterbend_additional() {
                    Some(WaterbendAdditional::Required(amount)) => Some(amount),
                    Some(WaterbendAdditional::RequiredX) => self.item.x,
                    Some(WaterbendAdditional::Optional(amount))
                        if self.item.cast?.waterbend == Some(true) =>
                    {
                        Some(amount)
                    }
                    _ => None,
                };
                let alternate = self.waterbend_alternative();
                (addition.is_some() || alternate.is_some()).then(|| {
                    TapPayment::waterbend(
                        u64::from(addition.unwrap_or(0))
                            + u64::from(alternate.map_or(0, |cost| cost.generic)),
                    )
                })
            }
            _ => None,
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub fn waterbend_choice(&self) -> Option<WaterbendChoiceView> {
        if matches!(self.state.pending, Pending::ResolutionMana { .. }) {
            let resolution = self.state.resolving.as_ref()?;
            let request = resolution.mana_payment.as_ref()?;
            let selection = request.waterbend.as_ref()?.selected.as_ref()?;
            let remaining = request.remaining_mana();
            return Some(WaterbendChoiceView {
                source: resolution.item.source?.object,
                selected: selection.clone(),
                remaining,
                can_pay: convoke::payable(
                    self.available_mana(request.payer, mana_lots::Purpose::Other)
                        .map(u128::from),
                    remaining.colored,
                    remaining.generic,
                    &[0; 32],
                ),
            });
        }
        let announcement = self.state.announcing.as_ref()?;
        matches!(
            announcement.tap_payment()?.kind,
            TapPaymentKind::Waterbend(_)
        )
        .then(|| self.tap_payment_view())
        .flatten()
    }

    pub fn creature_tap_payment(&self) -> Option<ActivationTapPaymentView> {
        if !matches!(self.state.pending, Pending::CreatureTapPayment { .. }) {
            return None;
        }
        self.tap_payment_view()
    }

    fn tap_payment_view(&self) -> Option<ActivationTapPaymentView> {
        let announcement = self.state.announcing.as_ref()?;
        let selection = announcement.convoke.as_ref()?;
        Some(WaterbendChoiceView {
            source: announcement.item.source?.object,
            selected: selection.substituted.clone(),
            remaining: selection.remaining,
            can_pay: self.mana_payment_domain().is_some(),
        })
    }

    pub(super) fn begin_tap_payment(&mut self) {
        if matches!(self.state.pending, Pending::ResolutionMana { .. }) {
            let waterbend = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .mana_payment
                .as_mut()
                .unwrap()
                .waterbend
                .as_mut()
                .unwrap();
            assert!(waterbend.selected.is_none());
            waterbend.selected = Some(Vec::new());
            return;
        }
        let announcement = self
            .state
            .announcing
            .as_ref()
            .expect("payment announcement");
        let payment = announcement.tap_payment().expect("tap contributions");
        let goal = self.funding_goal(announcement);
        let announcement = self.state.announcing.as_mut().unwrap();
        assert!(announcement.convoke.is_none());
        announcement.convoke = Some(Box::new(convoke::ConvokeSelection::new(ManaCost {
            colored: goal.required,
            generic: goal.generic,
        })));
        let player = announcement.item.controller;
        self.state.pending = if let Some(costs) = &mut announcement.cast_costs {
            costs.mana_closed = true;
            Pending::CastCosts { player }
        } else {
            match payment.kind {
                TapPaymentKind::Waterbend(_) => Pending::Waterbend { player },
                TapPaymentKind::Creatures => Pending::CreatureTapPayment { player },
            }
        };
    }

    pub(super) fn choose_tap_payment_contributor(&mut self, object: ObjectRef) {
        if matches!(self.state.pending, Pending::ResolutionMana { .. }) {
            let waterbend = self
                .state
                .resolving
                .as_mut()
                .unwrap()
                .mana_payment
                .as_mut()
                .unwrap()
                .waterbend
                .as_mut()
                .unwrap();
            waterbend.selected.as_mut().unwrap().push(object);
            return;
        }
        let frame = self
            .state
            .announcing
            .as_mut()
            .unwrap()
            .convoke
            .as_mut()
            .unwrap();
        frame.remaining.generic = frame
            .remaining
            .generic
            .checked_sub(1)
            .expect("generic share");
        frame.substituted.push(object);
    }
}
