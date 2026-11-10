//! Owned nonmana cost clauses. Mana is committed once before any clause is paid.
use super::*;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CastCostClause {
    Teamwork(u16),
    Blight {
        amount: u16,
        selected: Option<ObjectRef>,
    },
    Casualty(u16),
    Conspire(u16),
    Additional,
    OriginCounters(u16),
    SneakReturn(Option<ObjectRef>),
    WebSlingingReturn(Option<ObjectRef>),
    /// Exile the activated craft source as its own ordered cost component.
    CraftSource,
    Harmonize(ObjectRef),
    Behold(&'static str),
    Mana,
    Life(u32),
    AlternativeLife(u32),
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CastCostRow {
    pub clause: CastCostClause,
    pub paid: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CastCostTransaction {
    pub clauses: Vec<CastCostRow>,
    pub active: Option<u16>,
    pub mana_paid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub teammates: Vec<ObjectRef>,
    pub mana_closed: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CastCostTransaction {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.clauses.len() * std::mem::size_of::<CastCostRow>()
            + self.teammates.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    /// The initial craft consumers have exactly a mana cost, source exile, and
    /// one material cost. Other activation shapes retain their existing path.
    pub(super) fn ordered_craft(&self) -> bool {
        let StackKind::ActivatedAbility { ability } = self.item.kind else {
            return false;
        };
        let Some(source) = self.item.source else {
            return false;
        };
        let definition = source.card.activation(ability);
        definition.cost.exile_self
            && definition.cost.additional.is_some_and(|cost| {
                matches!(
                    cost.kind,
                    EffectChoiceKind::ExileControlledOrGraveyard
                        | EffectChoiceKind::ExileCraftNonlands(_)
                )
            })
            && definition
                .program
                .contains(&Instruction::ReturnCraftedSourceTransformed)
    }
    pub(super) fn craft_source_paid(&self) -> bool {
        self.cast_costs.as_ref().is_some_and(|costs| {
            costs
                .clauses
                .iter()
                .any(|row| row.clause == CastCostClause::CraftSource && row.paid)
        })
    }
    pub(super) fn add_cast_cost_clause(&mut self, clause: CastCostClause) {
        let costs = self.cast_costs.get_or_insert_with(Default::default);
        assert!(!costs.mana_closed, "additional cost declared after payment");
        if !costs.clauses.iter().any(|r| r.clause == clause) {
            costs.clauses.push(CastCostRow {
                clause,
                paid: false,
            });
        }
    }
    pub(super) fn add_origin_life_cost(&mut self, life: u32) {
        if life != 0 {
            self.add_cast_cost_clause(CastCostClause::Mana);
            self.add_cast_cost_clause(CastCostClause::Life(life));
        }
    }
    pub(super) fn has_ordered_life_cost(&self) -> bool {
        self.cast_costs.as_ref().is_some_and(|costs| {
            costs
                .clauses
                .iter()
                .any(|row| matches!(row.clause, CastCostClause::Life(_)))
        })
    }
    pub(super) fn initialize_alternative_life(&mut self) {
        if let Some((life, _)) = self.self_graveyard_cast_cost() {
            self.add_cast_cost_clause(CastCostClause::Mana);
            if life != 0 {
                self.add_cast_cost_clause(CastCostClause::Life(life));
            }
        }
        if self
            .item
            .cast
            .is_some_and(|c| c.method.pays_life_for_mana_value())
        {
            self.add_cast_cost_clause(CastCostClause::AlternativeLife(
                u32::try_from(self.item.mana_value()).expect("spell mana value fits life cost"),
            ));
        }
    }
    pub(super) fn initialize_cast_costs(&mut self, improvise: bool) {
        self.initialize_alternative_life();
        if let Some(amount) = self.linked_origin_counter_cost() {
            self.add_cast_cost_clause(CastCostClause::Mana);
            self.add_cast_cost_clause(CastCostClause::OriginCounters(amount));
        }
        if self.has_compound_sneak_cost() {
            self.add_cast_cost_clause(CastCostClause::Mana);
            self.add_cast_cost_clause(CastCostClause::SneakReturn(None));
        }
        if self
            .item
            .cast
            .is_some_and(|cast| cast.method == CastMethod::WebSlinging)
        {
            self.add_cast_cost_clause(CastCostClause::Mana);
            self.add_cast_cost_clause(CastCostClause::WebSlingingReturn(None));
        }
        if self.additional_cost().is_some_and(|s| {
            s.kind == EffectChoiceKind::ReturnTappedCreature
                || matches!(
                    s.kind.payment_kind(),
                    EffectChoiceKind::BeholdMany(_)
                        | EffectChoiceKind::BeholdAndExile
                        | EffectChoiceKind::ChooseControlledOrWarpedCreature
                        | EffectChoiceKind::BlightCost(_)
                )
        }) {
            self.add_cast_cost_clause(CastCostClause::Mana);
        }
        if self.nonmana_x() {
            self.add_cast_cost_clause(CastCostClause::Mana);
        }
        if self.item.cast.is_some_and(|cast| cast.beheld == Some(true)) {
            let subtype = self
                .item
                .source
                .expect("cast source")
                .card
                .behold_cast()
                .expect("behold recipe")
                .subtype;
            self.add_cast_cost_clause(CastCostClause::Mana);
            self.add_cast_cost_clause(CastCostClause::Behold(subtype));
        }
        if self.item.kind == StackKind::Spell
            && ((self
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .has_convoke()
                || improvise)
                && self.additional_cost().is_some()
                || self
                    .additional_cost()
                    .is_some_and(|s| matches!(s.kind, EffectChoiceKind::DiscardOrLife(_))))
        {
            self.cast_costs.get_or_insert_with(Default::default);
        }
        if self.item.cast.is_some_and(|c| c.blight == Some(true)) {
            self.add_cast_cost_clause(CastCostClause::Blight {
                amount: self
                    .item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .blight()
                    .unwrap()
                    .amount,
                selected: None,
            });
        }
        if self.item.cast.is_some_and(|c| c.teamwork == Some(true)) {
            self.add_cast_cost_clause(CastCostClause::Teamwork(
                self.item
                    .source
                    .expect("cost announcement has object source")
                    .card
                    .teamwork()
                    .unwrap()
                    .minimum,
            ));
        }
        let conspire: Vec<_> = self
            .conspire
            .as_deref()
            .into_iter()
            .flatten()
            .enumerate()
            .filter(|(_, r)| r.pay)
            .map(|(i, _)| CastCostClause::Conspire(u16::try_from(i).expect("conspire index")))
            .collect();
        for clause in conspire {
            self.add_cast_cost_clause(clause);
            self.add_cast_cost_clause(CastCostClause::Mana);
        }
        let casualty: Vec<_> = self
            .casualty
            .as_deref()
            .into_iter()
            .flatten()
            .enumerate()
            .filter(|(_, r)| r.pay)
            .map(|(i, _)| CastCostClause::Casualty(u16::try_from(i).expect("casualty index")))
            .collect();
        for clause in casualty {
            self.add_cast_cost_clause(clause);
        }
        if self.ordered_craft() {
            self.add_cast_cost_clause(CastCostClause::Mana);
            self.add_cast_cost_clause(CastCostClause::CraftSource);
        }
        if self.cast_costs.is_some()
            && self.item.kind == StackKind::Spell
            && (self
                .item
                .source
                .expect("cost announcement has object source")
                .card
                .has_convoke()
                || improvise
                || self
                    .additional_cost()
                    .is_some_and(|s| matches!(s.kind, EffectChoiceKind::DiscardOrLife(_)))
                || self
                    .item
                    .cast
                    .is_some_and(|c| c.method.is_harmonize() || c.blight == Some(true)))
        {
            self.add_cast_cost_clause(CastCostClause::Mana);
        }
        if self.cast_costs.is_some() && self.additional_cost().is_some() {
            self.add_cast_cost_clause(CastCostClause::Additional);
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    // Compatibility name: callers ask whether the mana-ability window is closed.
    pub(super) fn cast_costs_mana_paid(&self) -> bool {
        self.state
            .announcing
            .as_ref()
            .and_then(|a| a.cast_costs.as_ref())
            .is_some_and(|c| c.mana_closed)
    }
    pub(super) fn resume_cast_cost_selection(&mut self) {
        if self.cast_costs_mana_paid() && matches!(self.state.pending, Pending::PayCosts { .. }) {
            self.finish_announcement();
        }
    }
    pub(super) fn continue_cast_cost_payment(&mut self, a: &mut Announcement) -> bool {
        let Some(costs) = &mut a.cast_costs else {
            return false;
        };
        costs.mana_closed = true;
        if !costs
            .clauses
            .iter()
            .any(|r| r.clause == CastCostClause::Mana && !r.paid)
        {
            costs.mana_paid = true;
        }
        if let Some(index) = costs.active {
            let clause = costs.clauses[usize::from(index)].clause;
            assert!(matches!(
                clause,
                CastCostClause::Additional
                    | CastCostClause::Mana
                    | CastCostClause::OriginCounters(_)
            ));
            costs.clauses[usize::from(index)].paid = true;
            costs.active = None;
            if clause == CastCostClause::Mana {
                costs.mana_paid = true;
                self.pay_mana_contributors(a);
            }
            if let CastCostClause::OriginCounters(amount) = clause {
                let selected = a.creature_counter_cost.take().unwrap_or_default();
                assert_eq!(selected.len(), usize::from(amount));
                self.pay_selected_creature_counters(&mut a.item, &selected);
                a.additional = None;
            }
            if clause == CastCostClause::Additional {
                a.additional_paid = true;
                self.pay_additional_cost(a);
                if self.state.entry_batch.is_some() || self.state.replacements.is_some() {
                    return true;
                }
            }
        }
        if costs_unpaid(a) {
            self.state.pending = Pending::CastCosts {
                player: a.item.controller,
            };
            return true;
        }
        false
    }
    pub(super) fn begin_ordered_costs(&mut self) {
        if let Pending::AdditionalCost { player } = self.state.pending {
            self.open_counter_cost_mana(player);
            return;
        }
        let a = self.state.announcing.as_mut().unwrap();
        a.cast_costs.as_mut().unwrap().mana_closed = true;
        self.state.pending = Pending::CastCosts {
            player: a.item.controller,
        };
    }
    pub(super) fn choose_cast_cost(&mut self, index: u16) {
        let mut a = self.state.announcing.take().unwrap();
        let kind = a.cast_costs.as_ref().unwrap().clauses[usize::from(index)].clause;
        a.cast_costs.as_mut().unwrap().active = Some(index);
        match kind {
            CastCostClause::Blight { .. } => {
                self.state.pending = Pending::BlightPayment {
                    player: a.item.controller,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::Teamwork(_) => {
                self.state.pending = Pending::TeamworkPayment {
                    player: a.item.controller,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::Mana => {
                let delving = self.item_delve(&a.item);
                let contributors = a.item.kind == StackKind::Spell
                    && (self.item_convoke(&a.item) || self.item_improvise(&a.item) || delving);
                if contributors && a.convoke.is_none() {
                    a.convoke = Some(Box::new(convoke::ConvokeSelection {
                        remaining: a.total_cost.expect("locked mana cost"),
                        selected: Vec::new(),
                        improvised: Vec::new(),
                        delved: Vec::new(),
                    }));
                }
                self.state.pending = if self.item_convoke(&a.item) {
                    Pending::Convoke {
                        player: a.item.controller,
                    }
                } else if improvise::potential(
                    self,
                    a.item
                        .source
                        .expect("cost announcement has object source")
                        .card,
                ) {
                    Pending::Improvise {
                        player: a.item.controller,
                    }
                } else if delving {
                    Pending::Delve {
                        player: a.item.controller,
                    }
                } else {
                    Pending::PayCosts {
                        player: a.item.controller,
                    }
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::CraftSource => {
                let source = a.item.source.expect("craft has source");
                let view = self.object_view(source.id);
                a.item.source.as_mut().unwrap().facts = DamageFacts::from_view(&view);
                a.item.source_keywords = view.keywords;
                a.item.source_tapped = view.tapped;
                a.item.source_attachment = view.attached_to;
                let event = self.move_object(source.id, Zone::Exile);
                self.craft_exile_triggers(&event, a.item.controller);
                a.item.source_move = Some(Box::new(event.source_movement()));
                let costs = a.cast_costs.as_mut().unwrap();
                costs.clauses[usize::from(index)].paid = true;
                costs.active = None;
                self.state.announcing = Some(a);
                self.finish_announcement();
            }
            CastCostClause::Harmonize(object) => {
                let c = a.cast_costs.as_mut().unwrap();
                c.clauses[usize::from(index)].paid = true;
                c.active = None;
                self.state.announcing = Some(a);
                self.tap_objects(&[object.card]);
                self.finish_announcement();
            }
            CastCostClause::Behold(_) => {
                self.state.pending = Pending::BeholdCost {
                    player: a.item.controller,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::Life(amount) | CastCostClause::AlternativeLife(amount) => {
                self.pay_life(a.item.controller, i128::from(amount));
                let c = a.cast_costs.as_mut().unwrap();
                c.clauses[usize::from(index)].paid = true;
                c.active = None;
                self.state.announcing = Some(a);
                self.finish_announcement();
            }
            CastCostClause::Conspire(instance) => {
                self.state.pending = Pending::ConspirePayment {
                    player: a.item.controller,
                    instance,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::Casualty(instance) => {
                self.state.pending = Pending::CasualtyPayment {
                    player: a.item.controller,
                    instance,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::SneakReturn(_) => {
                self.state.pending = Pending::SneakReturnCost {
                    player: a.item.controller,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::WebSlingingReturn(_) => {
                self.state.pending = Pending::WebSlingingReturnCost {
                    player: a.item.controller,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::OriginCounters(amount) => {
                a.additional = None;
                a.creature_counter_cost = Some(Box::new(Vec::new()));
                let player = a.item.controller;
                if amount == 0 {
                    a.additional = Some(a.item.source.unwrap().object);
                    self.state.pending = Pending::PayCosts { player };
                    self.state.announcing = Some(a);
                    self.resume_cast_cost_selection();
                } else {
                    self.state.pending = Pending::AdditionalCost { player };
                    self.state.announcing = Some(a);
                }
            }
            CastCostClause::Additional => {
                let spec = a.additional_cost().unwrap();
                let p = a.item.controller;
                let selected =
                    a.additional.is_some() || a.additional_group_complete || a.additional_life;
                self.state.announcing = Some(a);
                if selected {
                    self.state.pending = Pending::PayCosts { player: p };
                    self.finish_announcement();
                } else if evidence::threshold(spec).is_some() {
                    self.start_announcement_evidence();
                } else {
                    self.state.pending = if forage::is_forage_cost(spec) {
                        Pending::Forage { player: p }
                    } else {
                        Pending::AdditionalCost { player: p }
                    };
                }
            }
        }
    }
    pub(super) fn pay_conspire_clause(&mut self, objects: [ObjectRef; 2]) {
        let Pending::ConspirePayment { instance, .. } = self.state.pending else {
            unreachable!()
        };
        let a = self.state.announcing.as_mut().unwrap();
        let row = &mut a.conspire.as_mut().unwrap()[usize::from(instance)];
        row.objects = Some(objects);
        row.paid = true;
        let costs = a.cast_costs.as_mut().unwrap();
        let active = costs.active.take().unwrap();
        costs.clauses[usize::from(active)].paid = true;
        self.tap_objects(&objects.map(|r| r.card));
        self.finish_announcement();
    }
    pub(super) fn pay_casualty_clause(&mut self, object: ObjectRef) {
        let instance = match self.state.pending {
            Pending::CasualtyPayment { instance, .. } => instance,
            _ => unreachable!(),
        };
        let a = self.state.announcing.as_mut().unwrap();
        let row = &mut a.casualty.as_mut().unwrap()[usize::from(instance)];
        row.sacrifice = Some(object);
        row.paid = true;
        let costs = a.cast_costs.as_mut().unwrap();
        let active = costs.active.take().unwrap();
        costs.clauses[usize::from(active)].paid = true;
        let player = a.item.controller;
        self.sacrifice_batch(&[(player, object.card)]);
        if self.state.entry_batch.is_none() {
            self.finish_announcement();
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn costs_unpaid(a: &Announcement) -> bool {
    a.cast_costs
        .as_ref()
        .is_some_and(|c| c.clauses.iter().any(|r| !r.paid))
}
