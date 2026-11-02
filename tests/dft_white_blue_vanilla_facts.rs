//! Audited immutable absence facts preserve bodies and enable real abilityless searches.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("vanilla-fact resolution ceiling");
}
fn search_ready(g: &mut Game) {
    for _ in 0..32 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseSearchZones { .. }))
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("expected Fang-Druid zone choice");
}

#[test]
fn audited_vanilla_facts_preserve_all_fourteen_paid_bodies_costs_and_empty_rules() {
    for (name, body, cost) in [
        ("Aegis Turtle", (0, 5), 1),
        ("Bear Cub", (2, 2), 2),
        ("Fire Elemental", (5, 4), 5),
        ("Gigantosaurus", (10, 10), 5),
        ("Highborn Vampire", (4, 3), 4),
        ("Ordinary Bear", (4, 5), 4),
        ("Quakestrider Ceratops", (12, 8), 6),
        ("Savannah Lions", (2, 1), 1),
        ("Swab Goblin", (2, 2), 2),
        ("Yargle, Glutton of Urborg", (9, 3), 5),
        ("Yargle, Goliath of Otaria", (3, 9), 5),
        ("Grizzly Bears", (2, 2), 2),
        ("Goblin Piker", (2, 1), 2),
        ("Hill Giant", (3, 3), 4),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, name, Zone::Hand);
        assert!(
            !Card::from_name(name).unwrap().has_rules_abilities(),
            "{name}"
        );
        let before: u32 = g.state.players[0].mana.iter().sum();
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), body, "{name}");
        assert_eq!(
            g.state.players[0].mana.iter().sum::<u32>(),
            before - cost,
            "{name}"
        );
        assert_eq!(g.current_kinds(source), 1, "{name}");
        assert_eq!(g.keywords(source), Keywords::NONE, "{name}");
        assert!(!g.object_has_rules_abilities(source), "{name}");
    }
}

#[test]
fn actual_fang_druid_searches_audited_vanillas_in_either_zone_and_rejects_all_abilities_and_basic_lands()
 {
    for p in [Player::P0, Player::P1] {
        for library in [false, true] {
            for name in ["Aegis Turtle", "Yargle, Goliath of Otaria", "Grizzly Bears"] {
                let mut g = scenario(p);
                let zone = if library {
                    Zone::Library
                } else {
                    Zone::Graveyard
                };
                let found = put(&mut g, p, name, zone);
                let second = put(&mut g, p, "Bear Cub", zone);
                let cycling = put(&mut g, p, "Howler's Heavy", zone);
                let activated = put(&mut g, p, "Daring Mechanic", zone);
                let keyword = put(&mut g, p, "Glitch Ghost Surveyor", zone);
                let land = put(&mut g, p, "Forest", zone);
                assert!(
                    g.object_has_rules_abilities(land),
                    "basic lands have their intrinsic mana ability"
                );
                let before = g.state.players[p.index()].library.clone();
                let source = put(&mut g, p, "Fang-Druid Summoner", Zone::Hand);
                cast(&mut g, source, None);
                search_ready(&mut g);
                act(
                    &mut g,
                    Action::ChooseSearchZones {
                        library,
                        graveyard: !library,
                    },
                );
                for card in [cycling, activated, keyword, land] {
                    reject(&mut g, Action::ChooseSearch { card });
                }
                assert!(
                    g.observe(p.other())
                        .zone_search
                        .as_ref()
                        .unwrap()
                        .cards
                        .is_none()
                );
                act(&mut g, Action::ChooseSearch { card: found });
                reject(&mut g, Action::ChooseSearch { card: second });
                act(&mut g, Action::FinishSearch);
                drain(&mut g);
                assert_eq!(g.object_view(found).zone, Zone::Hand, "{name}");
                assert!(
                    g.observe(p.other())
                        .known_hand
                        .contains(&g.object_ref(found)),
                    "selected card is revealed"
                );
                if !library {
                    assert_eq!(
                        g.state.players[p.index()].library,
                        before,
                        "graveyard-only search does not shuffle"
                    );
                }
            }
        }
    }
}
