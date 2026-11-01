//! Shared Harmonize cost facts for the real and speculative payment worlds.
use super::*;
use ability_world::CharacteristicWorld;

/// One untapped controlled creature may be tapped; its power is sampled as the
/// total spell cost is determined, before mana abilities or payment alter it.
pub(super) fn contribution<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    object: ObjectRef,
) -> Option<u32> {
    world.game().state.objects.get(object.card as usize)?;
    let permanent = world.object(object.card);
    if world.reference(object.card) != object
        || permanent.zone != Zone::Battlefield
        || permanent.controller != player
        || permanent.tapped
        || world.kinds(object.card) & 1 == 0
    {
        return None;
    }
    Some(u32::try_from(world.power(object.card).max(0)).unwrap_or(u32::MAX))
}

impl Game {
    pub(super) fn harmonize_choice_fundable(
        &self,
        a: &Announcement,
        object: Option<ObjectRef>,
    ) -> bool {
        let discount = match object {
            Some(reference) => match contribution(self, a.item.controller, reference) {
                Some(power) => power,
                None => return false,
            },
            None => 0,
        };
        let base = Self::announcement_base_cost(a);
        let extra = Self::announcement_optional_reduction(a) + u64::from(discount);
        let Some(total) = self.target_spell_cost_with_extra(
            a.item.controller,
            a.item
                .source
                .expect("cost announcement has object source")
                .id,
            base,
            &a.item.targets,
            extra,
        ) else {
            return false;
        };
        self.harmonize_proposal_fundable(a, object, total)
    }

    pub(super) fn visit_harmonize_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        if self.harmonize_choice_fundable(a, None)
            && emit(Action::ChooseHarmonizer { object: None })
        {
            return true;
        }
        for id in 0..self.state.objects.len() {
            let reference = self.object_ref(id as CardId);
            if self.harmonize_choice_fundable(a, Some(reference))
                && emit(Action::ChooseHarmonizer {
                    object: Some(reference),
                })
            {
                return true;
            }
        }
        false
    }

    pub(super) fn choose_harmonizer(&mut self, object: Option<ObjectRef>) {
        let definition = self
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .source
            .expect("cast announcement has source")
            .card;
        let can_improvise = improvise::potential(self, definition);
        let a = self.state.announcing.as_mut().unwrap();
        a.harmonizer = Some(object);
        if let Some(reference) = object {
            a.add_cast_cost_clause(CastCostClause::Harmonize(reference));
            a.initialize_cast_costs(can_improvise);
        }
        self.continue_announcement_choices(false);
    }
}
