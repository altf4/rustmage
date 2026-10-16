use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let old = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, old, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(after, g.snapshot());
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(saved, g.snapshot());
    assert_eq!(rev, g.revision());
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn choose_trigger(g: &mut Game, target: CardId) {
    g.prepare_triggers();
    for _ in 0..20 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a, Action::PutTrigger { .. } | Action::FinishTargets)
                    || *a
                        == Action::ChooseTarget {
                            target: Target::Card(target),
                        }
            })
            .unwrap();
        replay(g, a);
    }
    panic!("trigger did not finish");
}
fn committed(g: &mut Game, name: &str, cost: ManaCost, target: Option<CardId>) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [20; 6];
    replay(g, Action::AnnounceCast { card: id });
    assert_eq!(g.state.pending, Pending::SpellCost { player: Player::P0 });
    assert!(g.observe(Player::P1).actions.is_empty());
    let index = g
        .observe(Player::P0)
        .cost_choices
        .iter()
        .find(|c| c.cost == cost)
        .unwrap()
        .index;
    reject(g, Action::ChooseSpellCost { cost: u16::MAX });
    replay(g, Action::ChooseSpellCost { cost: index });
    while g.state.pending != (Pending::PayCosts { player: Player::P0 }) {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a, Action::FinishTargets)
                    || target.is_some_and(|id| {
                        *a == Action::ChooseTarget {
                            target: Target::Card(id),
                        }
                    })
            })
            .unwrap();
        replay(g, a);
    }
    let request = g.observe(Player::P0).payment_request.unwrap();
    assert_eq!(request.required, cost.colored);
    assert_eq!(request.generic, cost.generic);
    reject(g, Action::ChooseSpellCost { cost: 0 });
    reject(g, Action::PayCosts { payment: [0; 6] });
    let mut payment = cost.colored;
    payment[5] += cost.generic;
    replay(g, Action::PayCosts { payment });
    assert_eq!(
        g.state.players[0].mana,
        std::array::from_fn(|i| 20 - payment[i])
    );
    assert_eq!(g.object_view(id).zone, Zone::Stack);
    id
}

#[test]
fn two_brid_prefunded_payments_match_independent_exhaustive_assignments_without_duplicates() {
    let card = Card::from_name("Kin-Tree Severance").unwrap();
    for w in 0..=2 {
        for b in 0..=2 {
            for green in 0..=2 {
                for blue in 0..=1 {
                    for c in 0..=4 {
                        let pool = [w, blue, b, 0, green, c];
                        let mut actual = Vec::new();
                        card.spell_payments(pool, |p| actual.push(p));
                        let mut expected = Vec::new();
                        for pw in 0..=w {
                            for pu in 0..=blue {
                                for pb in 0..=b {
                                    for pg in 0..=green {
                                        for pc in 0..=c {
                                            let spent = pw + pu + pb + pg + pc;
                                            if (0u32..8).any(|mask| {
                                                let cw = mask & 1;
                                                let cb = (mask >> 1) & 1;
                                                let cg = (mask >> 2) & 1;
                                                let n = cw + cb + cg;
                                                pw >= cw && pb >= cb && pg >= cg && spent == 6 - n
                                            }) {
                                                expected.push([pw, pu, pb, 0, pg, pc]);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        actual.sort();
                        expected.sort();
                        assert_eq!(actual, expected, "pool {pool:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn all_eight_committed_choices_pay_their_exact_cost_and_keep_mana_value_six() {
    let card = Card::from_name("Kin-Tree Severance").unwrap();
    let options = card.cost_options().collect::<Vec<_>>();
    assert_eq!(options.len(), 8);
    assert_eq!(
        options[0],
        ManaCost {
            colored: [0; 6],
            generic: 6
        }
    );
    assert_eq!(card.fixed_mana_value(), 6);
    let mut totals = std::collections::BTreeSet::new();
    for cost in options {
        let mut g = scenario();
        let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
        let spell = committed(&mut g, "Kin-Tree Severance", cost, Some(victim));
        totals.insert(cost.generic + cost.colored.iter().sum::<u32>());
        assert_eq!(g.state.stack[0].source_card.fixed_mana_value(), 6);
        assert_eq!(g.state.stack[0].source_card.colors(), 1 | 4 | 16);
        passes(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
    assert_eq!(totals, [3, 4, 5, 6].into_iter().collect());
}

#[test]
fn funding_offers_only_affordable_two_brid_choices_and_cannot_switch_after_commit() {
    let mut g = scenario();
    let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    for name in ["Plains", "Swamp", "Forest"] {
        named(&mut g, Player::P0, name, Zone::Battlefield);
    }
    let spell = named(&mut g, Player::P0, "Kin-Tree Severance", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: spell });
    let choices = g.observe(Player::P0).cost_choices;
    assert_eq!(choices.len(), 1);
    let chosen = &choices[0];
    assert_eq!(
        chosen.cost,
        ManaCost {
            colored: [1, 0, 1, 0, 1, 0],
            generic: 0
        }
    );
    reject(&mut g, Action::ChooseSpellCost { cost: 0 });
    replay(&mut g, Action::ChooseSpellCost { cost: chosen.index });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    assert_eq!(g.state.pending, Pending::PayCosts { player: Player::P0 });
    g.state.players[0].mana = [0, 6, 0, 0, 0, 0];
    // Six generic mana would cover another assignment, but not this commitment.
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 6, 0, 0, 0, 0],
        },
    );
    reject(&mut g, Action::ChooseSpellCost { cost: 0 });
    for _ in 0..3 {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::TapMana { .. }))
            .unwrap();
        replay(&mut g, a);
    }
    replay(
        &mut g,
        Action::PayCosts {
            payment: [1, 0, 1, 0, 1, 0],
        },
    );
    assert_eq!(g.state.players[0].mana, [0, 6, 0, 0, 0, 0]);
    passes(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Exile);
}

#[test]
fn cheap_two_brid_cast_is_high_mana_value_for_counter_targets() {
    for cost in [
        ManaCost {
            colored: [1, 0, 1, 0, 1, 0],
            generic: 0,
        },
        ManaCost {
            colored: [0; 6],
            generic: 6,
        },
    ] {
        let mut g = scenario();
        let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
        let spell = committed(&mut g, "Kin-Tree Severance", cost, Some(victim));
        replay(&mut g, Action::Pass);
        g.state.players[1].mana = [20; 6];
        let snare = named(&mut g, Player::P1, "Spell Snare", Zone::Hand);
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Cast{card,target:Some(Target::Spell(id)),..}if *card==snare&&*id==spell)));
        let counter = named(&mut g, Player::P1, "Disdainful Stroke", Zone::Hand);
        cast(&mut g, counter, Some(Target::Spell(spell)));
        passes(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
    }
}

#[test]
fn two_brid_reputable_merchant_counters_on_entry_and_death_after_source_leaves() {
    for cost in [
        ManaCost {
            colored: [1, 0, 1, 0, 1, 0],
            generic: 0,
        },
        ManaCost {
            colored: [0; 6],
            generic: 6,
        },
    ] {
        let mut g = scenario();
        let recipient = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let source = committed(&mut g, "Reputable Merchant", cost, None);
        passes(&mut g);
        choose_trigger(&mut g, recipient);
        passes(&mut g);
        assert_eq!(g.object_view(recipient).counters.plus_one, 1);
        g.state.priority = Player::P1;
        g.state.players[1].mana = [20; 6];
        let murder = named(&mut g, Player::P1, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(source)));
        passes(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        choose_trigger(&mut g, recipient);
        passes(&mut g);
        assert_eq!(g.object_view(recipient).counters.plus_one, 2);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
    }
}

#[test]
fn temur_two_brid_entry_draws_once_then_restores_private_discard_continuation() {
    for cost in [
        ManaCost {
            colored: [0, 1, 0, 1, 1, 0],
            generic: 0,
        },
        ManaCost {
            colored: [0; 6],
            generic: 6,
        },
    ] {
        let mut g = scenario();
        let spare = named(&mut g, Player::P0, "Shock", Zone::Hand);
        let library = g.state.players[0].library.len();
        let source = committed(&mut g, "Temur Tawnyback", cost, None);
        passes(&mut g);
        choose_trigger(&mut g, source);
        passes(&mut g);
        assert_eq!(g.state.players[0].library.len(), library - 1);
        assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
        assert_eq!(g.actor(), Some(Player::P0));
        assert!(g.observe(Player::P1).effect_discards.is_empty());
        assert!(g.observe(Player::P1).actions.is_empty());
        reject(&mut g, Action::Pass);
        replay(&mut g, Action::Discard { card: spare });
        assert_eq!(g.state.players[0].library.len(), library - 1);
        assert_eq!(g.object_view(spare).zone, Zone::Graveyard);
        assert_eq!(
            (g.object_view(source).power, g.object_view(source).toughness),
            (4, 3)
        );
        assert!(g.state.stack.is_empty());
    }
}
