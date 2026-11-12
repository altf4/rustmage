//! Optional zero mana base for an independently authorized spell cast from exile.
use super::*;
use crate::cards::StaticAbilityFragment;
use std::num::NonZeroU32;

/// The independently granted permission used to cast the spell from exile.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExileZeroOrigin {
    Ordinary,
    Taxed { surcharge: NonZeroU32 },
    LinkedPaid { source: ObjectRef },
    LinkedCounters { source: ObjectRef },
    PaidResolution,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ExileZeroUseView {
    pub source: ObjectRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability_origin: Option<u64>,
    pub player: Player,
    pub turn: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ExileZeroUses {
    completed: Vec<ExileZeroUseView>,
    pending: Vec<(u64, ExileZeroUseView)>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ExileZeroUses {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.completed.len() * std::mem::size_of::<ExileZeroUseView>()
            + self.pending.len() * std::mem::size_of::<(u64, ExileZeroUseView)>()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn exile_zero_uses(&self) -> Vec<ExileZeroUseView> {
        self.state
            .linked_choices
            .as_ref()
            .and_then(|data| data.exile_zero_uses.as_deref())
            .map_or_else(Vec::new, |uses| uses.completed.clone())
    }

    pub(super) fn exile_zero_providers(&self, player: Player) -> Vec<ObjectRef> {
        let uses = self
            .state
            .linked_choices
            .as_ref()
            .and_then(|data| data.exile_zero_uses.as_deref());
        self.state
            .static_sources
            .iter()
            .copied()
            .filter_map(|id| {
                if self.state.objects[id as usize].controller != player
                    || !self.active_rule_fragment(id, |fragment| {
                        matches!(fragment, StaticAbilityFragment::ExileCastForZeroOncePerTurn)
                    })
                {
                    return None;
                }
                let source = self.object_ref(id);
                let origin = self.copy_ability_origin(source);
                let used = |row: &ExileZeroUseView| {
                    row.source == source
                        && row.ability_origin == origin
                        && row.turn == self.state.turn
                };
                (!uses.is_some_and(|uses| {
                    uses.completed.iter().any(used) || uses.pending.iter().any(|(_, row)| used(row))
                }))
                .then_some(source)
            })
            .collect()
    }

    fn exile_zero_origin_terms(
        &self,
        player: Player,
        object: ObjectRef,
        face: CastFace,
        origin: ExileZeroOrigin,
    ) -> Option<(Option<u16>, cast_permissions::OriginAlternative)> {
        let origin = match origin {
            ExileZeroOrigin::Ordinary => PaidCastOrigin::Ordinary,
            ExileZeroOrigin::PaidResolution => PaidCastOrigin::PaidResolution,
            ExileZeroOrigin::Taxed { surcharge } => PaidCastOrigin::TaxedExile { surcharge },
            ExileZeroOrigin::LinkedPaid { source } => {
                PaidCastOrigin::LinkedExilePaid { object, source }
            }
            ExileZeroOrigin::LinkedCounters { source } => {
                PaidCastOrigin::LinkedExileCounters { source }
            }
        };
        let terms = self.paid_cast_terms(player, object.card, face, origin)?;
        Some((terms.counters, terms.alternative))
    }

    pub(super) fn visit_ordinary_exile_zero_casts(
        &self,
        player: Player,
        card: CardId,
        sorcery: bool,
        providers: &[ObjectRef],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if providers.is_empty() {
            return false;
        }
        CastFace::choices(self.state.objects[card as usize].card).any(|face| {
            let permitted = if face == CastFace::Normal {
                self.can_play_exiled(player, card)
            } else {
                self.can_cast_exiled_card(player, card)
            };
            permitted
                && self.visit_exile_zero_face(
                    player,
                    self.object_ref(card),
                    face,
                    sorcery,
                    ExileZeroOrigin::Ordinary,
                    providers,
                    emit,
                )
        })
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "The existing origin and alternative-cost provider are independent"
    )]
    pub(super) fn visit_exile_zero_face(
        &self,
        player: Player,
        object: ObjectRef,
        face: CastFace,
        sorcery: bool,
        origin: ExileZeroOrigin,
        providers: &[ObjectRef],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if providers.is_empty() {
            return false;
        }
        let Some(definition) = face.definition(self.state.objects[object.card as usize].card)
        else {
            return false;
        };
        if definition.is_land() || matches!(definition.effect(), Effect::Unsupported) {
            return false;
        }
        let Some((counters, terms)) = self.exile_zero_origin_terms(player, object, face, origin)
        else {
            return false;
        };
        let independent_terms = !matches!(
            origin,
            ExileZeroOrigin::Ordinary | ExileZeroOrigin::PaidResolution
        );
        for flash_surcharge in [false, true] {
            if flash_surcharge
                && (origin == ExileZeroOrigin::PaidResolution
                    || definition.flash_surcharge().is_none())
            {
                continue;
            }
            if self.permitted_cast_with_alternative(
                player,
                object.card,
                CastMethod::Free,
                face,
                sorcery,
                flash_surcharge,
                counters,
                None,
                None,
                independent_terms,
                terms,
            ) && providers.iter().copied().any(|provider| {
                emit(Action::AnnounceExileForZero {
                    object,
                    provider,
                    face,
                    flash_surcharge,
                    origin,
                })
            }) {
                return true;
            }
        }
        false
    }

    pub(super) fn start_exile_zero_cast(
        &mut self,
        player: Player,
        object: ObjectRef,
        provider: ObjectRef,
        face: CastFace,
        flash_surcharge: bool,
        origin: ExileZeroOrigin,
    ) {
        let usage = ExileZeroUseView {
            source: provider,
            ability_origin: self.copy_ability_origin(provider),
            player,
            turn: self.state.turn,
        };
        if origin == ExileZeroOrigin::PaidResolution {
            self.cast_during_resolution_configured(
                player,
                object,
                face,
                Some(CastMethod::Free),
                None,
                |game, item| {
                    game.configure_paid_resolution_offer(item);
                    game.approve_exile_zero_cast(item.id, usage, false);
                },
            );
            return;
        }

        let (counters, terms) = self
            .exile_zero_origin_terms(player, object, face, origin)
            .expect("approved exact exile origin");
        let mut item = self.cast_item(player, object.card, [0; 6]);
        self.set_cast_face(&mut item, face);
        if origin == ExileZeroOrigin::Ordinary {
            let cast = item.cast.as_mut().unwrap();
            cast.method = CastMethod::Free;
            cast.flash_surcharge = flash_surcharge;
            if item.source.unwrap().card.x_symbols() > 0 {
                item.x = Some(0);
            }
        } else {
            Self::configure_exile_free_cast(&mut item, flash_surcharge);
        }
        if terms.exile_play_surcharge != 0 {
            Self::configure_taxed_exile_cast(&mut item, terms.exile_play_surcharge);
        }
        self.approve_exile_zero_cast(item.id, usage, origin == ExileZeroOrigin::Ordinary);
        self.open_payment_announcement_configured(
            item,
            Counters::default(),
            None,
            counters,
            |announcement| {
                if let Some(permission) = terms.linked_exile_paid {
                    announcement.limited_cast_permission =
                        Some(Box::new(LimitedCastPermission::LinkedExilePaid(permission)));
                }
            },
        );
    }

    fn approve_exile_zero_cast(&mut self, stack: u64, usage: ExileZeroUseView, ordinary: bool) {
        if ordinary {
            self.select_ordinary_paid_exile_cast(stack);
        }
        let data = self.state.linked_choices.get_or_insert_default();
        if !ordinary {
            data.play_approvals.retain(|row| row.0 != stack);
        }
        data.exile_zero_uses
            .get_or_insert_default()
            .pending
            .push((stack, usage));
    }

    pub(super) fn commit_exile_zero_cast(&mut self, stack: u64) {
        let Some(uses) = self
            .state
            .linked_choices
            .as_mut()
            .and_then(|data| data.exile_zero_uses.as_deref_mut())
        else {
            return;
        };
        if let Some(index) = uses.pending.iter().position(|&(id, _)| id == stack) {
            let (_, usage) = uses.pending.remove(index);
            uses.completed.push(usage);
        }
    }
}
