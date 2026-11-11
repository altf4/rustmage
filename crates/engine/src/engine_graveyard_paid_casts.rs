//! Owned, independently consumable paid casting permissions for this turn.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PaidGraveyardCastPermission {
    pub id: u64,
    pub source: ObjectRef,
    pub player: Player,
    pub turn: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_counter: Option<CounterKind>,
    pub additional_types: &'static [&'static str],
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn grant_creature_graveyard_cast_this_turn(
        &mut self,
        item: &SpellView,
        entry_counter: Option<CounterKind>,
        additional_types: &'static [&'static str],
    ) {
        let Some(source) = item.source else { return };
        assert!(
            additional_types
                .iter()
                .all(|subtype| crate::cards::is_creature_type(subtype))
        );
        let id = self.next_stack_id();
        self.state
            .linked_choices
            .get_or_insert_default()
            .graveyard_paid_permissions
            .push(PaidGraveyardCastPermission {
                id,
                source: source.object,
                player: item.controller,
                turn: self.state.turn,
                entry_counter,
                additional_types,
            });
    }

    pub(super) fn paid_graveyard_permission(
        &self,
        actor: Player,
        object: ObjectRef,
        face: CastFace,
        permission: u64,
    ) -> Option<PaidGraveyardCastPermission> {
        let card = self.state.objects.get(object.card as usize)?;
        if self.object_ref(object.card) != object
            || card.zone != Zone::Graveyard
            || card.owner != actor
            || !card.is_card()
        {
            return None;
        }
        let face = match face {
            CastFace::Normal => Some(card.card),
            CastFace::Adventure => card.card.adventure_spell(),
            CastFace::Omen => card.card.omen_spell(),
            CastFace::Back => card.card.back_face(),
        }?;
        if !face.is_creature() || matches!(face.effect(), Effect::Unsupported) {
            return None;
        }
        self.state
            .linked_choices
            .as_ref()?
            .graveyard_paid_permissions
            .iter()
            .copied()
            .find(|row| row.player == actor && row.turn == self.state.turn && row.id == permission)
    }

    pub(super) fn visit_paid_graveyard_casts(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Some(data) = &self.state.linked_choices else {
            return false;
        };
        let object = self.object_ref(card);
        for row in &data.graveyard_paid_permissions {
            if row.player != actor || row.turn != self.state.turn {
                continue;
            }
            for face in [
                CastFace::Normal,
                CastFace::Adventure,
                CastFace::Omen,
                CastFace::Back,
            ] {
                let Some(permission) = self.paid_graveyard_permission(actor, object, face, row.id)
                else {
                    continue;
                };
                if self.visit_web_slinging_face(
                    actor,
                    card,
                    face,
                    sorcery,
                    WebSlingingOrigin::GraveyardPaid {
                        object,
                        permission: row.id,
                    },
                    emit,
                ) {
                    return true;
                }
                let costs = [
                    (CastMethod::Normal, false),
                    (CastMethod::Normal, true),
                    (CastMethod::Sneak, false),
                ];
                for (method, flash_surcharge) in costs {
                    if method == CastMethod::Sneak
                        && (face != CastFace::Normal
                            || self.state.active != actor
                            || self.state.step != Step::Blockers
                            || !self.has_sneak_cost(actor, card))
                    {
                        continue;
                    }
                    if self.permitted_cast_with_alternative(
                        actor,
                        card,
                        method,
                        face,
                        sorcery,
                        flash_surcharge,
                        None,
                        None,
                        None,
                        false,
                        cast_permissions::OriginAlternative {
                            paid_graveyard: Some(permission),
                            ..Default::default()
                        },
                    ) && emit(Action::AnnounceGraveyardPaid {
                        object,
                        permission: permission.id,
                        face,
                        method,
                        flash_surcharge,
                    }) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(super) fn start_paid_graveyard_cast(
        &mut self,
        actor: Player,
        object: ObjectRef,
        permission: u64,
        face: CastFace,
        method: CastMethod,
        flash_surcharge: bool,
    ) {
        let permission = self
            .paid_graveyard_permission(actor, object, face, permission)
            .expect("selected paid graveyard cast permission");
        let mut item = self.cast_item(actor, object.card, [0; 6]);
        self.set_cast_face(&mut item, face);
        if method == CastMethod::Sneak {
            Self::configure_sneak(&mut item);
        }
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        self.apply_paid_graveyard_spell_types(&mut item, permission);
        self.open_payment_announcement_configured(item, Counters::default(), None, None, |a| {
            a.limited_cast_permission =
                Some(Box::new(LimitedCastPermission::PaidGraveyard(permission)));
        });
    }

    pub(super) fn consume_paid_graveyard_permission(
        &mut self,
        item: &SpellView,
        permission: PaidGraveyardCastPermission,
    ) {
        let permissions = &mut self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .graveyard_paid_permissions;
        let position = permissions
            .iter()
            .position(|row| *row == permission)
            .expect("retained paid graveyard grant");
        permissions.remove(position);
        if let Some(kind) = permission.entry_counter {
            self.record_cast_entry_counters(item.source.expect("paid spell").object, kind, 1);
        }
    }

    pub(super) fn expire_paid_graveyard_permissions(&mut self) {
        if let Some(data) = &mut self.state.linked_choices {
            data.graveyard_paid_permissions.clear();
        }
    }
}
