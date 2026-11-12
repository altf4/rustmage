//! Alternative costs for independently authorized paid casting routes.
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
pub enum PaidCastOrigin {
    Ordinary,
    LibraryTop {
        permission: LibraryTopCastPermission,
    },
    GraveyardType {
        permission: ObjectRef,
        permanent_type: PermanentType,
    },
    GraveyardFinality,
    GraveyardAdditional,
    GraveyardCounters,
    GraveyardLife {
        life: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entry_counter: Option<CounterKind>,
    },
    SelfGraveyardWithCounter {
        object: ObjectRef,
    },
    LinkedExileCounters {
        source: ObjectRef,
    },
    LinkedExilePaid {
        object: ObjectRef,
        source: ObjectRef,
    },
    GraveyardPaid {
        object: ObjectRef,
        permission: u64,
    },
    TaxedExile {
        surcharge: std::num::NonZeroU32,
    },
    PaidResolution,
}

/// The existing Web-slinging action uses the same independent paid origins.
pub type WebSlingingOrigin = PaidCastOrigin;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PaidCastAlternative {
    WebSlinging(ManaCost),
    Source { provider: ObjectRef, cost: ManaCost },
    Evidence { provider: ObjectRef, threshold: u64 },
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl PaidCastAlternative {
    fn mana_base(self) -> ManaCost {
        match self {
            Self::WebSlinging(cost) | Self::Source { cost, .. } => cost,
            Self::Evidence { .. } => ManaCost::ZERO,
        }
    }

    fn evidence_threshold(self) -> Option<u64> {
        match self {
            Self::Evidence { threshold, .. } => Some(threshold),
            _ => None,
        }
    }

    pub(super) fn configure_announcement(self, announcement: &mut Announcement) {
        announcement.origin_alternative_cost = Some(self.mana_base());
        if let Some(threshold) = self.evidence_threshold() {
            announcement.add_origin_evidence_cost(threshold);
        }
    }

    fn method(self, origin: PaidCastOrigin) -> CastMethod {
        match self {
            Self::WebSlinging(_) => CastMethod::WebSlinging,
            Self::Source { .. } | Self::Evidence { .. } => match origin {
                PaidCastOrigin::LinkedExileCounters { .. } => CastMethod::LinkedExileCounterCost,
                PaidCastOrigin::GraveyardAdditional => CastMethod::GraveyardAdditionalCosts,
                PaidCastOrigin::GraveyardCounters => CastMethod::GraveyardCounterCost,
                _ => CastMethod::Normal,
            },
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Default)]
pub(super) struct PaidCastTerms {
    pub counters: Option<u16>,
    additional: Option<Card>,
    life: Option<u32>,
    entry_counter: Option<CounterKind>,
    pub alternative: cast_permissions::OriginAlternative,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn source_cast_alternatives(&self, actor: Player) -> Vec<PaidCastAlternative> {
        let mut alternatives = Vec::new();
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
                let provider = self.object_ref(id);
                let row = match *fragment {
                    StaticAbilityFragment::SpellAlternativeManaCost(cost) => {
                        PaidCastAlternative::Source { provider, cost }
                    }
                    StaticAbilityFragment::SpellAlternativeEvidenceCost(threshold) => {
                        PaidCastAlternative::Evidence {
                            provider,
                            threshold,
                        }
                    }
                    _ => continue,
                };
                if !alternatives.contains(&row)
                    && self.continuous_fragment_available(id, |found| std::ptr::eq(found, fragment))
                {
                    alternatives.push(row);
                }
            }
        }
        alternatives
    }

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

    pub(super) fn visit_paid_cast_alternatives(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        cast_alternatives: &[PaidCastAlternative],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if !object.is_card() || object.zone == Zone::Battlefield {
            return false;
        }
        for (face, definition) in
            CastFace::choices(object.card).map(|face| (face, face.definition(object.card)))
        {
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
                && self.visit_paid_cast_alternative_face(
                    actor,
                    card,
                    face,
                    sorcery,
                    PaidCastOrigin::Ordinary,
                    cast_alternatives,
                    emit,
                )
            {
                return true;
            }
            if face == CastFace::Normal && object.zone == Zone::Graveyard && object.owner == actor {
                for (eligible, origin) in [
                    (
                        object.card.self_graveyard_cast_cost().is_some(),
                        PaidCastOrigin::GraveyardAdditional,
                    ),
                    (
                        object.card.graveyard_counter_cost().is_some(),
                        PaidCastOrigin::GraveyardCounters,
                    ),
                ] {
                    if eligible
                        && self.visit_paid_cast_alternative_face(
                            actor,
                            card,
                            face,
                            sorcery,
                            origin,
                            cast_alternatives,
                            emit,
                        )
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(super) fn paid_cast_terms(
        &self,
        actor: Player,
        card: CardId,
        face: CastFace,
        origin: PaidCastOrigin,
    ) -> Option<PaidCastTerms> {
        let physical = self.state.objects[card as usize].card;
        let mut terms = PaidCastTerms::default();
        match origin {
            PaidCastOrigin::Ordinary | PaidCastOrigin::GraveyardType { .. } => {}
            PaidCastOrigin::PaidResolution => terms.alternative.exact_paid_resolution_offer = true,
            PaidCastOrigin::LibraryTop { permission } => {
                terms.alternative.library_top = Some(permission);
            }
            PaidCastOrigin::GraveyardFinality => terms.entry_counter = Some(CounterKind::Finality),
            PaidCastOrigin::GraveyardAdditional => terms.additional = Some(physical),
            PaidCastOrigin::GraveyardCounters => terms.counters = physical.graveyard_counter_cost(),
            PaidCastOrigin::GraveyardLife {
                life,
                entry_counter,
            } => {
                terms.life = Some(life);
                terms.entry_counter = entry_counter;
            }
            PaidCastOrigin::SelfGraveyardWithCounter { object }
                if object.card == card && face == CastFace::Normal =>
            {
                terms.entry_counter = Some(self.self_graveyard_cast_entry_counter(actor, object)?);
            }
            PaidCastOrigin::LinkedExileCounters { source } => {
                terms.counters = Some(self.linked_counter_permission(actor, source, card, face)?);
            }
            PaidCastOrigin::LinkedExilePaid { object, source } if object.card == card => {
                terms.alternative.linked_exile_paid =
                    Some(self.linked_paid_permission(actor, source, object, face)?);
            }
            PaidCastOrigin::GraveyardPaid { object, permission } if object.card == card => {
                terms.alternative.paid_graveyard = Some(self.paid_graveyard_permission(
                    actor,
                    object,
                    face,
                    CastMethod::Normal,
                    permission,
                )?);
            }
            PaidCastOrigin::LinkedExilePaid { .. }
            | PaidCastOrigin::GraveyardPaid { .. }
            | PaidCastOrigin::SelfGraveyardWithCounter { .. } => {
                return None;
            }
            PaidCastOrigin::TaxedExile { surcharge } => {
                terms.alternative.exile_play_surcharge = surcharge.get();
            }
        }
        Some(terms)
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "Origin permission and alternative cost are independent"
    )]
    pub(super) fn visit_paid_cast_alternative_face(
        &self,
        actor: Player,
        card: CardId,
        face: CastFace,
        sorcery: bool,
        origin: PaidCastOrigin,
        cast_alternatives: &[PaidCastAlternative],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let physical = self.state.objects[card as usize].card;
        let Some(definition) = face
            .definition(physical)
            .filter(|card| !card.is_land() && !matches!(card.effect(), Effect::Unsupported))
        else {
            return false;
        };
        let web = self.web_slinging_bases(actor, definition);
        if web.is_empty() && cast_alternatives.is_empty() {
            return false;
        }
        let Some(terms) = self.paid_cast_terms(actor, card, face, origin) else {
            return false;
        };
        let choices = web
            .into_iter()
            .map(PaidCastAlternative::WebSlinging)
            .chain(cast_alternatives.iter().copied());
        for choice in choices {
            let method = choice.method(origin);
            let cost = choice.mana_base();
            for flash_surcharge in [false, true] {
                if flash_surcharge
                    && (origin == PaidCastOrigin::PaidResolution
                        || definition.flash_surcharge().is_none())
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
                    terms.counters,
                    terms.additional,
                    terms.life,
                    false,
                    cast_permissions::OriginAlternative {
                        mana_base: Some(cost),
                        evidence_threshold: choice.evidence_threshold(),
                        exact_paid_resolution_offer: terms.alternative.exact_paid_resolution_offer
                            && method != CastMethod::WebSlinging,
                        ..terms.alternative
                    },
                ) && emit(match choice {
                    PaidCastAlternative::WebSlinging(_) => Action::AnnounceWebSlinging {
                        card,
                        face,
                        cost,
                        flash_surcharge,
                        origin,
                    },
                    PaidCastAlternative::Source { provider, .. } => {
                        Action::AnnounceCastWithManaAlternative {
                            object: self.object_ref(card),
                            provider,
                            face,
                            cost,
                            flash_surcharge,
                            origin,
                        }
                    }
                    PaidCastAlternative::Evidence {
                        provider,
                        threshold,
                    } => Action::AnnounceCastWithEvidenceAlternative {
                        object: self.object_ref(card),
                        provider,
                        face,
                        threshold,
                        flash_surcharge,
                        origin,
                    },
                }) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn start_paid_cast_alternative(
        &mut self,
        actor: Player,
        card: CardId,
        face: CastFace,
        choice: PaidCastAlternative,
        flash_surcharge: bool,
        origin: PaidCastOrigin,
    ) {
        let method = choice.method(origin);
        if origin == PaidCastOrigin::PaidResolution {
            self.cast_during_resolution_configured(
                actor,
                self.object_ref(card),
                face,
                Some(method),
                Some(choice),
                |game, item| {
                    if method != CastMethod::WebSlinging {
                        game.configure_paid_resolution_offer(item);
                        Self::set_fixed_alternative_x(item);
                    }
                },
            );
            return;
        }

        let terms = self
            .paid_cast_terms(actor, card, face, origin)
            .expect("selected independent casting permission");
        if let PaidCastOrigin::GraveyardType {
            permission,
            permanent_type,
        } = origin
        {
            self.consume_graveyard_type(permission, permanent_type);
        }
        let mut item = self.cast_item(actor, card, [0; 6]);
        self.set_cast_face(&mut item, face);
        item.cast.as_mut().unwrap().method = method;
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        Self::set_fixed_alternative_x(&mut item);
        if terms.alternative.linked_exile_paid.is_some()
            || method == CastMethod::LinkedExileCounterCost
        {
            self.select_linked_paid_cast(&mut item);
        }
        if let Some(permission) = terms.alternative.paid_graveyard {
            self.apply_paid_graveyard_spell_types(&mut item, permission);
        }
        if terms.alternative.exile_play_surcharge != 0 {
            Self::configure_taxed_exile_cast(&mut item, terms.alternative.exile_play_surcharge);
        }
        if (terms.counters.is_some() || terms.alternative.exile_play_surcharge != 0)
            && let Some(data) = &mut self.state.linked_choices
        {
            data.play_approvals.retain(|row| row.0 != item.id);
        }
        if origin == PaidCastOrigin::Ordinary
            && !matches!(choice, PaidCastAlternative::WebSlinging(_))
            && item.cast.unwrap().origin == Zone::Exile
        {
            self.select_ordinary_paid_exile_cast(item.id);
        }
        if let Some(kind) = terms.entry_counter {
            self.record_cast_entry_counters(item.source.unwrap().object, kind, 1);
        }
        self.open_payment_announcement_configured(
            item,
            Counters::default(),
            None,
            terms.counters,
            |announcement| {
                choice.configure_announcement(announcement);
                if let Some(permission) = terms.alternative.library_top {
                    announcement.limited_cast_permission =
                        Some(Box::new(LimitedCastPermission::LibraryTop(permission)));
                }
                if let Some(permission) = terms.alternative.linked_exile_paid {
                    announcement.limited_cast_permission =
                        Some(Box::new(LimitedCastPermission::LinkedExilePaid(permission)));
                }
                if let Some(permission) = terms.alternative.paid_graveyard {
                    announcement.set_paid_graveyard_cast_permission(permission);
                }
                announcement.origin_additional_cost = terms.additional;
                if let Some(life) = terms.life {
                    announcement.add_origin_life_cost(life);
                }
            },
        );
    }

    pub(super) fn set_fixed_alternative_x(item: &mut SpellView) {
        if item.source.unwrap().card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }

    pub(super) fn configure_web_slinging(item: &mut SpellView) {
        item.cast.as_mut().unwrap().method = CastMethod::WebSlinging;
        Self::set_fixed_alternative_x(item);
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
