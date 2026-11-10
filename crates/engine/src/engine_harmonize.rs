//! Shared Harmonize cost facts for the real and speculative payment worlds.
use super::*;
use ability_world::CharacteristicWorld;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CastMethod {
    pub(super) fn is_harmonize(self) -> bool {
        matches!(self, Self::Harmonize | Self::GrantedHarmonize)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn payment_x_symbols(&self) -> u32 {
        let card = self.item.source.expect("object-backed cost").card;
        match self.item.cast.map(|cast| cast.method) {
            Some(CastMethod::Harmonize) => u32::from(card.harmonize_x_symbols()),
            Some(CastMethod::Sneak) => 0,
            Some(CastMethod::Warp) => u32::from(self.warp_spec().x_symbols),
            Some(CastMethod::GrantedHarmonize) => {
                crate::cards::mana_symbols::printed_cost(card.info().mana_cost)
                    .map_or(0, |cost| cost.x_symbols)
            }
            _ => u32::from(card.x_symbols()),
        }
    }
}

/// One untapped controlled creature may be tapped; its power is sampled as the
/// total spell cost is determined, before mana abilities or payment alter it.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn contribution<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
    object: ObjectRef,
) -> Option<u64> {
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
    Some(u64::try_from(world.power(object.card).max(0)).unwrap_or(u64::MAX))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[cfg(feature = "test-support")]
pub(super) fn reduce(mut cost: ManaCost, amount: u32) -> ManaCost {
    cost.generic = cost.generic.saturating_sub(amount);
    cost
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn configure_harmonize_cast(&mut self, item: &mut SpellView, method: CastMethod) {
        let source = item.source.expect("harmonize source");
        item.cast.as_mut().unwrap().method = method;
        if method == CastMethod::Harmonize
            && source.card.harmonize_x_symbols() == 0
            && source.card.x_symbols() > 0
        {
            item.x = Some(0);
        }
        // Use the exact-object leave-stack exile replacement.
        self.state.objects[source.id as usize].flashback = true;
    }

    pub(super) fn harmonizer_discount(&self, item: &SpellView, object: ObjectRef) -> Option<u64> {
        let source = item.source.expect("harmonize has a source");
        if let Some(world) = self.cast_departure(source.object, item.controller) {
            contribution(&world, item.controller, object)
        } else {
            contribution(self, item.controller, object)
        }
    }

    pub(super) fn harmonize_choice_fundable(
        &self,
        a: &Announcement,
        object: Option<ObjectRef>,
    ) -> bool {
        let discount = match object {
            Some(reference) => match self.harmonizer_discount(&a.item, reference) {
                Some(power) => power,
                None => return false,
            },
            None => 0,
        };
        let base = Self::announcement_base_cost(a);
        let extra = Self::announcement_optional_reduction(a).saturating_add(discount);
        let source = a.item.source.expect("harmonize has a source");
        let x = a.item.x.unwrap_or(0);
        let total = if source.card.has_target_cost_modifier() {
            let plan = if a.plan == TargetPlan::NONE {
                self.target_spec(&a.item)
            } else {
                a.plan
            };
            self.target_proposal_total(a, base, plan, None, false, extra)
        } else {
            self.scalar_x_total_with_symbols(
                a.item.controller,
                source.id,
                base,
                x,
                source.card,
                a.payment_x_symbols(),
                &a.item.targets,
                extra,
            )
        };
        let Some(total) = total else {
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
