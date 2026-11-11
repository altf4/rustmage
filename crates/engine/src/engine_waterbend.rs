//! Pay part of an activated ability's generic mana cost by tapping permanents.
use super::*;
use ability_world::CharacteristicWorld;

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
    Waterbend(u16),
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
    pub(super) fn for_activation(ability: &ActivatedAbility, source: ObjectRef) -> Option<Self> {
        let kind = match ability.program.first() {
            Some(Instruction::WaterbendCost(amount)) if *amount != 0 => {
                TapPaymentKind::Waterbend(*amount)
            }
            Some(Instruction::TapCreaturesForActivationMana) => TapPaymentKind::Creatures,
            _ => return None,
        };
        Some(Self {
            kind,
            reserved_source: ability.cost.tap.then_some(source),
        })
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
            TapPaymentKind::Waterbend(amount) => u64::from(amount),
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

    pub(super) fn masks<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        player: Player,
        selected: &[ObjectRef],
    ) -> [u64; 32] {
        let mut masks = [0; 32];
        for id in 0..world.game().state.objects.len() {
            let object = world.reference(id as CardId);
            if !selected.contains(&object) && self.eligible(world, player, object) {
                masks[0] += 1;
            }
        }
        masks[0] = masks[0].min(self.cap().saturating_sub(selected.len() as u64));
        masks
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn item_activation_tap_payment(&self, item: &SpellView) -> Option<TapPayment> {
        let StackKind::ActivatedAbility { ability } = item.kind else {
            return None;
        };
        let source = item.source?;
        TapPayment::for_activation(source.card.activation(ability), source.object)
    }

    pub fn waterbend_choice(&self) -> Option<WaterbendChoiceView> {
        if !matches!(self.state.pending, Pending::Waterbend { .. }) {
            return None;
        }
        self.activation_tap_payment_view()
    }

    pub fn creature_tap_payment(&self) -> Option<ActivationTapPaymentView> {
        if !matches!(self.state.pending, Pending::CreatureTapPayment { .. }) {
            return None;
        }
        self.activation_tap_payment_view()
    }

    fn activation_tap_payment_view(&self) -> Option<ActivationTapPaymentView> {
        let announcement = self.state.announcing.as_ref()?;
        let selection = announcement.convoke.as_ref()?;
        Some(WaterbendChoiceView {
            source: announcement.item.source?.object,
            selected: selection.improvised.clone(),
            remaining: selection.remaining,
            can_pay: self.mana_payment_domain().is_some(),
        })
    }

    pub(super) fn begin_activation_tap_payment(&mut self) {
        let announcement = self
            .state
            .announcing
            .as_ref()
            .expect("activation announcement");
        let payment = self
            .item_activation_tap_payment(&announcement.item)
            .expect("activation permits tap contributions");
        let goal = self.funding_goal(announcement);
        let announcement = self.state.announcing.as_mut().unwrap();
        assert!(announcement.convoke.is_none());
        announcement.convoke = Some(Box::new(convoke::ConvokeSelection {
            remaining: ManaCost {
                colored: goal.required,
                generic: goal.generic,
            },
            selected: Vec::new(),
            improvised: Vec::new(),
            delved: Vec::new(),
        }));
        let player = announcement.item.controller;
        self.state.pending = match payment.kind {
            TapPaymentKind::Waterbend(_) => Pending::Waterbend { player },
            TapPaymentKind::Creatures => Pending::CreatureTapPayment { player },
        };
    }
}
