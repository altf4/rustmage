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
        let object = &self.state.objects[card as usize];
        if object.zone == Zone::Hand && self.hand_play_prohibited() {
            return false;
        }
        let Some(definition) = (match face {
            CastFace::Normal => Some(object.card),
            CastFace::Adventure => object.card.adventure_spell(),
            CastFace::Omen => object.card.omen_spell(),
        }) else {
            return false;
        };
        if flash_surcharge && definition.flash_surcharge().is_none() {
            return false;
        }
        if !sorcery
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
        if matches!(
            method,
            CastMethod::Free | CastMethod::Flashback | CastMethod::Miracle
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
                CastMethod::Free | CastMethod::Flashback | CastMethod::Miracle
            )
        {
            return !self.scalar_x_intervals(actor, card).is_empty();
        }
        let a = Announcement {
            activation_discount: 0,
            free_hand_permission: None,
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
            additional_group: None,
            creature_counter_cost: None,
            forage: None,
            casualty: None,
            cast_costs: None,
            evidence: None,
            convoke: None,
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
