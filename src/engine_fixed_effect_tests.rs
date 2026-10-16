use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 12] = [
    "Callous Inspector",
    "Cosmium Blast",
    "Deadly Riposte",
    "Defibrillating Current",
    "Fear of Lost Teeth",
    "Joust Through",
    "Keep Out",
    "Lightning Helix",
    "Ravenous Giant",
    "Risky Shortcut",
    "Romantic Rendezvous",
    "Viashino Pyromancer",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn choose(g: &mut Game, mode: u16, target: Option<Target>) {
    let actions = g.legal_actions();
    let a = actions
        .iter()
        .find(|a| **a == Action::ChooseMode { mode })
        .or_else(|| {
            target.and_then(|t| {
                actions
                    .iter()
                    .find(|a| **a == Action::ChooseTarget { target: t })
            })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::Discard { .. }
                )
            })
        })
        .expect("fixture choice")
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, mode: u16, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..160 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, mode, target);
    }
    panic!("unsettled");
}
fn announce(g: &mut Game, id: CardId, mode: u16, target: Option<Target>) {
    replay(g, Action::AnnounceCast { card: id });
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(g, mode, target);
    }
}
fn cast(g: &mut Game, id: CardId, mode: u16, target: Option<Target>) {
    announce(g, id, mode, target);
    settle(g, mode, target);
}
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let v = g.object_view(id);
    (v.power, v.toughness)
}
fn clues(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.owner == p
                && o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == "Clue")
        })
        .count()
}
fn attack(g: &mut Game, active: Player, id: CardId) {
    g.state.active = active;
    g.state.priority = active;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(g, Action::Attack { card: id });
    replay(g, Action::FinishAttackers);
    settle(g, 0, None);
}
fn respond(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        replay(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
#[test]
fn every_complete_body_actually_casts_and_resolves_its_full_effect_for_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            let target = if i == 7 || i == 11 {
                Some(Target::Player(p.other()))
            } else {
                Some(Target::Card(enemy))
            };
            if i == 1 || i == 5 {
                attack(&mut g, p.other(), enemy);
                respond(&mut g, p);
            } else if i == 2 || i == 6 {
                g.state.objects[enemy as usize].tapped = true;
            }
            let library = g.state.players[p.index()].library.len();
            cast(&mut g, id, 0, target);
            match i {
                0 | 4 => {
                    assert_eq!(pt(&g, id), (1, 1));
                    if i == 0 {
                        assert!(g.keywords(id).contains(Keywords::MENACE));
                    }
                    let shock = named(&mut g, p, "Shock", Zone::Hand);
                    announce(&mut g, shock, 0, Some(Target::Card(id)));
                    settle(&mut g, 0, Some(Target::Player(p.other())));
                    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                    if i == 0 {
                        assert_eq!(g.state.players[p.index()].life, 19);
                        assert_eq!(clues(&g, p), 1);
                    } else {
                        assert_eq!(g.state.players[p.index()].life, 21);
                        assert_eq!(g.state.players[p.other().index()].life, 19);
                    }
                }
                1 | 2 | 3 | 5 | 6 => {
                    assert_eq!(
                        g.object_view(enemy).damage,
                        if i == 1 || i == 3 || i == 6 { 4 } else { 3 }
                    );
                    assert_eq!(
                        g.state.players[p.index()].life,
                        20 + if i == 2 || i == 3 {
                            2
                        } else if i == 5 {
                            1
                        } else {
                            0
                        }
                    );
                }
                7 => {
                    assert_eq!(g.state.players[p.index()].life, 23);
                    assert_eq!(g.state.players[p.other().index()].life, 17);
                }
                8 => {
                    assert_eq!(pt(&g, id), (5, 5));
                    g.begin_step(Step::Upkeep);
                    settle(&mut g, 0, None);
                    assert_eq!(g.state.players[p.index()].life, 19);
                }
                9 => {
                    assert_eq!(g.state.players[p.index()].library.len(), library - 2);
                    assert_eq!(g.state.players[0].life, 18);
                    assert_eq!(g.state.players[1].life, 18);
                }
                10 => {
                    assert_eq!(g.state.players[p.index()].library.len(), library - 2);
                    assert_eq!(g.hand_count(p), 2);
                }
                11 => {
                    assert_eq!(pt(&g, id), (2, 1));
                    assert_eq!(g.state.players[p.other().index()].life, 18);
                }
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn defibrillating_current_pays_each_of_eight_exact_two_brid_options() {
    for p in [Player::P0, Player::P1] {
        for option in 0..8 {
            let mut g = setup(p);
            let enemy = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
            let id = named(&mut g, p, NAMES[3], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            let costs = g.observe(p).cost_choices;
            assert_eq!(costs.len(), 8);
            let cost = costs[option].cost;
            let spent = cost.colored.iter().copied().sum::<u32>() + cost.generic;
            replay(
                &mut g,
                Action::ChooseSpellCost {
                    cost: costs[option].index,
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            while g.observe(p).announcing.is_some() {
                choose(&mut g, 0, Some(Target::Card(enemy)));
            }
            assert_eq!(
                120 - g.state.players[p.index()].mana.iter().sum::<u32>(),
                spent
            );
            settle(&mut g, 0, None);
            assert_eq!(g.object_view(enemy).damage, 4);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(Card::from_name(NAMES[3]).unwrap().fixed_mana_value(), 6);
        }
    }
}
#[test]
fn exact_tapped_filter_is_required_and_revalidated_without_life_gain_fallback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[2], Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: id });
        g.state.objects[enemy as usize].tapped = true;
        replay(&mut g, Action::AnnounceCast { card: id });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        while g.observe(p).announcing.is_some() {
            choose(&mut g, 0, Some(Target::Card(enemy)));
        }
        g.state.objects[enemy as usize].tapped = false;
        settle(&mut g, 0, None);
        assert_eq!(g.object_view(enemy).damage, 0);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn combat_damage_subjects_require_committed_attacking_or_blocking_creatures() {
    for p in [Player::P0, Player::P1] {
        for i in [1, 5] {
            let mut g = setup(p);
            let attacker = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
            let outsider = named(&mut g, p, "Ravenous Giant", Zone::Battlefield);
            let id = named(&mut g, p, NAMES[i], Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: id });
            attack(&mut g, p.other(), attacker);
            respond(&mut g, p);
            replay(&mut g, Action::AnnounceCast { card: id });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(outsider),
                },
            );
            while g.observe(p).announcing.is_some() {
                choose(&mut g, 0, Some(Target::Card(attacker)));
            }
            g.state.attackers.clear();
            settle(&mut g, 0, None);
            assert_eq!(g.object_view(attacker).damage, 0);
            assert_eq!(g.state.players[p.index()].life, 20);
        }
    }
}
#[test]
fn blocking_creature_is_a_legal_damage_target_without_being_tapped() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attacker = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
        let blocker = named(&mut g, p, "Ravenous Giant", Zone::Battlefield);
        attack(&mut g, p.other(), attacker);
        for _ in 0..12 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            replay(&mut g, Action::Pass);
        }
        replay(&mut g, Action::Block { blocker, attacker });
        replay(&mut g, Action::FinishBlockers);
        respond(&mut g, p);
        assert!(!g.object_view(blocker).tapped);
        g.state.players[p.index()].mana = [20; 6];
        let id = named(&mut g, p, NAMES[1], Zone::Hand);
        cast(&mut g, id, 0, Some(Target::Card(blocker)));
        assert_eq!(g.object_view(blocker).damage, 4);
    }
}
#[test]
fn keep_out_modes_bind_their_own_target_rules_without_fallback() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..2 {
            let mut g = setup(p);
            let creature = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
            g.state.objects[creature as usize].tapped = true;
            let enchantment = named(&mut g, p.other(), "Ferocification", Zone::Battlefield);
            let id = named(&mut g, p, NAMES[6], Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: id });
            replay(&mut g, Action::ChooseMode { mode });
            let (legal, illegal) = if mode == 0 {
                (creature, enchantment)
            } else {
                (enchantment, creature)
            };
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(illegal),
                },
            );
            while g.observe(p).announcing.is_some() {
                choose(&mut g, mode, Some(Target::Card(legal)));
            }
            settle(&mut g, mode, None);
            if mode == 0 {
                assert_eq!(g.object_view(creature).damage, 4);
                assert_eq!(g.object_view(enchantment).zone, Zone::Battlefield);
            } else {
                assert_eq!(g.object_view(enchantment).zone, Zone::Graveyard);
                assert_eq!(g.object_view(creature).damage, 0);
            }
        }
    }
}
#[test]
fn viashino_player_or_planeswalker_union_excludes_creatures_and_deals_loyalty_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
        let walker = named(
            &mut g,
            p.other(),
            "Professor Dellian Fel",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        let id = named(&mut g, p, NAMES[11], Zone::Hand);
        announce(&mut g, id, 0, None);
        for _ in 0..12 {
            if g.observe(p)
                .announcing
                .as_ref()
                .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
            {
                break;
            }
            choose(&mut g, 0, None);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        settle(&mut g, 0, Some(Target::Card(walker)));
        assert_eq!(g.object_view(walker).counters.loyalty, 3);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn illegal_exact_target_incarnation_fizzles_helix_and_its_life_gain() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Ravenous Giant", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[7], Zone::Hand);
        announce(&mut g, id, 0, Some(Target::Card(target)));
        let old = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_ne!(old, g.object_ref(target));
        settle(&mut g, 0, None);
        assert_eq!(g.object_view(target).damage, 0);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn ward_decline_counters_damage_and_gain_as_one_spell() {
    let mut g = setup(Player::P0);
    let target = named(&mut g, Player::P1, "Skyward Spider", Zone::Battlefield);
    let id = named(&mut g, Player::P0, NAMES[7], Zone::Hand);
    announce(&mut g, id, 0, Some(Target::Card(target)));
    for _ in 0..12 {
        if g.observe(Player::P0).resolution_payment.is_some() {
            break;
        }
        choose(&mut g, 0, None);
    }
    replay(&mut g, Action::DeclineResolutionMana);
    settle(&mut g, 0, None);
    assert_eq!(g.object_view(target).damage, 0);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
}
#[test]
fn romantic_discards_before_drawing_and_private_continuation_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hand = named(&mut g, p, "Plains", Zone::Hand);
        let id = named(&mut g, p, NAMES[10], Zone::Hand);
        announce(&mut g, id, 0, None);
        let library = g.state.players[p.index()].library.len();
        for _ in 0..12 {
            if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                break;
            }
            choose(&mut g, 0, None);
        }
        assert_eq!(g.state.players[p.index()].library.len(), library);
        assert_eq!(g.hand_count(p), 1);
        let other = g.observe(p.other());
        assert!(other.effect_discards.is_empty() && other.actions.is_empty());
        g.move_object(id, Zone::Exile);
        replay(&mut g, Action::Discard { card: hand });
        settle(&mut g, 0, None);
        assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].library.len(), library - 2);
    }
}
#[test]
fn empty_hand_romantic_still_draws_and_risky_draw_then_both_life_losses_finish_before_sbas() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[10], Zone::Hand);
        cast(&mut g, id, 0, None);
        assert_eq!(g.hand_count(p), 2);
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[9], Zone::Hand);
        g.state.players[0].life = 2;
        g.state.players[1].life = 2;
        let library = g.state.players[p.index()].library.len();
        cast(&mut g, id, 0, None);
        assert_eq!(g.state.players[p.index()].library.len(), library - 2);
        assert_eq!(g.state.players[0].life, 0);
        assert_eq!(g.state.players[1].life, 0);
        assert_eq!(g.outcome(), Some(Outcome::Draw));
    }
}
#[test]
fn death_damage_retains_last_known_lifelink_facts_and_its_independent_life_gain() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[4], Zone::Hand);
        cast(&mut g, id, 0, None);
        g.state.objects[id as usize].gained_keywords = Keywords::LIFELINK;
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        announce(&mut g, shock, 0, Some(Target::Card(id)));
        settle(&mut g, 0, Some(Target::Player(p.other())));
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn callous_clue_keeps_its_complete_sacrifice_and_draw_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[0], Zone::Hand);
        cast(&mut g, id, 0, None);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, 0, Some(Target::Card(id)));
        assert_eq!(clues(&g, p), 1);
        let clue = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.owner == p
                    && o.zone == Zone::Battlefield
                    && o.card.token_info().is_some_and(|t| t.subtypes == "Clue")
            })
            .unwrap()
            .0 as CardId;
        let library = g.state.players[p.index()].library.len();
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: clue,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.observe(p).announcing.is_some() {
            choose(&mut g, 0, None);
        }
        settle(&mut g, 0, None);
        assert_eq!(clues(&g, p), 0);
        assert_eq!(g.state.players[p.index()].library.len(), library - 1);
    }
}
#[test]
fn giant_triggers_only_on_its_controllers_upkeep_and_uses_damage_not_life_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[8], Zone::Hand);
        cast(&mut g, id, 0, None);
        g.state.objects[id as usize].gained_keywords = Keywords::LIFELINK;
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        settle(&mut g, 0, None);
        assert_eq!(g.state.players[p.index()].life, 20);
        g.state.active = p;
        g.begin_step(Step::Upkeep);
        settle(&mut g, 0, None);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.index()].life_lost_this_turn, 1);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 1);
    }
}
#[test]
fn gain_overflow_restores_damage_histories_source_spell_and_revision_atomically() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, NAMES[7], Zone::Hand);
    g.state.players[0].life = i128::MAX;
    announce(&mut g, id, 0, Some(Target::Player(Player::P1)));
    replay(&mut g, Action::Pass);
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    assert!(matches!(
        g.apply(p, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
