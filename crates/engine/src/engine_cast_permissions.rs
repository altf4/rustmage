//! Exact, owned graveyard permissions and hand-only alternative casting costs.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Default)]
pub(super) struct OriginAlternative {
    pub granted_warp: Option<ManaCost>,
    pub exile_play_surcharge: u32,
    pub web_slinging: Option<ManaCost>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GraveyardCastPermissionView {
    pub object: ObjectRef,
    pub player: Player,
    pub kind: GraveyardCastKind,
    /// None until the player begins their next actual turn.
    pub expires_turn: Option<u32>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl GraveyardCastPermissionView {
    pub(super) fn available_at(&self, turn: u32) -> bool {
        self.expires_turn.is_none_or(|expires| expires >= turn)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) type Rows = Option<SharedVec<GraveyardCastPermissionView>>;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<GraveyardCastPermissionView>>()
            + rows.len() * std::mem::size_of::<GraveyardCastPermissionView>()
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    pub(super) fn has_own_turn_free_hand_cast(self) -> bool {
        self.static_ability_groups().iter().any(|group| {
            group
                .fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::OwnTurnFreeHandInstantSorcery))
        })
    }
    pub(super) fn has_creature_count_free_hand_cast(self) -> bool {
        self.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|f| {
                matches!(
                    f,
                    StaticAbilityFragment::FreeHandCastAtMostControlledCreatures
                )
            })
        })
    }
    pub(super) fn has_free_hand_cast(self) -> bool {
        self.static_ability_groups().iter().any(|group| {
            group
                .fragments
                .iter()
                .any(|f| matches!(f, StaticAbilityFragment::FreeHandCast))
        })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// The source is already on the stack when casting restrictions are checked.
    pub(super) fn spell_graveyard_requirement_holds(
        &self,
        actor: Player,
        id: CardId,
        definition: Card,
    ) -> bool {
        if definition.cast_after_first_three_turns()
            && actor == self.state.active
            && (1..=3).contains(&self.state.players[actor.index()].turns_started)
        {
            return false;
        }
        let minimum = usize::from(definition.cast_graveyard_minimum());
        if minimum == 0 {
            return true;
        }
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(index, object)| {
                *index != id as usize
                    && object.owner == actor
                    && object.zone == Zone::Graveyard
                    && !object.is_token()
                    && !object.is_spell_copy()
            })
            .take(minimum)
            .count()
            >= minimum
    }

    pub(super) fn limited_free_hand_permissions(&self, player: Player) -> Vec<ObjectRef> {
        if player != self.state.active {
            return Vec::new();
        }
        self.state
            .static_sources
            .iter()
            .filter_map(|&id| {
                let object = &self.state.objects[id as usize];
                let reference = self.object_ref(id);
                (object.controller == player
                    && object.zone == Zone::Battlefield
                    && object.card.has_own_turn_free_hand_cast()
                    && self.printed_abilities_available(id)
                    && self
                        .state
                        .used_free_hand_permissions
                        .as_ref()
                        .is_none_or(|rows| !rows.contains(&reference)))
                .then_some(reference)
            })
            .collect()
    }
    pub(super) fn use_free_hand_permission(&mut self, permission: ObjectRef) {
        let used = self
            .state
            .used_free_hand_permissions
            .get_or_insert_default();
        assert!(!used.contains(&permission));
        used.push(permission);
    }
    pub(super) fn free_hand_cast_available(&self, player: Player) -> bool {
        self.state.static_sources.iter().any(|&id| {
            let object = &self.state.objects[id as usize];
            object.controller == player
                && object.zone == Zone::Battlefield
                && object.card.has_free_hand_cast()
                && self.printed_abilities_available(id)
        })
    }
    /// Evaluate against the selected spell face at announcement, before any
    /// additional cost can remove creatures. A free X spell has X equal to zero.
    pub(super) fn creature_count_free_hand_cast_available(
        &self,
        player: Player,
        definition: Card,
    ) -> bool {
        let count = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && self.is_creature(*id as CardId)
            })
            .count();
        if u128::from(definition.fixed_mana_value()) > count as u128 {
            return false;
        }
        self.state.static_sources.iter().any(|&id| {
            let object = &self.state.objects[id as usize];
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card.has_creature_count_free_hand_cast()
                && self.continuous_fragment_available(id, |fragment| {
                    matches!(
                        fragment,
                        StaticAbilityFragment::FreeHandCastAtMostControlledCreatures
                    )
                })
        })
    }
    pub(super) fn has_graveyard_cast(
        &self,
        player: Player,
        card: CardId,
        kind: GraveyardCastKind,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        object.zone == Zone::Graveyard
            && object.owner == player
            && object.is_card()
            && ((kind == GraveyardCastKind::Play
                && object.card.is_land()
                && object.card.has_mayhem_land()
                && self.discarded_to_graveyard_this_turn(card))
                || (kind == GraveyardCastKind::Cast
                    && object.card.static_ability_groups().iter().any(|group| {
                        group.fragments.iter().any(|fragment| {
                        matches!(*fragment, StaticAbilityFragment::SelfGraveyardCastIf(condition)
                            if self.stat_value(condition, self.object_ref(card), player) != 0)
                    })
                    }))
                || (matches!(kind, GraveyardCastKind::Cast | GraveyardCastKind::Play)
                    && self.state.active == player
                    && cast_origins::has_fragment(self, player, |fragment| {
                        matches!(
                            fragment,
                            StaticAbilityFragment::PlayFromGraveyardDuringOwnTurn
                        )
                    }))
                || self
                    .state
                    .graveyard_cast_permissions
                    .as_ref()
                    .is_some_and(|rows| {
                        rows.iter().any(|row| {
                            row.player == player
                                && row.object == self.object_ref(card)
                                && (row.kind == kind
                                    || (kind == GraveyardCastKind::Cast
                                        && row.kind == GraveyardCastKind::Play))
                                && row.available_at(self.state.turn)
                        })
                    }))
    }
    pub(super) fn has_graveyard_cast_face(
        &self,
        player: Player,
        card: CardId,
        face: CastFace,
    ) -> bool {
        self.has_graveyard_cast(player, card, GraveyardCastKind::Cast)
            || (face == CastFace::Adventure
                && self.has_graveyard_cast(player, card, GraveyardCastKind::Adventure))
    }
    pub(super) fn grant_graveyard_cast(&mut self, item: &SpellView, kind: GraveyardCastKind) {
        for bound in &item.targets {
            if !self.same_target_object(*bound) {
                continue;
            }
            let Target::Card(card) = bound.target else {
                continue;
            };
            if self.state.objects[card as usize].zone != Zone::Graveyard {
                continue;
            }
            self.grant_graveyard_permission(item.controller, self.object_ref(card), kind);
        }
    }
    pub(super) fn grant_graveyard_permission(
        &mut self,
        player: Player,
        object: ObjectRef,
        kind: GraveyardCastKind,
    ) {
        self.grant_graveyard_permission_until(player, object, kind, Some(self.state.turn));
    }
    fn grant_graveyard_permission_until(
        &mut self,
        player: Player,
        object: ObjectRef,
        kind: GraveyardCastKind,
        expires_turn: Option<u32>,
    ) {
        let card = &self.state.objects[object.card as usize];
        if self.object_ref(object.card) != object || card.zone != Zone::Graveyard || !card.is_card()
        {
            return;
        }
        if kind == GraveyardCastKind::Adventure
            && (card.owner != player || card.card.adventure_spell().is_none())
        {
            return;
        }
        let row = GraveyardCastPermissionView {
            object,
            player,
            kind,
            expires_turn,
        };
        let rows = self
            .state
            .graveyard_cast_permissions
            .get_or_insert_with(Default::default);
        if !rows.contains(&row) {
            rows.try_reserve(1)
                .capacity("graveyard casting permission allocation exhausted");
            rows.push(row);
        }
    }
    pub(super) fn grant_departed_adventure_until_next_turn(&mut self, item: &SpellView) {
        let Some(movement) = item.source_move.as_ref().filter(|movement| {
            movement.origin == Zone::Battlefield && movement.destination == Zone::Graveyard
        }) else {
            return;
        };
        for object in movement.objects_after() {
            self.grant_graveyard_permission_until(
                item.controller,
                object,
                GraveyardCastKind::Adventure,
                None,
            );
        }
    }
    pub(super) fn grant_milled_play(&mut self, player: Player) {
        let milled = self.state.resolving.as_ref().unwrap().milled.clone();
        for object in milled {
            if self.object_ref(object.card) != object {
                continue;
            }
            match self.state.objects[object.card as usize].zone {
                Zone::Graveyard => {
                    self.grant_graveyard_permission(player, object, GraveyardCastKind::Play)
                }
                Zone::Exile => self.grant_exile_play_permission(player, object, false),
                _ => (),
            }
        }
    }
    pub(super) fn forget_graveyard_cast_permission(&mut self, object: ObjectRef) {
        if let Some(rows) = &mut self.state.graveyard_cast_permissions {
            rows.retain(|row| row.object != object);
            if rows.is_empty() {
                self.state.graveyard_cast_permissions = None;
            }
        }
    }
    pub(super) fn begin_graveyard_cast_permission_turn(&mut self) {
        if let Some(rows) = &mut self.state.graveyard_cast_permissions {
            for row in rows
                .iter_mut()
                .filter(|row| row.player == self.state.active)
            {
                row.expires_turn.get_or_insert(self.state.turn);
            }
        }
    }
    pub(super) fn expire_graveyard_cast_permissions(&mut self) {
        if let Some(rows) = &mut self.state.graveyard_cast_permissions {
            rows.retain(|row| {
                row.expires_turn
                    .is_none_or(|expires| expires > self.state.turn)
            });
            if rows.is_empty() {
                self.state.graveyard_cast_permissions = None;
            }
        }
    }
    pub(super) fn set_cast_face(&self, item: &mut SpellView, face: CastFace) {
        if face == CastFace::Normal {
            return;
        }
        let physical =
            self.state.objects[item.source.expect("cast spell has source").id as usize].card;
        let definition = match face {
            CastFace::Adventure => physical.adventure_spell(),
            CastFace::Omen => physical.omen_spell(),
            CastFace::Back => physical.back_face(),
            CastFace::Normal => unreachable!(),
        }
        .expect("validated spell face");
        let cast = item.cast.as_mut().unwrap();
        cast.face = face;
        cast.physical_card = Some(physical);
        item.set_source_definition(definition);
        item.source_keywords = definition.info().keywords;
        item.source
            .as_mut()
            .expect("spell source exists")
            .facts
            .card = definition;
        item.source
            .as_mut()
            .expect("spell source exists")
            .facts
            .colors = definition.colors();
        item.source
            .as_mut()
            .expect("spell source exists")
            .facts
            .kinds = definition.kind_mask();
        item.source
            .as_mut()
            .expect("spell source exists")
            .facts
            .power = 0;
    }
    pub(super) fn permitted_cast_available(
        &self,
        actor: Player,
        card: CardId,
        method: CastMethod,
        face: CastFace,
        sorcery: bool,
        flash_surcharge: bool,
    ) -> bool {
        self.permitted_cast_with_origin_cost(
            actor,
            card,
            method,
            face,
            sorcery,
            flash_surcharge,
            None,
        )
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "Mana alternative and mandatory origin cost are independent"
    )]
    pub(super) fn permitted_cast_with_origin_cost(
        &self,
        actor: Player,
        card: CardId,
        method: CastMethod,
        face: CastFace,
        sorcery: bool,
        flash_surcharge: bool,
        origin_counter_cost: Option<u16>,
    ) -> bool {
        self.permitted_cast_with_origin_rules(
            actor,
            card,
            method,
            face,
            sorcery,
            flash_surcharge,
            origin_counter_cost,
            None,
            None,
            false,
        )
    }
    pub(super) fn permitted_exile_free_cast_available(
        &self,
        actor: Player,
        card: CardId,
        face: CastFace,
        sorcery: bool,
        flash_surcharge: bool,
    ) -> bool {
        self.permitted_cast_with_origin_rules(
            actor,
            card,
            CastMethod::Free,
            face,
            sorcery,
            flash_surcharge,
            None,
            None,
            None,
            true,
        )
    }
    pub(super) fn permitted_free_additional_origin_cast(
        &self,
        actor: Player,
        card: CardId,
        face: CastFace,
        sorcery: bool,
        flash_surcharge: bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if object.zone != Zone::Graveyard || object.owner != actor || !object.is_card() {
            return false;
        }
        let Some(_) = object.card.self_graveyard_cast_cost() else {
            return false;
        };
        self.permitted_cast_with_origin_rules(
            actor,
            card,
            CastMethod::Free,
            face,
            sorcery,
            flash_surcharge,
            None,
            Some(object.card),
            None,
            false,
        )
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "Independent origin permission selects its own cost modifiers"
    )]
    pub(super) fn permitted_cast_with_origin_rules(
        &self,
        actor: Player,
        card: CardId,
        method: CastMethod,
        face: CastFace,
        sorcery: bool,
        flash_surcharge: bool,
        origin_counter_cost: Option<u16>,
        origin_additional_cost: Option<Card>,
        origin_life_cost: Option<u32>,
        independent_free: bool,
    ) -> bool {
        self.permitted_cast_with_alternative(
            actor,
            card,
            method,
            face,
            sorcery,
            flash_surcharge,
            origin_counter_cost,
            origin_additional_cost,
            origin_life_cost,
            independent_free,
            OriginAlternative::default(),
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "The selected alternative, additional origin costs and timing are independent"
    )]
    pub(super) fn permitted_cast_with_alternative(
        &self,
        actor: Player,
        card: CardId,
        method: CastMethod,
        face: CastFace,
        sorcery: bool,
        flash_surcharge: bool,
        origin_counter_cost: Option<u16>,
        origin_additional_cost: Option<Card>,
        origin_life_cost: Option<u32>,
        independent_free: bool,
        alternative: OriginAlternative,
    ) -> bool {
        let OriginAlternative {
            granted_warp,
            exile_play_surcharge,
            web_slinging,
        } = alternative;
        let object = &self.state.objects[card as usize];
        if method == CastMethod::GraveyardAdditionalCosts
            && (object.zone != Zone::Graveyard
                || object.owner != actor
                || !object.is_card()
                || object.card.self_graveyard_cast_cost().is_none())
        {
            return false;
        }
        if face == CastFace::Back
            && object.card.transforming_double_face()
            && !self
                .state
                .resolution_free_casts
                .as_ref()
                .is_some_and(|casts| {
                    casts.siege_transformed
                        && casts.player == actor
                        && casts.remaining.contains(&self.object_ref(card))
                })
        {
            return false;
        }
        if object.zone == Zone::Hand && self.hand_play_prohibited() {
            return false;
        }
        let Some(definition) = (match face {
            CastFace::Normal => Some(object.card),
            CastFace::Adventure => object.card.adventure_spell(),
            CastFace::Omen => object.card.omen_spell(),
            CastFace::Back => object.card.back_face(),
        }) else {
            return false;
        };
        if method == CastMethod::Plot && definition.is_land() {
            return false;
        }
        if method.is_mayhem()
            && (definition.is_land()
                || object.owner != actor
                || !self.discarded_to_graveyard_this_turn(card)
                || match method {
                    CastMethod::Mayhem => definition.mayhem_cost_options().is_none(),
                    CastMethod::GrantedMayhem => {
                        crate::cards::mana_symbols::printed_cost(definition.info().mana_cost)
                            .is_none()
                    }
                    _ => unreachable!("Mayhem method"),
                })
        {
            return false;
        }
        if method == CastMethod::GrantedHarmonize
            && (definition.is_land()
                || !self.has_graveyard_cast(actor, card, GraveyardCastKind::Harmonize)
                || crate::cards::mana_symbols::printed_cost(definition.info().mana_cost).is_none())
        {
            return false;
        }
        if !self.spell_graveyard_requirement_holds(actor, card, definition) {
            return false;
        }
        if flash_surcharge && definition.flash_surcharge().is_none() {
            return false;
        }
        if !sorcery
            && method != CastMethod::Sneak
            && !flash_surcharge
            && !definition.is_instant()
            && !definition.info().keywords.contains(Keywords::FLASH)
            && !self.cast_timing_granted(actor, definition)
            && !definition
                .behold_cast()
                .is_some_and(|spec| spec.grants_flash)
        {
            return false;
        }
        let mut item =
            self.cast_preview(actor, card, object.zone, self.object_ref(card), 0, [0; 6]);
        self.set_cast_face(&mut item, face);
        item.cast.as_mut().unwrap().method = method;
        if method == CastMethod::Sneak {
            Self::configure_sneak(&mut item);
        }
        let warp =
            (method == CastMethod::Warp).then(|| warp::configure_cast(&mut item, granted_warp));
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        if independent_free {
            Self::configure_exile_free_cast(&mut item, flash_surcharge);
        }
        if method == CastMethod::Airbend {
            Self::configure_airbend_cast(&mut item, flash_surcharge);
        }
        if method.pays_life_for_mana_value() {
            Self::configure_mana_value_life_cast(&mut item, method, flash_surcharge);
        }
        if exile_play_surcharge > 0 {
            Self::configure_taxed_exile_cast(&mut item, exile_play_surcharge);
        }
        if matches!(
            method,
            CastMethod::Airbend
                | CastMethod::Free
                | CastMethod::Plot
                | CastMethod::Flashback
                | CastMethod::Harmonize
                | CastMethod::Mayhem
                | CastMethod::WebSlinging
                | CastMethod::Miracle
        ) && definition.x_symbols() > 0
            && (method != CastMethod::Harmonize || definition.harmonize_x_symbols() == 0)
        {
            item.x = Some(0);
        }
        let available = |plan| {
            self.cast_definition_targets_available(plan, actor, self.object_ref(card), definition)
        };
        let teamwork = definition.teamwork().is_some();
        let kicker_changes_targets = definition.kicked_targets().is_some();
        let bargain_changes_targets = definition.bargain_targets().is_some();
        if !teamwork && !kicker_changes_targets && !bargain_changes_targets {
            let targets_available = match definition.effect() {
                Effect::Modal(modes) => modes.iter().any(|mode| available(mode.targets)),
                _ => available(self.target_spec(&item)),
            };
            if !targets_available
                && !definition
                    .gift()
                    .is_some_and(|gift| available(gift.promised_targets))
            {
                return false;
            }
        }
        let scalar_x = warp.map_or_else(
            || {
                definition.x_symbols() > 0
                    && !matches!(
                        method,
                        CastMethod::LinkedExileCounterCost
                            | CastMethod::ExileLife
                            | CastMethod::LibraryLife
                            | CastMethod::Airbend
                            | CastMethod::Free
                            | CastMethod::Plot
                            | CastMethod::Flashback
                            | CastMethod::Harmonize
                            | CastMethod::GrantedHarmonize
                            | CastMethod::GrantedMayhem
                            | CastMethod::Mayhem
                            | CastMethod::Miracle
                            | CastMethod::Sneak
                            | CastMethod::WebSlinging
                    )
            },
            |spec| spec.x_symbols > 0,
        );
        if scalar_x && origin_life_cost.is_none() && warp.is_none() && exile_play_surcharge == 0 {
            return !self
                .scalar_x_intervals_for_definition(actor, card, definition)
                .is_empty();
        }
        let mut a = Announcement {
            behold_required: !sorcery
                && method != CastMethod::Sneak
                && !flash_surcharge
                && !definition.is_instant()
                && !definition.info().keywords.contains(Keywords::FLASH)
                && !self.cast_timing_granted(actor, definition),
            equip_zero_available: false,
            exhaust_reuse_authorized: false,
            activation_cost_route: ActivationCostRoute::Printed,
            activation_discount: 0,
            activation_generic_increase: 0,
            activation_choice: None,
            mana_output: None,
            activation_reductions: None,
            free_hand_permission: None,
            origin_counter_cost,
            origin_additional_cost,
            origin_alternative_cost: granted_warp
                .or(web_slinging)
                .or_else(|| self.origin_alternative_cost_for(&item)),
            item,
            spec: TargetSpec::NONE,
            plan: TargetPlan::Single(TargetSpec::NONE),
            group: 0,
            trigger: false,
            payment: None,
            cost: None,
            total_cost: None,
            counter_payment: Counters::default(),
            pay_window: true,
            production: None,
            additional: None,
            additional_life: false,
            additional_group_complete: false,
            additional_group: None,
            creature_counter_cost: None,
            forage: None,
            casualty: None,
            conspire: None,
            cast_costs: None,
            evidence: None,
            convoke: None,
            harmonizer: None,
            harmonize_discount: 0,
            additional_paid: false,
        };
        let life = warp
            .map(|spec| spec.life)
            .filter(|&life| life != 0)
            .or(origin_life_cost);
        if let Some(life) = life {
            a.add_origin_life_cost(life);
        }
        if matches!(
            method,
            CastMethod::GraveyardAdditionalCosts
                | CastMethod::LinkedExileCounterCost
                | CastMethod::WebSlinging
        ) || origin_additional_cost.is_some()
            || life.is_some() && warp.is_none()
        {
            a.initialize_cast_costs(convoke::cost_potential(self, definition));
        }
        if scalar_x
            || (definition.x_symbols() > 0
                && matches!(
                    method,
                    CastMethod::LinkedExileCounterCost | CastMethod::GrantedMayhem
                ))
        {
            return !self
                .scalar_x_intervals_with_announcement(actor, card, definition, Some(&a))
                .is_empty();
        }
        if teamwork {
            return self.teamwork_intent_fundable(&a, false)
                || self.teamwork_intent_fundable(&a, true);
        }
        let fundable = |choice: &Announcement| {
            if bargain_changes_targets {
                return self.visit_bargain_choices(choice, |_| true);
            }
            if kicker_changes_targets {
                return self.kicker_choice_fundable(choice, false)
                    || self.kicker_choice_fundable(choice, true);
            }
            choice
                .cost_options()
                .any(|cost| self.announcement_cost_fundable(choice, cost))
        };
        if let Some(modes) = a.item.modes() {
            return modes.iter().enumerate().any(|(index, mode)| {
                if !kicker_changes_targets && !bargain_changes_targets && !available(mode.targets) {
                    return false;
                }
                let mut choice = a.clone();
                choice.item.mode = Some(u16::try_from(index).expect("mode index"));
                choice.plan = mode.targets;
                fundable(&choice)
            });
        }
        fundable(&a)
    }
}
