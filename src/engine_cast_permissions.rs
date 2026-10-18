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
                            && row.kind == kind
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
            let row = GraveyardCastPermissionView {
                object: self.object_ref(card),
                player: item.controller,
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
        if method == CastMethod::Free && definition.x_symbols() > 0 {
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
        if definition.x_symbols() > 0 && method != CastMethod::Free {
            return !self.scalar_x_intervals(actor, card).is_empty();
        }
        let a = Announcement {
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
            forage: None,
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
