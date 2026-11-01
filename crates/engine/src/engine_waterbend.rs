//! Generic-only contributions to a printed waterbend cost (CR 701.67).
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

/// Unlike a tap-symbol cost, this does not require a creature to have haste.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn eligible<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    object: ObjectRef,
) -> bool {
    if world
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
        && world.kinds(object.card) & (1 | 16) != 0
}

/// A dual-type object counts once, and taxes do not raise the printed cap.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn masks<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    cap: u16,
    excluded: &[ObjectRef],
) -> [u64; 32] {
    let mut masks = [0; 32];
    for id in 0..world.game().state.objects.len() {
        let object = world.reference(id as CardId);
        if !excluded.contains(&object) && eligible(world, player, object) {
            masks[0] += 1;
        }
    }
    masks[0] = masks[0].min(u64::from(cap));
    masks
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn item_waterbend(&self, item: &SpellView) -> u16 {
        if !matches!(item.kind, StackKind::ActivatedAbility { .. }) {
            return 0;
        }
        match item.program().and_then(|(_, program)| program.first()) {
            Some(Instruction::WaterbendCost(amount)) => *amount,
            _ => 0,
        }
    }

    pub fn waterbend_choice(&self) -> Option<WaterbendChoiceView> {
        if !matches!(self.state.pending, Pending::Waterbend { .. }) {
            return None;
        }
        let announcement = self.state.announcing.as_ref()?;
        let selection = announcement.convoke.as_ref()?;
        Some(WaterbendChoiceView {
            source: announcement.item.source?.object,
            selected: selection.improvised.clone(),
            remaining: selection.remaining,
            can_pay: self.mana_payment_domain().is_some(),
        })
    }

    pub(super) fn begin_waterbend(&mut self) {
        let goal = self.funding_goal(
            self.state
                .announcing
                .as_ref()
                .expect("waterbend announcement"),
        );
        let announcement = self
            .state
            .announcing
            .as_mut()
            .expect("waterbend announcement");
        assert!(announcement.convoke.is_none());
        announcement.convoke = Some(Box::new(convoke::ConvokeSelection {
            remaining: ManaCost {
                colored: goal.required,
                generic: goal.generic,
            },
            selected: Vec::new(),
            improvised: Vec::new(),
        }));
        self.state.pending = Pending::Waterbend {
            player: announcement.item.controller,
        };
    }
}
