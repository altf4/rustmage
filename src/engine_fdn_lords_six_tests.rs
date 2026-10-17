//! Paid six-card FDN obligations, continuous layers and event qualifications.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
#[test]
fn fdn_six_captain_treasure_tribal_lord_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Spectral Sailor", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Spectral Sailor", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let captain = paid(&mut g, p, "Corsair Captain", [2, 1, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.stats(own).power, 2);
        assert_eq!(g.stats(foe).power, 1);
        assert_eq!(g.stats(captain).power, 2);
        assert_eq!(g.stats(bear).power, 2);
        let treasure = g
            .state
            .objects
            .iter()
            .position(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card
                        .token_info()
                        .is_some_and(|t| t.name == "Treasure Token")
            })
            .unwrap() as CardId;
        assert_eq!(g.object_view(treasure).controller, p);
        g.state.players[p.index()].mana = [30; 6];
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(captain)]);
        settle(&mut g);
        assert_eq!(g.stats(own).power, 1);
        g.move_object(stop, Zone::Graveyard);
        assert_eq!(g.stats(own).power, 2);
        let before = g.state.players[p.index()].mana;
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production,..} if *card==treasure && production[4]==1)).unwrap();
        checked(&mut g, action);
        assert_eq!(g.object_view(treasure).zone, Zone::Outside);
        assert_eq!(g.state.players[p.index()].mana[4], before[4] + 1);
    }
}
#[test]
fn fdn_six_archdruid_paid_lord_sick_mana_count_and_blank() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let arch = paid(&mut g, p, "Elvish Archdruid", [1, 0, 0, 0, 2, 0]);
        settle(&mut g);
        assert_eq!(g.stats(elf).power, 2);
        assert_eq!(g.stats(foe).power, 1);
        assert_eq!(g.stats(arch).power, 2);
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,..}|Action::AnnounceMana{card,..} if *card==arch)));
        g.state.objects[arch as usize].summoning_sick = false;
        let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==arch&&*production==[0,0,0,0,2,0])).unwrap();
        checked(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
        g.state.objects[arch as usize].tapped = false;
        g.move_object(elf, Zone::Graveyard);
        assert!(g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,production,..} if *card==arch&&*production==[0,0,0,0,1,0])));
        g.state.players[p.index()].mana = [30; 6];
        let stop = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, stop, None, &[Target::Card(arch)]);
        settle(&mut g);
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,..}|Action::AnnounceMana{card,..} if *card==arch)));
    }
}
#[test]
fn fdn_six_uprising_intervening_if_and_entry_event_lki_trample() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let large = named(&mut g, p, "Aggressive Mammoth", Zone::Battlefield);
        let small = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let hand = g.hand_count(p);
        let uprising = paid(&mut g, p, "Garruk's Uprising", [2, 0, 0, 0, 1, 0]);
        super::tests::pass_pair(&mut g);
        g.prepare_triggers();
        g.move_object(large, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand);
        assert!(g.keywords(small).contains(Keywords::TRAMPLE));
        let large = paid(&mut g, p, "Aggressive Mammoth", [3, 0, 0, 0, 3, 0]);
        super::tests::pass_pair(&mut g);
        g.prepare_triggers();
        g.move_object(large, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(!g.keywords(small).contains(Keywords::FLYING));
        g.move_object(uprising, Zone::Graveyard);
        assert!(!g.keywords(small).contains(Keywords::TRAMPLE));
        let mut g = setup(p);
        named(&mut g, p, "Aggressive Mammoth", Zone::Battlefield);
        let hand = g.hand_count(p);
        paid(&mut g, p, "Garruk's Uprising", [2, 0, 0, 0, 1, 0]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
    }
}
fn attacks(g: &mut Game, id: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
}
#[test]
fn fdn_six_attack_qualifications_are_event_only_and_source_exact() {
    for p in [Player::P0, Player::P1] {
        for (name, mana, boost, menace) in [
            ("Courageous Goblin", [1, 0, 0, 1, 0, 0], 1, true),
            ("Ruby, Daring Tracker", [0, 0, 0, 1, 1, 0], 2, false),
        ] {
            for qualifies in [false, true] {
                let mut g = setup(p);
                let large = named(&mut g, p, "Aggressive Mammoth", Zone::Battlefield);
                let source = paid(&mut g, p, name, mana);
                settle(&mut g);
                if name == "Ruby, Daring Tracker" {
                    assert!(g.keywords(source).contains(Keywords::HASTE));
                    assert!(g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==source&&production[3]==1)));
                    assert!(g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==source&&production[4]==1)));
                }
                g.state.objects[source as usize].summoning_sick = false;
                if !qualifies {
                    g.move_object(large, Zone::Exile);
                }
                let base = g.stats(source);
                attacks(&mut g, source);
                if qualifies {
                    g.move_object(large, Zone::Exile);
                }
                g.prepare_triggers();
                // Resolve only the attack trigger before moving into blockers/combat.
                while !g.state.stack.is_empty()
                    || !g.state.pending_triggers.is_empty()
                    || !g.state.ordering_triggers.is_empty()
                {
                    g.prepare_triggers();
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                        .unwrap();
                    checked(&mut g, a);
                }
                assert_eq!(
                    g.stats(source).power,
                    base.power + if qualifies { boost } else { 0 }
                );
                assert_eq!(
                    g.keywords(source).contains(Keywords::MENACE),
                    qualifies && menace
                );
                next_turn(&mut g);
                assert_eq!(g.stats(source).power, base.power);
            }
        }
    }
}
#[test]
fn fdn_six_arbiter_sacrifice_required_paid_order_hidden_choice_and_empty_hand() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = setup(p);
            let arb = named(&mut g, p, "Arbiter of Woe", Zone::Hand);
            g.state.players[p.index()].mana = [4, 0, 2, 0, 0, 0];
            reject(&mut g, Action::AnnounceCast { card: arb });
            let fodder = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let hidden = if empty {
                None
            } else {
                Some(named(&mut g, p.other(), "Giant Growth", Zone::Hand))
            };
            let hand = g.hand_count(p);
            let life = g.state.players[p.index()].life;
            let otherlife = g.state.players[p.other().index()].life;
            checked(&mut g, Action::AnnounceCast { card: arb });
            while g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            for bad in [land, foe, arb] {
                let object = g.object_ref(bad);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            let object = g.object_ref(fodder);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, action);
            assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if let Some(hidden) = hidden {
                assert!(!g.observe(p).objects.iter().any(|o| o.id == hidden));
            }
            settle(&mut g);
            assert_eq!(g.object_view(arb).zone, Zone::Battlefield);
            assert!(g.keywords(arb).contains(Keywords::FLYING));
            assert_eq!(
                g.stats(arb),
                Stats {
                    power: 5,
                    toughness: 4
                }
            );
            assert_eq!(g.state.players[p.index()].life, life + 2);
            assert_eq!(g.state.players[p.other().index()].life, otherlife - 2);
            assert_eq!(g.hand_count(p), hand);
            if let Some(hidden) = hidden {
                assert_eq!(g.object_view(hidden).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn fdn_six_scaled_mana_capacity_failure_is_atomic_in_raw_and_owned_index_actions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let arch = named(&mut g, p, "Elvish Archdruid", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, u32::MAX, 0];
        let actions = g.legal_actions();
        let index = actions
            .iter()
            .position(|a| matches!(a,Action::ActivateMana{card,..}if *card==arch))
            .unwrap();
        let before = g.snapshot();
        let revision = g.revision();
        let views = [
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(g.observe(p.other())).unwrap(),
        ];
        for _ in 0..2 {
            assert!(matches!(
                g.apply(p, actions[index].clone()),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
        }
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        assert!(matches!(
            branch.apply_index(p, revision, index),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(branch.snapshot(), before);
        assert_eq!(
            views,
            [
                serde_json::to_value(g.observe(p)).unwrap(),
                serde_json::to_value(g.observe(p.other())).unwrap()
            ]
        );
    }
}
#[test]
fn fdn_six_attack_trigger_never_modifies_new_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        for name in ["Courageous Goblin", "Ruby, Daring Tracker"] {
            let mut g = setup(p);
            named(&mut g, p, "Aggressive Mammoth", Zone::Battlefield);
            let source = named(&mut g, p, name, Zone::Battlefield);
            let base = g.stats(source);
            attacks(&mut g, source);
            g.prepare_triggers();
            let old = g.object_ref(source);
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Battlefield);
            assert_ne!(g.object_ref(source), old);
            while !g.state.stack.is_empty()
                || !g.state.pending_triggers.is_empty()
                || !g.state.ordering_triggers.is_empty()
            {
                g.prepare_triggers();
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            assert_eq!(g.stats(source), base);
            assert!(!g.keywords(source).contains(Keywords::MENACE));
        }
    }
}
#[test]
fn fdn_six_uprising_entry_threshold_excludes_three_and_opponents_and_survives_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let uprising = paid(&mut g, p, "Garruk's Uprising", [2, 0, 0, 0, 1, 0]);
        settle(&mut g);
        let hand = g.hand_count(p);
        paid(&mut g, p, "Brazen Scourge", [1, 0, 0, 2, 0, 0]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand);
        paid(&mut g, p, "Magnigoth Sentry", [3, 0, 0, 0, 1, 0]);
        super::tests::pass_pair(&mut g);
        g.prepare_triggers();
        g.move_object(uprising, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
    }
}
