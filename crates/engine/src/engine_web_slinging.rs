//! CR 702.188: a paid casting alternative, independent of permission and timing.
use super::*;
use crate::cards::StaticAbilityFragment;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) const RETURN_COST: AdditionalCost = AdditionalCost {
    kind: EffectChoiceKind::ReturnTappedCreature,
    filter: SacrificeFilter {
        keywords: Keywords::NONE,
        source_only: false,
        kinds: 1,
        nonland: false,
        other: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
};

/// Exact permission when a casting route has an independent cost or consequence.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebSlingingOrigin {
    Ordinary,
    GraveyardType {
        permission: ObjectRef,
        permanent_type: PermanentType,
    },
    GraveyardFinality,
    GraveyardAdditional,
    GraveyardCounters,
    GraveyardLife {
        life: u32,
        entry_counter: CounterKind,
    },
    LinkedExileCounters {
        source: ObjectRef,
    },
    TaxedExile {
        surcharge: std::num::NonZeroU32,
    },
    PaidResolution,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn web_slinging_bases(&self, actor: Player, definition: Card) -> Vec<ManaCost> {
        let mut costs: Vec<_> = definition.web_slinging_cost().into_iter().collect();
        if definition.is_legendary() && definition.colors() != 0 {
            for &id in &self.state.static_sources {
                let source = &self.state.objects[id as usize];
                if source.zone != Zone::Battlefield || source.controller != actor {
                    continue;
                }
                for fragment in source
                    .card
                    .static_ability_groups()
                    .iter()
                    .flat_map(|group| group.fragments)
                {
                    let StaticAbilityFragment::ColoredLegendaryWebSlinging(cost) = *fragment else {
                        continue;
                    };
                    if !costs.contains(&cost)
                        && self.continuous_fragment_available(id, |found| {
                            std::ptr::eq(found, fragment)
                        })
                    {
                        costs.push(cost);
                    }
                }
            }
        }
        costs
    }

    pub(super) fn visit_web_slinging(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if !object.is_card() || object.zone == Zone::Battlefield {
            return false;
        }
        for (face, definition) in [
            (CastFace::Normal, Some(object.card)),
            (CastFace::Adventure, object.card.adventure_spell()),
            (CastFace::Omen, object.card.omen_spell()),
            (CastFace::Back, object.card.back_face()),
        ] {
            let Some(definition) = definition.filter(|card| !card.is_land()) else {
                continue;
            };
            let ordinary = match object.zone {
                Zone::Hand => object.owner == actor,
                Zone::Graveyard => self.has_graveyard_cast_face(actor, card, face),
                Zone::Exile if face == CastFace::Normal => self.can_play_exiled(actor, card),
                Zone::Exile => self.can_cast_exiled_card(actor, card),
                Zone::Library => self.can_cast_library_face(actor, card, definition),
                _ => false,
            };
            if ordinary
                && self.visit_web_slinging_face(
                    actor,
                    card,
                    face,
                    sorcery,
                    WebSlingingOrigin::Ordinary,
                    emit,
                )
            {
                return true;
            }
            if face == CastFace::Normal && object.zone == Zone::Graveyard && object.owner == actor {
                for (eligible, origin) in [
                    (
                        object.card.self_graveyard_cast_cost().is_some(),
                        WebSlingingOrigin::GraveyardAdditional,
                    ),
                    (
                        object.card.graveyard_counter_cost().is_some(),
                        WebSlingingOrigin::GraveyardCounters,
                    ),
                ] {
                    if eligible
                        && self.visit_web_slinging_face(actor, card, face, sorcery, origin, emit)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(super) fn visit_web_slinging_face(
        &self,
        actor: Player,
        card: CardId,
        face: CastFace,
        sorcery: bool,
        origin: WebSlingingOrigin,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let physical = self.state.objects[card as usize].card;
        let definition = match face {
            CastFace::Normal => Some(physical),
            CastFace::Adventure => physical.adventure_spell(),
            CastFace::Omen => physical.omen_spell(),
            CastFace::Back => physical.back_face(),
        };
        let Some(definition) = definition
            .filter(|card| !card.is_land() && !matches!(card.effect(), Effect::Unsupported))
        else {
            return false;
        };
        let origin_counter_cost = match origin {
            WebSlingingOrigin::LinkedExileCounters { source } => {
                self.linked_counter_permission(actor, source, card, face)
            }
            WebSlingingOrigin::GraveyardCounters => physical.graveyard_counter_cost(),
            _ => None,
        };
        let origin_additional_cost =
            (origin == WebSlingingOrigin::GraveyardAdditional).then_some(physical);
        let origin_life_cost = match origin {
            WebSlingingOrigin::GraveyardLife { life, .. } => Some(life),
            _ => None,
        };
        let exile_play_surcharge = match origin {
            WebSlingingOrigin::TaxedExile { surcharge } => surcharge.get(),
            _ => 0,
        };
        for cost in self.web_slinging_bases(actor, definition) {
            for flash_surcharge in [false, true] {
                if flash_surcharge
                    && (origin == WebSlingingOrigin::PaidResolution
                        || definition.flash_surcharge().is_none())
                {
                    continue;
                }
                if self.permitted_cast_with_alternative(
                    actor,
                    card,
                    CastMethod::WebSlinging,
                    face,
                    sorcery,
                    flash_surcharge,
                    origin_counter_cost,
                    origin_additional_cost,
                    origin_life_cost,
                    false,
                    cast_permissions::OriginAlternative {
                        exile_play_surcharge,
                        web_slinging: Some(cost),
                        ..Default::default()
                    },
                ) && emit(Action::AnnounceWebSlinging {
                    card,
                    face,
                    cost,
                    flash_surcharge,
                    origin,
                }) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn start_web_slinging(
        &mut self,
        actor: Player,
        card: CardId,
        face: CastFace,
        cost: ManaCost,
        flash_surcharge: bool,
        origin: WebSlingingOrigin,
    ) {
        let mut entry_counter = None;
        let mut origin_additional_cost = None;
        let mut origin_counter_cost = None;
        let mut origin_life_cost = None;
        let mut exile_play_surcharge = None;
        match origin {
            WebSlingingOrigin::Ordinary => {}
            WebSlingingOrigin::GraveyardType {
                permission,
                permanent_type,
            } => self.consume_graveyard_type(permission, permanent_type),
            WebSlingingOrigin::GraveyardFinality => entry_counter = Some(CounterKind::Finality),
            WebSlingingOrigin::GraveyardAdditional => {
                origin_additional_cost = Some(self.state.objects[card as usize].card);
            }
            WebSlingingOrigin::GraveyardCounters => {
                origin_counter_cost = self.state.objects[card as usize]
                    .card
                    .graveyard_counter_cost();
            }
            WebSlingingOrigin::GraveyardLife {
                life,
                entry_counter: kind,
            } => {
                entry_counter = Some(kind);
                origin_life_cost = Some(life);
            }
            WebSlingingOrigin::LinkedExileCounters { source } => {
                origin_counter_cost = self.linked_counter_permission(actor, source, card, face);
            }
            WebSlingingOrigin::TaxedExile { surcharge } => exile_play_surcharge = Some(surcharge),
            WebSlingingOrigin::PaidResolution => {
                self.cast_during_resolution_configured(
                    actor,
                    self.object_ref(card),
                    face,
                    Some(CastMethod::WebSlinging),
                    Some(cost),
                );
                return;
            }
        }
        let mut item = self.cast_item(actor, card, [0; 6]);
        self.set_cast_face(&mut item, face);
        Self::configure_web_slinging(&mut item);
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        if let Some(surcharge) = exile_play_surcharge {
            Self::configure_taxed_exile_cast(&mut item, surcharge.get());
        }
        if (origin_counter_cost.is_some() || exile_play_surcharge.is_some())
            && let Some(data) = &mut self.state.linked_choices
        {
            data.play_approvals.retain(|row| row.0 != item.id);
        }
        if let Some(kind) = entry_counter {
            self.record_cast_entry_counters(item.source.unwrap().object, kind, 1);
        }
        self.open_payment_announcement_configured(
            item,
            Counters::default(),
            None,
            origin_counter_cost,
            |announcement| {
                announcement.origin_alternative_cost = Some(cost);
                announcement.origin_additional_cost = origin_additional_cost;
                if let Some(life) = origin_life_cost {
                    announcement.add_origin_life_cost(life);
                }
            },
        );
    }

    pub(super) fn configure_web_slinging(item: &mut SpellView) {
        item.cast.as_mut().unwrap().method = CastMethod::WebSlinging;
        if item.source.unwrap().card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }

    pub(super) fn pay_web_slinging_return(&mut self, object: ObjectRef) {
        let mana_value = self.object_mana_value(object.card);
        let announcement = self.state.announcing.as_mut().unwrap();
        let costs = announcement.cast_costs.as_mut().unwrap();
        let index = usize::from(costs.active.take().unwrap());
        costs.clauses[index].clause = CastCostClause::WebSlingingReturn(Some(object));
        costs.clauses[index].paid = true;
        announcement
            .item
            .cast
            .as_mut()
            .unwrap()
            .web_slinging_return_mana_value = Some(mana_value);
        self.state.pending = Pending::CastCosts {
            player: announcement.item.controller,
        };
        self.move_object(object.card, Zone::Hand);
        if self.state.entry_batch.is_none() && self.state.replacements.is_none() {
            self.finish_announcement();
        }
    }
}
