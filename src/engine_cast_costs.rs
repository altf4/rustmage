//! Owned nonmana cost clauses. Mana is committed once before any clause is paid.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CastCostClause {
    Teamwork(u16),
    Blight {
        amount: u16,
        selected: Option<ObjectRef>,
    },
    Casualty(u16),
    Additional,
    Harmonize(ObjectRef),
    Mana,
    Life(u32),
    AlternativeLife(u32),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CastCostRow {
    pub clause: CastCostClause,
    pub paid: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CastCostTransaction {
    pub clauses: Vec<CastCostRow>,
    pub active: Option<u16>,
    pub mana_paid: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub teammates: Vec<ObjectRef>,
    pub mana_closed: bool,
}
impl CastCostTransaction {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.clauses.len() * std::mem::size_of::<CastCostRow>()
            + self.teammates.len() * std::mem::size_of::<ObjectRef>()
    }
}
impl Announcement {
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
    pub(super) fn initialize_alternative_life(&mut self) {
        if self
            .item
            .cast
            .is_some_and(|c| c.method == CastMethod::ExileLife)
        {
            self.add_cast_cost_clause(CastCostClause::AlternativeLife(
                u32::try_from(self.item.mana_value()).expect("spell mana value fits life cost"),
            ));
        }
    }
    pub(super) fn initialize_cast_costs(&mut self, improvise: bool) {
        self.initialize_alternative_life();
        if self.item.kind == StackKind::Spell
            && ((self.item.source.expect("cost announcement has object source").card.has_convoke() || improvise)
                && self.additional_cost().is_some()
                || self
                    .additional_cost()
                    .is_some_and(|s| matches!(s.kind, EffectChoiceKind::DiscardOrLife(_))))
        {
            self.cast_costs.get_or_insert_with(Default::default);
        }
        if self.item.cast.is_some_and(|c| c.blight == Some(true)) {
            self.add_cast_cost_clause(CastCostClause::Blight {
                amount: self.item.source.expect("cost announcement has object source").card.blight().unwrap().amount,
                selected: None,
            });
        }
        if self.item.cast.is_some_and(|c| c.teamwork == Some(true)) {
            self.add_cast_cost_clause(CastCostClause::Teamwork(
                self.item.source.expect("cost announcement has object source").card.teamwork().unwrap().minimum,
            ));
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
        if self.cast_costs.is_some()
            && self.item.kind == StackKind::Spell
            && (self.item.source.expect("cost announcement has object source").card.has_convoke()
                || improvise
                || self
                    .additional_cost()
                    .is_some_and(|s| matches!(s.kind, EffectChoiceKind::DiscardOrLife(_)))
                || self
                    .item
                    .cast
                    .is_some_and(|c| c.method == CastMethod::Harmonize || c.blight == Some(true)))
        {
            self.add_cast_cost_clause(CastCostClause::Mana);
        }
        if self.cast_costs.is_some() && self.additional_cost().is_some() {
            self.add_cast_cost_clause(CastCostClause::Additional);
        }
    }
}
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
                CastCostClause::Additional | CastCostClause::Mana
            ));
            costs.clauses[usize::from(index)].paid = true;
            costs.active = None;
            if clause == CastCostClause::Mana {
                costs.mana_paid = true;
                if let Some(frame) = &a.convoke {
                    self.tap_objects(&frame.objects().iter().map(|s| s.card).collect::<Vec<_>>());
                }
            }
            if clause == CastCostClause::Additional {
                a.additional_paid = true;
                self.pay_additional_cost(a);
                if self.state.entry_batch.is_some() {
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
                self.state.pending = if a.item.source.expect("cost announcement has object source").card.has_convoke() {
                    Pending::Convoke {
                        player: a.item.controller,
                    }
                } else if improvise::potential(self, a.item.source.expect("cost announcement has object source").card) {
                    Pending::Improvise {
                        player: a.item.controller,
                    }
                } else {
                    Pending::PayCosts {
                        player: a.item.controller,
                    }
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::Harmonize(object) => {
                let c = a.cast_costs.as_mut().unwrap();
                c.clauses[usize::from(index)].paid = true;
                c.active = None;
                self.state.announcing = Some(a);
                self.tap_objects(&[object.card]);
                self.finish_announcement();
            }
            CastCostClause::Life(amount) | CastCostClause::AlternativeLife(amount) => {
                self.lose_life(a.item.controller, i128::from(amount));
                let c = a.cast_costs.as_mut().unwrap();
                c.clauses[usize::from(index)].paid = true;
                c.active = None;
                self.state.announcing = Some(a);
                self.finish_announcement();
            }
            CastCostClause::Casualty(instance) => {
                self.state.pending = Pending::CasualtyPayment {
                    player: a.item.controller,
                    instance,
                };
                self.state.announcing = Some(a);
            }
            CastCostClause::Additional => {
                let spec = a.additional_cost().unwrap();
                let p = a.item.controller;
                let selected = a.additional.is_some() || a.additional_life;
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
fn costs_unpaid(a: &Announcement) -> bool {
    a.cast_costs
        .as_ref()
        .is_some_and(|c| c.clauses.iter().any(|r| !r.paid))
}
