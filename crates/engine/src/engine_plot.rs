//! Plot and foretell are paid special actions with independent later casts.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlottedCard {
    pub object: ObjectRef,
    pub owner: Player,
    pub plotted_turn: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PlotPaymentView {
    pub object: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
    pub cost: ManaCost,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub from_library: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub foretell: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_symbols: Option<std::num::NonZeroU32>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl PlotPaymentView {
    pub(super) fn goal(self) -> FundingGoal {
        FundingGoal {
            actor: self.controller,
            required: self.cost.colored,
            generic: self.cost.generic,
            reserve: None,
            source: None,
        }
    }

    pub(super) fn source_is_current<W: ability_world::CharacteristicWorld + ?Sized>(
        self,
        world: &W,
    ) -> bool {
        let object = world.object(self.object.card);
        let source_zone = if self.from_library {
            Zone::Library
        } else {
            Zone::Hand
        };
        world.reference(self.object.card) == self.object
            && object.zone == source_zone
            && object.owner == self.controller
            && object.is_card()
    }

    fn choose_x(mut self, x: u32) -> Self {
        let symbols = self.x_symbols.take().expect("variable Plot cost").get();
        self.cost.generic = self
            .cost
            .generic
            .checked_add(symbols.checked_mul(x).expect("legal Plot X"))
            .expect("legal Plot total");
        self
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ForetoldCardView {
    pub serial: u64,
    pub owner: Player,
    pub foretold_turn: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<ObjectRef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
struct ForetoldCard {
    serial: u64,
    object: ObjectRef,
    owner: Player,
    foretold_turn: u32,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlotState {
    pub cards: Vec<PlottedCard>,
    foretold: Vec<ForetoldCard>,
    pub payment: Option<PlotPaymentView>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl PlotState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.cards.len() * std::mem::size_of::<PlottedCard>()
            + self.foretold.len() * std::mem::size_of::<ForetoldCard>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn reduced_plot_cost(&self, actor: Player, cost: ManaCost) -> ManaCost {
        let reduction = self.state.static_sources.iter().copied().filter(|&id| {
            self.on_battlefield(id) && self.state.objects[id as usize].controller == actor
        }).flat_map(|id| {
            self.state.objects[id as usize].card.static_ability_groups().iter().flat_map(move |group| {
                group.fragments.iter().filter_map(move |fragment| match *fragment {
                    crate::cards::StaticAbilityFragment::PlotCostReduction(amount)
                        if self.continuous_fragment_available(id, |candidate| matches!(candidate, crate::cards::StaticAbilityFragment::PlotCostReduction(n) if *n == amount)) => Some(amount),
                    _ => None,
                })
            })
        }).fold(0u32, u32::saturating_add);
        ManaCost {
            generic: cost.generic.saturating_sub(reduction),
            ..cost
        }
    }
    pub(super) fn plot_payment(&self) -> Option<&PlotPaymentView> {
        self.state.plot.as_ref().and_then(|p| p.payment.as_ref())
    }
    pub(super) fn plot_can_fund(&self, actor: Player, cost: ManaCost) -> bool {
        self.can_fund(FundingGoal {
            actor,
            required: cost.colored,
            generic: cost.generic,
            reserve: None,
            source: None,
        })
    }
    pub(super) fn begin_plot(&mut self, actor: Player, card: CardId) {
        self.start_plot_payment(PlotPaymentView {
            object: self.object_ref(card),
            source_card: self.state.objects[card as usize].card,
            controller: actor,
            cost: self.reduced_plot_cost(
                actor,
                self.state.objects[card as usize].card.plot_cost().unwrap(),
            ),
            from_library: false,
            foretell: false,
            x_symbols: None,
        });
    }

    pub(super) fn can_plot_library_top(&self, actor: Player) -> bool {
        self.state.static_sources.iter().copied().any(|source| {
            self.state.objects[source as usize].controller == actor
                && self.active_rule_fragment(source, |fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::PlotFromLibraryTop
                    )
                })
        })
    }

    pub(super) fn visit_library_top_plots(
        &self,
        actor: Player,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Some(id) = self.top_library_card(actor) else {
            return false;
        };
        let source = &self.state.objects[id as usize];
        if !source.is_card() || source.card.is_land() || !self.can_plot_library_top(actor) {
            return false;
        }
        let object = self.object_ref(id);
        let mut visit = |cost, x_symbols| {
            self.plot_payment_fundable(PlotPaymentView {
                object,
                source_card: source.card,
                controller: actor,
                cost,
                from_library: true,
                foretell: false,
                x_symbols,
            }) && emit(Action::AnnounceLibraryTopPlot {
                object,
                cost,
                x_symbols,
            })
        };
        let intrinsic = source.card.plot_cost();
        if intrinsic.is_some_and(|cost| visit(cost, None)) {
            return true;
        }
        let mana_cost = source.card.info().mana_cost;
        // The inset Adventure or Omen is not a characteristic of the library card.
        let mana_cost =
            if source.card.adventure_spell().is_some() || source.card.omen_spell().is_some() {
                mana_cost
                    .split_once(" // ")
                    .map_or(mana_cost, |(front, _)| front)
            } else {
                mana_cost
            };
        let Some(printed) = crate::cards::mana_symbols::printed_cost(mana_cost) else {
            // A nonexistent mana cost is unpayable, unlike an explicit {0}.
            return false;
        };
        let x_symbols = std::num::NonZeroU32::new(printed.x_symbols);
        printed
            .options
            .into_iter()
            .any(|cost| (x_symbols.is_some() || intrinsic != Some(cost)) && visit(cost, x_symbols))
    }

    pub(super) fn begin_library_top_plot(
        &mut self,
        actor: Player,
        object: ObjectRef,
        cost: ManaCost,
        x_symbols: Option<std::num::NonZeroU32>,
    ) {
        self.start_plot_payment(PlotPaymentView {
            object,
            source_card: self.state.objects[object.card as usize].card,
            controller: actor,
            cost,
            from_library: true,
            foretell: false,
            x_symbols,
        });
    }

    fn start_plot_payment(&mut self, payment: PlotPaymentView) {
        let player = payment.controller;
        self.state.plot.get_or_insert_with(Default::default).payment = Some(payment);
        self.state.pending = if payment.x_symbols.is_some() {
            Pending::PlotX { player }
        } else {
            Pending::PlotPayment { player }
        };
        self.state.passes = 0;
    }

    pub(super) fn plot_x_choice(&self) -> Option<XChoiceView> {
        let request = *self.plot_payment()?;
        let symbols = request.x_symbols?.get();
        let mut minimum = 0;
        let mut maximum = (u32::MAX - request.cost.generic) / symbols;
        while minimum < maximum {
            let midpoint = minimum + (maximum - minimum).div_ceil(2);
            if self.plot_payment_fundable(request.choose_x(midpoint)) {
                minimum = midpoint;
            } else {
                maximum = midpoint - 1;
            }
        }
        Some(XChoiceView {
            card: request.object,
            intervals: vec![XInterval {
                minimum: 0,
                maximum,
            }],
        })
    }

    pub(super) fn choose_plot_x(&mut self, value: u32) {
        let request = self.state.plot.as_mut().unwrap().payment.as_mut().unwrap();
        *request = request.choose_x(value);
        self.state.pending = Pending::PlotPayment {
            player: request.controller,
        };
    }

    pub(super) fn finish_plot(&mut self, payment: Mana) {
        let pending = self.state.plot.as_mut().unwrap().payment.take().unwrap();
        self.spend_mana(pending.controller, payment);
        if pending.foretell {
            self.foretell(pending);
        } else {
            let event = self.move_object(pending.object.card, Zone::Exile);
            self.commit_plotted_move(&event);
        }
        self.clear_empty_plot_state();
        self.state.pending = Pending::Priority;
        self.state.priority = pending.controller;
        self.state.passes = 0;
    }
    pub(super) fn exile_target_spell_plotted(&mut self, item: &SpellView) {
        let source = item.targets.iter().find_map(|target| {
            let spell = self.bound_stack_item(target)?;
            spell.source.filter(|source| {
                spell.kind == StackKind::Spell
                    && self.state.objects[source.id as usize].zone == Zone::Stack
                    && self.object_ref(source.id) == source.object
            })
        });
        if let Some(source) = source {
            let event = self.move_object(source.id, Zone::Exile);
            self.commit_plotted_move(&event);
        }
    }

    pub(super) fn grant_cast_plot_on_resolution(&mut self, item: &SpellView) {
        let Some(spell) = self.live_cast_spell(item) else {
            return;
        };
        if !self.state.objects[spell.card as usize].is_card() {
            return;
        }
        let marks = &mut self.state.zone_rules.get_or_insert_default().plot_spells;
        if !marks.contains(&spell) {
            marks.push(spell);
        }
    }

    pub(super) fn commit_plotted_move(&mut self, event: &ZoneChange) {
        if event.moved && event.destination == Zone::Exile && event.was_card {
            self.remember_plotted(event.before.owner, event.after);
        }
    }

    pub(super) fn remember_plotted(&mut self, owner: Player, object: ObjectRef) {
        let card = &self.state.objects[object.card as usize];
        if self.object_ref(object.card) != object
            || card.zone != Zone::Exile
            || !card.is_card()
            || card.owner != owner
        {
            return;
        }
        let cards = &mut self.state.plot.get_or_insert_with(Default::default).cards;
        let plotted = PlottedCard {
            object,
            owner,
            plotted_turn: self.state.turn,
        };
        match cards.binary_search_by_key(&object.card, |entry| entry.object.card) {
            Ok(index) if cards[index].object == object => return,
            Ok(index) => cards[index] = plotted,
            Err(index) => {
                cards
                    .try_reserve(1)
                    .capacity("plotted permission allocation exhausted");
                cards.insert(index, plotted);
            }
        }
        let source = self.object_view(object.card);
        for (index, ability) in source.trigger_templates() {
            if ability.condition.event() == TriggerCondition::SelfBecomesPlotted {
                self.queue_trigger(&source, index);
            }
        }
    }
    pub(super) fn remove_plot_and_foretell(&mut self, object: ObjectRef) {
        if let Some(state) = &mut self.state.plot {
            state.cards.retain(|card| card.object != object);
            state.foretold.retain(|card| card.object != object);
        }
        self.clear_empty_plot_state();
    }
    fn clear_empty_plot_state(&mut self) {
        if self.state.plot.as_ref().is_some_and(|state| {
            state.cards.is_empty() && state.foretold.is_empty() && state.payment.is_none()
        }) {
            self.state.plot = None;
        }
    }

    fn foretell_request(&self, actor: Player, object: ObjectRef) -> PlotPaymentView {
        PlotPaymentView {
            object,
            source_card: self.state.objects[object.card as usize].card,
            controller: actor,
            cost: ManaCost {
                colored: [0; 6],
                generic: 2,
            },
            from_library: false,
            foretell: true,
            x_symbols: None,
        }
    }

    pub(super) fn visit_foretell(
        &self,
        actor: Player,
        card: CardId,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let candidate = &self.state.objects[card as usize];
        if actor != self.state.active
            || candidate.zone != Zone::Hand
            || candidate.owner != actor
            || !candidate.is_card()
            || candidate.card.foretell_cost().is_none()
        {
            return false;
        }
        let object = self.object_ref(card);
        self.plot_payment_fundable(self.foretell_request(actor, object))
            && emit(Action::AnnounceForetell { object })
    }

    pub(super) fn begin_foretell(&mut self, actor: Player, object: ObjectRef) {
        self.start_plot_payment(self.foretell_request(actor, object));
    }

    fn foretell(&mut self, request: PlotPaymentView) {
        self.obscure_private_hand_selection_matching(request.controller, |card| {
            card.foretell_cost().is_some()
        });
        let event = self.move_object_raw(request.object.card, Zone::Exile);
        if event.moved && event.destination == Zone::Exile && event.was_card {
            self.remember_face_down_exile(event.after, 1 << request.controller.index(), None);
            let serial = self.next_role_timestamp();
            self.state
                .plot
                .as_mut()
                .unwrap()
                .foretold
                .push(ForetoldCard {
                    serial,
                    object: event.after,
                    owner: request.controller,
                    foretold_turn: self.state.turn,
                });
        }
        self.finish_zone_events(std::slice::from_ref(&event), &[]);
    }

    pub(super) fn is_foretold(&self, object: ObjectRef) -> bool {
        self.state
            .plot
            .as_ref()
            .is_some_and(|state| state.foretold.iter().any(|card| card.object == object))
    }

    pub(super) fn foretold_cards(&self, viewer: Player) -> Vec<ForetoldCardView> {
        self.state.plot.as_ref().map_or_else(Vec::new, |state| {
            state
                .foretold
                .iter()
                .map(|card| ForetoldCardView {
                    serial: card.serial,
                    owner: card.owner,
                    foretold_turn: card.foretold_turn,
                    object: self
                        .can_view_hideaway(viewer, card.object)
                        .then_some(card.object),
                })
                .collect()
        })
    }

    pub(super) fn visit_foretold_casts(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let Some(state) = &self.state.plot else {
            return false;
        };
        let object = self.object_ref(card);
        if !state.foretold.iter().any(|row| {
            row.object == object && row.owner == actor && self.state.turn > row.foretold_turn
        }) {
            return false;
        }
        CastFace::choices(self.state.objects[card as usize].card).any(|face| {
            [false, true].into_iter().any(|flash_surcharge| {
                self.permitted_cast_available(
                    actor,
                    card,
                    CastMethod::Foretell,
                    face,
                    sorcery,
                    flash_surcharge,
                ) && emit(Action::AnnounceForetold {
                    object,
                    face,
                    flash_surcharge,
                })
            })
        })
    }

    pub(super) fn configure_foretold_cast(item: &mut SpellView, flash_surcharge: bool) {
        let cast = item.cast.as_mut().expect("foretold spell");
        cast.method = CastMethod::Foretell;
        cast.flash_surcharge = flash_surcharge;
        cast.spend_any_mana = false;
        cast.exile_on_graveyard = false;
        cast.stash_origin = false;
        if item.source.expect("foretold spell source").card.x_symbols() > 0 {
            item.x = Some(0);
        }
    }
    pub(super) fn has_plotted_permission(&self, actor: Player, card: CardId) -> bool {
        let Some(state) = &self.state.plot else {
            return false;
        };
        let Ok(i) = state.cards.binary_search_by_key(&card, |p| p.object.card) else {
            return false;
        };
        let plotted = state.cards[i];
        let object = &self.state.objects[card as usize];
        object.is_card()
            && object.card.is_supported()
            && plotted.owner == actor
            && plotted.object == self.object_ref(card)
            && object.zone == Zone::Exile
            && object.owner == actor
            && self.state.turn > plotted.plotted_turn
    }

    pub(super) fn visit_plotted_casts(
        &self,
        actor: Player,
        card: CardId,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        CastFace::choices(self.state.objects[card as usize].card).any(|face| {
            self.permitted_cast_available(actor, card, CastMethod::Plot, face, true, false)
                && emit(if face == CastFace::Normal {
                    Action::AnnouncePlotted { card }
                } else {
                    Action::AnnouncePlottedFace { card, face }
                })
        })
    }
}
