//! A permission route freezes its own nonmana costs; ordinary casts of the
//! same physical card never inherit them just because their origin is a graveyard.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    pub(super) fn self_graveyard_cast_cost(self) -> Option<(u32, AdditionalCost)> {
        self.static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .find_map(|fragment| {
                if let StaticAbilityFragment::SelfGraveyardCastWithAdditionalCosts {
                    life,
                    additional,
                } = *fragment
                {
                    Some((life, additional))
                } else {
                    None
                }
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn self_graveyard_cast_cost(&self) -> Option<(u32, AdditionalCost)> {
        self.item
            .cast
            .filter(|cast| cast.method == CastMethod::GraveyardAdditionalCosts)
            .and_then(|_| self.item.source?.card.self_graveyard_cast_cost())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn self_graveyard_cost_route_available(
        &self,
        player: Player,
        id: CardId,
        sorcery: bool,
    ) -> bool {
        let object = &self.state.objects[id as usize];
        object.zone == Zone::Graveyard
            && object.owner == player
            && object.is_card()
            && object.card.self_graveyard_cast_cost().is_some()
            && self.permitted_cast_available(
                player,
                id,
                CastMethod::GraveyardAdditionalCosts,
                CastFace::Normal,
                sorcery,
                false,
            )
    }
    pub(super) fn self_graveyard_announcement_fundable(
        &self,
        a: &Announcement,
        cost: ManaCost,
    ) -> bool {
        let source = a.item.source.expect("graveyard route spell source");
        let base = Self::with_kicker_cost(
            Self::with_offspring_cost(
                cost,
                source.card,
                a.item.cast.is_some_and(|c| c.offspring == Some(true)),
            ),
            source.card,
            a.item.cast.is_some_and(|c| c.kicker == Some(true)),
        );
        let Some(total) =
            self.spell_total_for_definition(a.item.controller, source.id, base, source.card)
        else {
            return false;
        };
        let mut proposed = a.clone();
        proposed.cost = Some(cost);
        proposed.total_cost = Some(total);
        proposed.initialize_cast_costs(improvise::potential(self, source.card));
        let goal = self.funding_goal(&proposed);
        let mut world =
            FundingState::from_game_with_convoke(self, &goal, source.card.has_convoke());
        world.set_purpose(self.purpose_for_item(&proposed.item));
        world.set_payment_context(&proposed);
        world.reachable(&goal, &mut HashSet::new())
    }
    pub(super) fn exile_library_top_face_down_play(
        &mut self,
        item: &SpellView,
        libraries: PlayerSet,
        beneficiaries: PlayerSet,
        amount: u16,
        spend_any_mana: bool,
    ) {
        let Some(source) = item.source else {
            return;
        };
        let libraries = self.program_players(libraries, item).collect::<Vec<_>>();
        let beneficiaries = self
            .program_players(beneficiaries, item)
            .collect::<Vec<_>>();
        if beneficiaries.is_empty() {
            return;
        }
        let viewers = beneficiaries
            .iter()
            .fold(0u8, |mask, player| mask | (1 << player.index()));
        for owner in libraries {
            let ids = exile::top_library_cards(self, owner, u128::from(amount));
            // The look precedes replacement planning for exile. Its knowledge
            // is already owned if a replacement decision suspends this event.
            if let Some(&id) = ids.first() {
                let object = self.object_ref(id);
                for &player in &beneficiaries {
                    self.remember_private_library_top(owner, player, object);
                }
            }
            let events = self.move_objects_raw(&ids, Zone::Exile);
            for event in &events {
                if !event.moved || event.destination != Zone::Exile {
                    continue;
                }
                self.state
                    .linked_choices
                    .get_or_insert_default()
                    .hidden
                    .push((source.object, event.after, viewers));
                for &player in &beneficiaries {
                    self.grant_exile_play_permission(player, event.after, false);
                    let permission = self
                        .state
                        .exile_permissions
                        .iter_mut()
                        .find(|permission| {
                            permission.player == player && permission.object == event.after
                        })
                        .expect("new exact exile permission");
                    permission.expiry = ExilePlayExpiry::WhileExiled;
                    permission.spend_any_mana = spend_any_mana;
                }
            }
            self.finish_zone_events(&events, &[]);
        }
    }
}
