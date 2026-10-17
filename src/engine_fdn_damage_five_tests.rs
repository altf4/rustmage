use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn replay(g: &mut Game, a: Action) {
    let p = g.actor().unwrap();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|b| *b == a)
        .expect("legal");
    let before = g.snapshot();
    assert_eq!(g.apply(p.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(g.snapshot(), before);
    let mut raw = {
        let parent = g.clone();
        parent.clone()
    };
    let mut indexed = {
        let parent = g.clone();
        parent.clone()
    };
    raw.apply(p, a.clone()).unwrap();
    indexed.apply_index(p, rev, index).unwrap();
    act(g, a.clone());
    assert_eq!(g.snapshot(), raw.snapshot());
    assert_eq!(g.snapshot(), indexed.snapshot());
    for v in [p, p.other()] {
        assert_eq!(
            serde_json::to_value(g.observe(v)).unwrap(),
            serde_json::to_value(raw.observe(v)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, index), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn settle(g: &mut Game) {
    g.advance_replacements(false);
    g.prepare_triggers();
    for _ in 0..120 {
        if g.outcome().is_some() || g.state.pending == Pending::Priority && g.state.stack.is_empty()
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseReplacement { .. }
                )
            })
            .expect("settle");
        replay(g, a);
    }
    panic!("unsettled");
}
fn paid(g: &mut Game, p: Player, n: &str) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    replay(g, Action::AnnounceCast { card: id });
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        replay(g, a);
    }
    settle(g);
    id
}
fn hit(g: &Game, id: CardId, target: Target, n: i128, combat: bool) -> DamageEvent {
    DamageEvent {
        source: g.damage_source(id),
        target,
        amount: n,
        combat,
    }
}
#[test]
fn fdn_damage_five_paid_mana_keywords_and_static_permissions() {
    for p in [Player::P0, Player::P1] {
        for (name, mana) in [
            ("Gratuitous Violence", [0, 0, 0, 3, 0, 2]),
            ("Twinflame Tyrant", [0, 0, 0, 2, 0, 3]),
            ("Crystal Barricade", [1, 0, 0, 0, 0, 1]),
            ("Niv-Mizzet, Visionary", [0, 1, 0, 1, 0, 4]),
            ("Sphinx of the Final Word", [0, 2, 0, 0, 0, 5]),
        ] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = mana;
            let id = paid(&mut g, p, name);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            if name == "Crystal Barricade" {
                assert!(g.keywords(id).contains(Keywords::DEFENDER));
                assert!(g.player_has_hexproof(p));
                assert!(!g.player_has_hexproof(p.other()));
            }
            if [
                "Twinflame Tyrant",
                "Niv-Mizzet, Visionary",
                "Sphinx of the Final Word",
            ]
            .contains(&name)
            {
                assert!(g.keywords(id).contains(Keywords::FLYING));
            }
            if name == "Sphinx of the Final Word" {
                assert!(g.keywords(id).contains(Keywords::HEXPROOF));
                assert!(g.controlled_spells_uncounterable(
                    p,
                    Card::from_name("Lightning Strike").unwrap()
                ));
                assert!(!g.controlled_spells_uncounterable(p, Card::GrizzlyBears));
            }
        }
    }
}
#[test]
fn fdn_damage_five_recipient_chooses_each_exact_source_once_owned_replay() {
    for p in [Player::P0, Player::P1] {
        for reverse in [false, true] {
            let mut g = setup(p);
            let dealer = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let a = paid(&mut g, p, "Gratuitous Violence");
            let b = paid(&mut g, p, "Twinflame Tyrant");
            let event = hit(&g, dealer, Target::Player(p.other()), 2, true);
            g.apply_damage_batch(&[event]);
            assert!(g.advance_replacements(false));
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(g.state.players[p.other().index()].life, 20);
            let choices = g.legal_actions();
            assert_eq!(
                choices
                    .iter()
                    .filter(|a| matches!(a, Action::ChooseReplacement { .. }))
                    .count(),
                2
            );
            let want = if reverse { b } else { a };
            let choice = choices
                .into_iter()
                .find(|a| matches!(a,Action::ChooseReplacement{source,..}if source.card==want))
                .unwrap();
            let view = g.observe(p).replacement_choice.unwrap();
            assert_eq!(view.damage.unwrap().amount, 2);
            assert_eq!(view.effects.len(), 2);
            replay(&mut g, choice);
            assert_eq!(g.state.players[p.other().index()].life, 12);
            assert!(g.state.replacements.is_none());
        }
    }
}
#[test]
fn fdn_damage_five_prevention_other_only_noncombat_and_no_damage_history() {
    for p in [Player::P0, Player::P1] {
        for prevent_first in [false, true] {
            let mut g = setup(p);
            let barrier = paid(&mut g, p, "Crystal Barricade");
            let other = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let dealer = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let doubler = named(&mut g, p.other(), "Twinflame Tyrant", Zone::Battlefield);
            let e = hit(&g, dealer, Target::Card(other), 2, false);
            g.apply_damage_batch(&[e]);
            assert!(g.advance_replacements(false));
            assert_eq!(g.actor(), Some(p));
            let choose = if prevent_first { barrier } else { doubler };
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ChooseReplacement{source,..}if source.card==choose))
                .unwrap();
            replay(&mut g, a);
            assert_eq!(g.object_view(other).damage, 0);
            assert!(!g.object_view(dealer).dealt_damage_this_turn);
            assert!(!g.object_view(other).damaged_this_turn);
            let e = hit(&g, dealer, Target::Card(barrier), 1, false);
            g.apply_damage_batch(&[e]);
            g.advance_replacements(false);
            assert_eq!(g.object_view(barrier).damage, 2);
            let e = hit(&g, dealer, Target::Card(other), 1, true);
            g.apply_damage_batch(&[e]);
            g.advance_replacements(false);
            assert_eq!(g.object_view(other).damage, 2);
        }
    }
}
#[test]
fn fdn_damage_five_gratuitous_all_recipients_creature_lki_and_tyrant_any_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let violence = paid(&mut g, p, "Gratuitous Violence");
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let wall = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let source = g.damage_source(bear);
        g.move_object(bear, Zone::Graveyard);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Card(wall),
            amount: 1,
            combat: false,
        }]);
        g.advance_replacements(false);
        assert_eq!(g.object_view(wall).damage, 2);
        let spell = named(&mut g, p, "Lightning Strike", Zone::Stack);
        let e = hit(&g, spell, Target::Player(p.other()), 2, false);
        g.apply_damage_batch(&[e]);
        g.advance_replacements(false);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        g.move_object(violence, Zone::Graveyard);
        let tyrant = named(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        let e = hit(&g, spell, Target::Player(p.other()), 2, false);
        g.apply_damage_batch(&[e]);
        g.advance_replacements(false);
        assert_eq!(g.state.players[p.other().index()].life, 14);
        let e = hit(&g, spell, Target::Player(p), 2, false);
        g.apply_damage_batch(&[e]);
        assert_eq!(g.state.players[p.index()].life, 18);
        g.move_object(tyrant, Zone::Graveyard);
    }
}
#[test]
fn fdn_damage_five_niv_captures_actual_noncombat_amount_and_survives_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let niv = paid(&mut g, p, "Niv-Mizzet, Visionary");
        let spell = named(&mut g, p, "Lightning Strike", Zone::Stack);
        let tyrant = named(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        for _ in 0..10 {
            named(&mut g, p, "Island", Zone::Library);
        }
        let e = hit(&g, spell, Target::Player(p.other()), 2, false);
        g.apply_damage_batch(&[e]);
        g.advance_replacements(false);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.card == niv)
                .count(),
            1
        );
        let t = g
            .state
            .pending_triggers
            .iter()
            .find(|s| s.card == niv)
            .unwrap();
        assert_eq!(t.damage_event.unwrap().amount, 4);
        g.move_object(niv, Zone::Graveyard);
        g.move_object(tyrant, Zone::Graveyard);
        let before = g
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Hand && o.owner == p)
            .count();
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Hand && o.owner == p)
                .count(),
            before + 4
        );
        let niv = named(&mut g, p, "Niv-Mizzet, Visionary", Zone::Battlefield);
        for (target, combat) in [(p.other(), true), (p, false)] {
            let e = hit(&g, spell, Target::Player(target), 1, combat);
            g.apply_damage_batch(&[e]);
            assert!(!g.state.pending_triggers.iter().any(|s| s.card == niv));
        }
    }
}
#[test]
fn fdn_damage_five_sphinx_legal_counter_targets_live_removal_and_hexproof() {
    for p in [Player::P0, Player::P1] {
        for protected in [true, false] {
            let mut g = setup(p);
            let sphinx = named(&mut g, p, "Sphinx of the Final Word", Zone::Hand);
            cast(&mut g, sphinx, None);
            act(&mut g, Action::Pass);
            let c = named(&mut g, p.other(), "Counterspell", Zone::Hand);
            cast(&mut g, c, Some(Target::Spell(sphinx)));
            pass_pair(&mut g);
            assert_eq!(g.object_view(sphinx).zone, Zone::Stack);
            pass_pair(&mut g);
            assert_eq!(g.object_view(sphinx).zone, Zone::Battlefield);
            let spell = named(&mut g, p, "Lightning Strike", Zone::Hand);
            cast(&mut g, spell, Some(Target::Player(p.other())));
            act(&mut g, Action::Pass);
            let c = named(&mut g, p.other(), "Counterspell", Zone::Hand);
            cast(&mut g, c, Some(Target::Spell(spell)));
            if !protected {
                g.move_object(sphinx, Zone::Graveyard);
            }
            pass_pair(&mut g);
            assert_eq!(
                g.object_view(spell).zone,
                if protected {
                    Zone::Stack
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn fdn_damage_five_player_hexproof_control_departure_and_ability_loss() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let barrier = paid(&mut g, p, "Crystal Barricade");
        let strike = named(&mut g, p.other(), "Lightning Strike", Zone::Hand);
        g.state.priority = p.other();
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Cast{card,target:Some(Target::Player(t)),..}if *card==strike&&*t==p)));
        g.state.objects[barrier as usize].controller = p.other();
        assert!(!g.player_has_hexproof(p));
        assert!(g.player_has_hexproof(p.other()));
        g.move_object(barrier, Zone::Graveyard);
        assert!(!g.player_has_hexproof(p.other()));
    }
}

#[test]
fn fdn_damage_five_replacement_capacity_failure_is_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let dealer = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Gratuitous Violence", Zone::Battlefield);
        named(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        let e = hit(&g, dealer, Target::Player(p.other()), i128::MAX, false);
        g.apply_damage_batch(&[e]);
        g.advance_replacements(false);
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        let before = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(p.other(), action),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
    }
}
#[test]
fn fdn_damage_five_paid_noncombat_spell_suspends_and_resumes_after_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        named(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        let strike = named(&mut g, p, "Lightning Strike", Zone::Hand);
        cast(&mut g, strike, Some(Target::Player(p.other())));
        pass_pair(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.observe(p).decision, "replacement");
        assert!(g.state.resolving.is_some());
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[p.other().index()].life, 8);
        assert!(g.state.resolving.is_none());
        assert_eq!(g.object_view(strike).zone, Zone::Graveyard);
    }
}
#[test]
fn fdn_damage_five_removed_static_abilities_stop_protection_and_doubling() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Crystal Barricade",
            "Twinflame Tyrant",
            "Sphinx of the Final Word",
        ] {
            let mut g = setup(p);
            let source = paid(&mut g, p, name);
            let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
            cast(&mut g, aura, Some(Target::Card(source)));
            settle(&mut g);
            assert!(!g.printed_abilities_available(source));
            assert!(!g.player_has_hexproof(p));
            assert!(
                !g.controlled_spells_uncounterable(p, Card::from_name("Lightning Strike").unwrap())
            );
            let spell = named(&mut g, p, "Lightning Strike", Zone::Stack);
            let e = hit(&g, spell, Target::Player(p.other()), 1, false);
            g.apply_damage_batch(&[e]);
            g.advance_replacements(false);
            assert_eq!(g.state.players[p.other().index()].life, 19);
        }
    }
}
#[test]
fn fdn_damage_five_simultaneous_replacement_choices_are_apnap_before_any_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        named(&mut g, p, "Gratuitous Violence", Zone::Battlefield);
        named(&mut g, p, "Gratuitous Violence", Zone::Battlefield);
        let events = [
            hit(&g, source, Target::Player(p.other()), 1, false),
            hit(&g, source, Target::Player(p), 1, false),
        ];
        g.apply_damage_batch(&events);
        assert!(g.advance_replacements(false));
        assert_eq!(g.actor(), Some(p));
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[p.index()].life, 16);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn fdn_damage_five_departed_source_controller_lki_differs_from_ability_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let item = g.activation_item(p, source, 0);
        g.state.stack.push(item);
        g.state.objects[source as usize].controller = p.other();
        g.move_object(source, Zone::Graveyard);
        let item = g.state.stack.last().unwrap();
        assert_eq!(item.controller, p);
        let dealer = g.program_damage_source(item);
        assert_eq!(dealer.controller, p.other());
        assert!(dealer.facts.battlefield);
        assert!(dealer.facts.is_creature());
        named(&mut g, p.other(), "Gratuitous Violence", Zone::Battlefield);
        g.apply_damage_batch(&[DamageEvent {
            source: dealer,
            target: Target::Player(p),
            amount: 1,
            combat: false,
        }]);
        g.advance_replacements(false);
        assert_eq!(g.state.players[p.index()].life, 18);
    }
}
#[test]
fn fdn_damage_five_niv_hand_limit_tracks_live_source_controller_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let niv = paid(&mut g, p, "Niv-Mizzet, Visionary");
            assert!(g.no_maximum_hand_size(p));
            assert!(g.observe(p.other()).players[p.index()].no_maximum_hand_size);
            match mode {
                0 => {}
                1 => {
                    g.move_object(niv, Zone::Graveyard);
                }
                2 => {
                    let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
                    cast(&mut g, aura, Some(Target::Card(niv)));
                    settle(&mut g);
                }
                3 => {
                    g.state.objects[niv as usize].controller = p.other();
                }
                _ => unreachable!(),
            }
            assert_eq!(g.no_maximum_hand_size(p), mode == 0);
            assert_eq!(g.no_maximum_hand_size(p.other()), mode == 3);
            for _ in 0..8 {
                named(&mut g, p, "Grizzly Bears", Zone::Hand);
            }
            g.state.step = Step::End;
            g.give_priority();
            pass_pair(&mut g);
            if mode == 0 {
                assert_ne!(g.state.pending, Pending::Discard);
                assert_eq!(g.state.active, p.other());
                assert_eq!(g.hand_count(p), 8);
            } else {
                assert_eq!(g.state.pending, Pending::Discard);
                assert_eq!(g.actor(), Some(p));
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Discard { .. }))
                    .unwrap();
                replay(&mut g, a);
                assert_eq!(g.hand_count(p), 7);
            }
        }
    }
}
