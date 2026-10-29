//! Exact, owned graveyard permissions and hand-only alternative casting costs.
use super::*;
use crate::cards::StaticAbilityFragment;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GraveyardCastPermissionView {
    pub object: ObjectRef,
    pub player: Player,
    pub kind: GraveyardCastKind,
    pub expires_turn: u32,
}
pub(super) type Rows = Option<SharedVec<GraveyardCastPermissionView>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<GraveyardCastPermissionView>>()
            + rows.len() * std::mem::size_of::<GraveyardCastPermissionView>()
    })
}
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
impl Game {
    /// The source is already on the stack when casting restrictions are checked.
    pub(super) fn spell_graveyard_requirement_holds(
        &self,
        actor: Player,
        id: CardId,
        definition: Card,
    ) -> bool {
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
            && !object.is_token()
            && self
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
                            && row.expires_turn >= self.state.turn
                    })
                })
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
        let card = &self.state.objects[object.card as usize];
        if self.object_ref(object.card) != object
            || card.zone != Zone::Graveyard
            || card.is_token()
            || card.card.is_emblem()
        {
            return;
        }
        let row = GraveyardCastPermissionView {
            object,
            player,
            kind,
            expires_turn: self.state.turn,
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
        }
    }
    pub(super) fn expire_graveyard_cast_permissions(&mut self) {
        if let Some(rows) = &mut self.state.graveyard_cast_permissions {
            rows.retain(|row| row.expires_turn > self.state.turn);
        }
    }
    pub(super) fn set_cast_face(&self, item: &mut SpellView, face: CastFace) {
        if face == CastFace::Normal {
            return;
        }
        let physical = self.state.objects[item.card as usize].card;
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
        item.source_card = definition;
        item.source_keywords = definition.info().keywords;
        item.source_facts.card = definition;
        item.source_facts.colors = definition.colors();
        item.source_facts.kinds = definition.kind_mask();
        item.source_facts.power = 0;
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
            true,
        )
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "Independent origin permission selects its own cost modifiers"
    )]
    fn permitted_cast_with_origin_rules(
        &self,
        actor: Player,
        card: CardId,
        method: CastMethod,
        face: CastFace,
        sorcery: bool,
        flash_surcharge: bool,
        origin_counter_cost: Option<u16>,
        independent_free: bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
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
        {
            return false;
        }
        let mut item =
            self.cast_preview(actor, card, object.zone, self.object_ref(card), 0, [0; 6]);
        self.set_cast_face(&mut item, face);
        item.cast.as_mut().unwrap().method = method;
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        if independent_free {
            Self::configure_exile_free_cast(&mut item, flash_surcharge);
        }
        if method == CastMethod::ExileLife {
            Self::configure_exile_life_cast(&mut item, flash_surcharge);
        }
        if matches!(
            method,
            CastMethod::ExileLife
                | CastMethod::Free
                | CastMethod::Flashback
                | CastMethod::Harmonize
                | CastMethod::Miracle
        ) && definition.x_symbols() > 0
        {
            item.x = Some(0);
        }
        let available = |plan| {
            self.cast_definition_targets_available(plan, actor, self.object_ref(card), definition)
        };
        let targets_available = match definition.effect() {
            Effect::Modal(modes) => modes.iter().any(|mode| available(mode.targets)),
            _ => available(self.target_spec(&item)),
        };
        if !targets_available
            && !definition
                .gift()
                .is_some_and(|gift| available(gift.promised_targets))
            && !definition.kicked_targets().is_some_and(available)
        {
            return false;
        }
        if definition.x_symbols() > 0
            && !matches!(
                method,
                CastMethod::ExileLife
                    | CastMethod::Free
                    | CastMethod::Flashback
                    | CastMethod::Harmonize
                    | CastMethod::Miracle
            )
        {
            return !self
                .scalar_x_intervals_for_definition(actor, card, definition)
                .is_empty();
        }
        let a = Announcement {
            equip_zero_available: false,
            activation_cost_route: ActivationCostRoute::Printed,
            activation_discount: 0,
            power_up_costs: None,
            free_hand_permission: None,
            origin_counter_cost,
            miracle_cost: self.miracle_cost_for(&item),
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
            additional_group: None,
            creature_counter_cost: None,
            forage: None,
            casualty: None,
            cast_costs: None,
            evidence: None,
            convoke: None,
            harmonizer: None,
            harmonize_discount: 0,
            additional_paid: false,
        };
        if let Some(modes) = a.item.modes() {
            return modes.iter().enumerate().any(|(index, mode)| {
                if !available(mode.targets) {
                    return false;
                }
                let mut choice = a.clone();
                choice.item.mode = Some(u16::try_from(index).expect("mode index"));
                choice.plan = mode.targets;
                choice
                    .cost_options()
                    .any(|cost| self.announcement_cost_fundable(&choice, cost))
            });
        }
        a.cost_options()
            .any(|cost| self.announcement_cost_fundable(&a, cost))
    }
}
