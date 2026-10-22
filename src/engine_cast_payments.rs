//! Actual cast payments and complete coupled numeric allocation domains.
use super::*;

/// Every six-vector `payment` satisfying these constraints is a legal PayCosts
/// allocation: required[i] <= payment[i] <= available[i], and the sum of
/// payment[i] - required[i] is exactly generic. Colorless is component five.
/// Numeric representatives in legal_actions do not restrict this complete space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ManaPaymentDomain {
    pub available: Mana,
    pub required: Mana,
    pub generic: u32,
}
impl ManaPaymentDomain {
    pub fn contains(&self, payment: Mana) -> bool {
        if (0..6).any(|i| payment[i] < self.required[i] || payment[i] > self.available[i]) {
            return false;
        }
        payment
            .into_iter()
            .zip(self.required)
            .map(|(p, r)| u64::from(p - r))
            .sum::<u64>()
            == u64::from(self.generic)
    }
    /// Exhaustive for spaces up to 64 points, bounded for larger numeric spaces.
    pub(super) fn representatives(&self, mut emit: impl FnMut(Mana)) {
        fn visit(
            d: &ManaPaymentDomain,
            i: usize,
            left: u32,
            mut paid: Mana,
            count: &mut usize,
            emit: &mut impl FnMut(Mana),
        ) {
            if *count == 64 {
                return;
            }
            if i == 5 {
                if d.available[i] - d.required[i] >= left {
                    paid[i] += left;
                    *count += 1;
                    emit(paid);
                }
                return;
            }
            let spare = d.available[i] - d.required[i];
            // Prune impossible branches before iterating. Work is bounded by
            // six components and the number of emitted representatives.
            let remaining_capacity: u64 = (i + 1..6)
                .map(|j| u64::from(d.available[j] - d.required[j]))
                .sum();
            let minimum = u64::from(left).saturating_sub(remaining_capacity) as u32;
            for n in minimum..=spare.min(left) {
                paid[i] = d.required[i] + n;
                visit(d, i + 1, left - n, paid, count, emit);
                if *count == 64 {
                    break;
                }
            }
        }
        if (0..6).any(|i| self.available[i] < self.required[i]) {
            return;
        }
        visit(self, 0, self.generic, self.required, &mut 0, &mut emit);
    }
}
impl CastInfo {
    pub const fn stash_origin(self) -> bool {
        self.origin_permissions.stash_origin()
    }
    pub const fn exile_any_mana(self) -> bool {
        self.origin_permissions.exile_any_mana()
    }

    pub fn mana_spent(self) -> u64 {
        self.payment.into_iter().map(u64::from).sum()
    }
    pub fn colors_spent(self) -> u8 {
        (0..5).filter(|&i| self.payment[i] > 0).count() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coupled_domain_accepts_every_exact_allocation_not_only_representatives() {
        let d = ManaPaymentDomain {
            available: [u32::MAX; 6],
            required: [0; 6],
            generic: u32::MAX,
        };
        let mut choices = Vec::new();
        d.representatives(|p| choices.push(p));
        assert_eq!(choices.len(), 64);
        assert!(choices.iter().all(|p| d.contains(*p)));
        let novel = [1, 2, 3, 4, 5, u32::MAX - 15];
        assert!(d.contains(novel));
        assert!(!choices.contains(&novel));
        assert!(!d.contains([u32::MAX; 6]));
        let d = ManaPaymentDomain {
            available: [u32::MAX; 6],
            required: [u32::MAX; 6],
            generic: 0,
        };
        assert!(d.contains([u32::MAX; 6]));
        let mut choices = Vec::new();
        d.representatives(|p| choices.push(p));
        assert_eq!(choices, vec![[u32::MAX; 6]]);
    }
    #[test]
    fn bounded_domain_matches_exhaustive_old_payments_for_small_pools() {
        for n in 0..5 {
            for generic in 0..5 {
                let pool = [n, 1, 1, 0, 0, 1];
                let required = [0, 1, 0, 0, 0, 0];
                let d = ManaPaymentDomain {
                    available: pool,
                    required,
                    generic,
                };
                let mut expected = Vec::new();
                payments(pool, required, generic, |p| expected.push(p));
                let mut actual = Vec::new();
                d.representatives(|p| actual.push(p));
                assert_eq!(actual, expected);
            }
        }
    }
}

#[cfg(test)]
mod payment_regressions {
    use super::super::tests::{scenario, spawn};
    use super::*;
    fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
        spawn(g, p, Card::from_name(n).unwrap(), z)
    }
    fn ready(p: Player) -> Game {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        g
    }
    #[test]
    fn real_large_x_payment_keeps_nonrepresentative_colors_private_and_owned() {
        for p in [Player::P0, Player::P1] {
            let mut g = ready(p);
            g.state.players[p.index()].mana = [u32::MAX; 6];
            let id = named(&mut g, p, "Choco-Comet", Zone::Hand);
            g.apply(p, Action::AnnounceCast { card: id }).unwrap();
            g.apply(
                p,
                Action::ChooseX {
                    value: 1_000_000_000,
                },
            )
            .unwrap();
            while !matches!(g.state.pending, Pending::PayCosts { .. }) {
                let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ChooseTarget{target:Target::Player(q)} if *q==p.other()))
                    .or_else(||g.legal_actions().into_iter().find(|a|matches!(a,Action::FinishTargets))).unwrap();
                g.apply(p, a).unwrap();
            }
            let payment = [1, 2, 3, 6, 5, 999_999_985];
            let domain = g.mana_payment_domain().unwrap();
            assert!(domain.contains(payment));
            assert!(!g.legal_actions().contains(&Action::PayCosts { payment }));
            assert_eq!(g.legal_action_space().mana_payment, Some(domain));
            assert_eq!(g.observe(p).mana_payment, Some(domain));
            assert_eq!(g.observe(p.other()).mana_payment, None);
            let saved = g.snapshot();
            let revision = g.revision();
            assert_eq!(
                g.apply(p.other(), Action::PayCosts { payment }),
                Err(Error::WrongActor)
            );
            assert_eq!(
                g.apply(
                    p,
                    Action::PayCosts {
                        payment: [u32::MAX; 6]
                    }
                ),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), saved);
            assert_eq!(g.revision(), revision);
            let parent = g.clone();
            let mut child = parent.clone();
            drop(parent);
            child.apply(p, Action::PayCosts { payment }).unwrap();
            g.apply(p, Action::PayCosts { payment }).unwrap();
            assert_eq!(g.snapshot(), child.snapshot());
            let c = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(c.payment, payment);
            assert_eq!(c.mana_spent(), 1_000_000_002);
            assert_eq!(c.colors_spent(), 5);
            assert_eq!(g.observe(p).players[p.index()].spells_cast_this_turn, 1);
            g.restore(&saved).unwrap();
            g.apply(p, Action::PayCosts { payment }).unwrap();
            assert_eq!(g.snapshot(), child.snapshot());
        }
    }
    #[test]
    fn convoke_taps_do_not_become_mana_spent() {
        for p in [Player::P0, Player::P1] {
            let mut g = ready(p);
            let id = named(&mut g, p, "Sun-Dappled Celebrant", Zone::Hand);
            let helpers: Vec<_> = (0..3)
                .map(|_| named(&mut g, p, "Chrome Companion", Zone::Battlefield))
                .collect();
            g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 1];
            g.apply(p, Action::AnnounceCast { card: id }).unwrap();
            g.apply(p, Action::BeginConvoke).unwrap();
            for helper in &helpers {
                g.apply(
                    p,
                    Action::ChooseConvoker {
                        object: g.object_ref(*helper),
                        contribution: ConvokeContribution::Generic,
                    },
                )
                .unwrap();
            }
            let payment = [2, 0, 0, 0, 0, 1];
            assert!(g.mana_payment_domain().unwrap().contains(payment));
            g.apply(p, Action::PayCosts { payment }).unwrap();
            assert!(
                helpers
                    .into_iter()
                    .all(|id| g.state.objects[id as usize].tapped)
            );
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.payment, payment);
            assert_eq!(cast.mana_spent(), 3);
            assert_eq!(cast.colors_spent(), 1);
        }
    }
}

/// Frozen identity and actual payment of the completed cast that caused a trigger.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CastPaymentEvent {
    pub mana_value: u64,
    pub stack_id: u64,
    pub object: ObjectRef,
    pub card: Card,
    pub controller: Player,
    pub payment: Mana,
}
impl Game {
    pub(super) fn item_cast_payment(&self, item: &SpellView) -> Mana {
        item.cast_event
            .as_ref()
            .map(|e| e.payment)
            .or_else(|| item.cast.map(|c| c.payment))
            .unwrap_or([0; 6])
    }
    pub(super) fn cast_payment_condition(
        &self,
        condition: TriggerCondition,
        source: ObjectRef,
        event: Option<&CastPaymentEvent>,
    ) -> bool {
        let Some((minimum, increment)) = condition.mana_spent_rule() else {
            return true;
        };
        let Some(event) = event else { return false };
        let spent = event.payment.into_iter().map(u64::from).sum::<u64>();
        if spent < minimum {
            return false;
        }
        if !increment {
            return true;
        }
        self.on_battlefield(source.card)
            && self.object_ref(source.card) == source
            && self.is_creature(source.card)
            && {
                let stats = self.stats(source.card);
                i128::from(spent) > stats.power || i128::from(spent) > stats.toughness
            }
    }
}

#[cfg(test)]
mod root_adverse_payment_review {
    use super::super::tests::{scenario, spawn};
    use super::*;
    fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
        spawn(g, p, Card::from_name(name).unwrap(), zone)
    }
    fn ready(p: Player) -> Game {
        let mut g = scenario();
        g.state.active = p;
        g.state.priority = p;
        g
    }
    fn run(g: &mut Game, p: Player, a: Action) {
        g.apply(p, a).unwrap();
    }
    #[test]
    fn additional_cost_exact_incarnation_controls_numeric_domain() {
        for p in [Player::P0, Player::P1] {
            let mut g = ready(p);
            let spell = named(&mut g, p, "Thrill of Possibility", Zone::Hand);
            let discard = named(&mut g, p, "Forest", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 0];
            run(&mut g, p, Action::AnnounceCast { card: spell });
            assert!(g.mana_payment_domain().is_none());
            let old = g.object_ref(discard);
            run(&mut g, p, Action::ChooseAdditionalCost { object: old });
            let payment = [0, 1, 0, 1, 0, 0];
            assert!(g.mana_payment_domain().unwrap().contains(payment));
            let valid = g.snapshot();
            let mut fork = g.clone();
            run(&mut fork, p, Action::PayCosts { payment });
            assert_eq!(fork.state.stack[0].cast.unwrap().payment, payment);
            assert_eq!(fork.state.players[p.index()].spells_cast_this_turn, 1);
            assert_eq!(fork.state.players[p.index()].cards_discarded_this_turn, 1);
            assert_eq!(g.snapshot(), valid);
            // Native adversarial invalidation: same card, different Hand incarnation.
            g.move_object(discard, Zone::Graveyard);
            g.move_object(discard, Zone::Hand);
            assert_ne!(g.object_ref(discard), old);
            assert!(g.mana_payment_domain().is_none());
            let invalid = g.snapshot();
            let revision = g.revision();
            assert_eq!(
                g.apply(p, Action::PayCosts { payment }),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), invalid);
            assert_eq!(g.revision(), revision);
        }
    }
    #[test]
    fn late_sacrifice_capacity_restores_payment_and_uncast_announcement() {
        for p in [Player::P0, Player::P1] {
            let mut g = ready(p);
            let spell = named(&mut g, p, "Corrupted Conviction", Zone::Hand);
            let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            g.state.objects[bear as usize].generation = u64::MAX;
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            run(&mut g, p, Action::AnnounceCast { card: spell });
            let object = g.object_ref(bear);
            run(&mut g, p, Action::ChooseAdditionalCost { object });
            let payment = [0, 0, 1, 0, 0, 0];
            assert!(g.mana_payment_domain().unwrap().contains(payment));
            let saved = g.snapshot();
            let revision = g.revision();
            let views =
                [Player::P0, Player::P1].map(|v| serde_json::to_value(g.observe(v)).unwrap());
            let parent = g.clone();
            let mut child = parent.clone();
            drop(parent);
            for game in [&mut g, &mut child] {
                assert_eq!(
                    game.apply(p, Action::PayCosts { payment }),
                    Err(Error::CapacityExceeded(crate::CapacityExceeded {
                        resource: "object generation exhausted"
                    }))
                );
                assert_eq!(game.snapshot(), saved);
                assert_eq!(game.revision(), revision);
                assert_eq!(
                    game.state
                        .announcing
                        .as_ref()
                        .unwrap()
                        .item
                        .cast
                        .unwrap()
                        .payment,
                    [0; 6]
                );
                assert_eq!(game.state.players[p.index()].spells_cast_this_turn, 0);
                for (i, v) in [Player::P0, Player::P1].into_iter().enumerate() {
                    assert_eq!(serde_json::to_value(game.observe(v)).unwrap(), views[i]);
                }
            }
        }
    }
    #[test]
    fn spent_mana_survives_cost_entry_pause_and_life_is_not_mana() {
        for p in [Player::P0, Player::P1] {
            let mut g = ready(p);
            let spell = named(&mut g, p, "Final Vengeance", Zone::Hand);
            let light = named(&mut g, p, "Banishing Light", Zone::Battlefield);
            let held = named(&mut g, p, "Temple Garden", Zone::Exile);
            let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            // Deliberately synthetic link to exercise a returned-land private entry pause.
            g.state.exile_links.push(ExileLink {
                source: g.object_ref(light),
                exiled: g.object_ref(held),
                return_zone: Zone::Battlefield,
            });
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            run(&mut g, p, Action::AnnounceCast { card: spell });
            run(
                &mut g,
                p,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            let object = g.object_ref(light);
            run(&mut g, p, Action::ChooseAdditionalCost { object });
            let payment = [0, 0, 1, 0, 0, 0];
            run(&mut g, p, Action::PayCosts { payment });
            assert_eq!(g.observe(p).decision, "entry_life");
            assert_eq!(
                g.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .item
                    .cast
                    .unwrap()
                    .payment,
                payment
            );
            assert!(g.mana_payment_domain().is_none());
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            let parent = g.clone();
            let mut child = parent.clone();
            drop(parent);
            run(&mut g, p, Action::ChooseEntryLife { pay: true });
            run(&mut child, p, Action::ChooseEntryLife { pay: true });
            assert_eq!(g.snapshot(), child.snapshot());
            let c = g.state.stack[0].cast.unwrap();
            assert_eq!(c.payment, payment);
            assert_eq!(c.mana_spent(), 1);
            assert_eq!(c.colors_spent(), 1);
            assert_eq!(g.state.players[p.index()].life, 18);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
