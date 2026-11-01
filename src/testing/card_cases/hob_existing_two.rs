//! Focused printed outcomes; shared engine invariants live in core tests.
//! Printed outcomes for the second existing-mechanics HOB cohort.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        if g.legal_actions().contains(&Action::Pass) {
            act(g, Action::Pass)
        } else {
            choose(g, |a| matches!(a, Action::PutTrigger { .. }))
        }
    }
    panic!("decision budget exhausted: {:?}", g.state.pending);
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let old = g.state.players[p.index()].library.to_vec();
    for id in old {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
    g.state.players[p.index()].library.clear();
    let mut ids: Vec<_> = names
        .iter()
        .rev()
        .map(|n| put(g, p, n, Zone::Library))
        .collect();
    ids.reverse();
    ids
}
fn activate(g: &mut Game, id: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
#[test]
fn companies_conditional_keywords_and_bolg_paid_mana() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let b = put(&mut g, p, "Bolg's Company", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 0];
        cast(&mut g, b, None);
        settle(&mut g);
        assert_eq!(stats(&g, b), (2, 2));
        assert!(!g.keywords(b).contains(Keywords::HASTE));
        let enemy = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        assert!(!g.keywords(b).contains(Keywords::HASTE));
        let friend = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        assert!(g.keywords(b).contains(Keywords::HASTE));
        act(
            &mut g,
            Action::AnnounceMana {
                card: b,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 1, 1, 0, 0],
            },
        );
        let own_ref = g.object_ref(b);
        let enemy_ref = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: own_ref });
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
        let fr = g.object_ref(friend);
        act(&mut g, Action::ChooseAdditionalCost { object: fr });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(friend).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 1, 0, 0]);
        assert!(g.object_view(b).tapped);
        assert!(!g.keywords(b).contains(Keywords::HASTE));
    }
}

#[test]
fn thrush_gain_is_unconditional_and_optional_basic_search_puts_top() {
    {
        let p = Player::P0;
        for accept in [false, true] {
            let mut g = scenario(p);
            let top = library(&mut g, p, &["Grizzly Bears", "Mountain", "Forest"]);
            let before = g.state.players[p.index()].library.clone();
            let c = put(&mut g, p, "Old Thrush", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            cast(&mut g, c, None);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
            });
            assert_eq!(g.state.players[p.index()].life, 22);
            act(&mut g, Action::ChooseOptional { accept });
            if accept {
                reject(&mut g, Action::ChooseSearch { card: top[0] });
                act(&mut g, Action::ChooseSearch { card: top[1] });
                if g.legal_actions().contains(&Action::FinishSearch) {
                    act(&mut g, Action::FinishSearch)
                };
            }
            settle(&mut g);

            assert_eq!(stats(&g, c), (1, 2));
            if accept {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&top[1]));
            } else {
                assert_eq!(g.state.players[p.index()].library, before);
            }
        }
    }
}
#[test]
fn keepers_affinity_counts_own_elves_and_returns_all_milled_elves() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let top = library(
            &mut g,
            p,
            &[
                "Llanowar Elves",
                "Forest",
                "Llanowar Elves",
                "Grizzly Bears",
                "Mountain",
            ],
        );
        let c = put(&mut g, p, "Cantankerous Keepers", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        cast(&mut g, c, None);
        settle(&mut g);
        assert_eq!(stats(&g, c), (4, 3));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        for (i, id) in top.iter().enumerate() {
            assert_eq!(
                g.object_view(*id).zone,
                match i {
                    0 | 2 => Zone::Hand,
                    1 | 3 => Zone::Graveyard,
                    _ => Zone::Library,
                }
            );
        }
    }
}

fn step(g: &mut Game, s: Step) {
    g.begin_step(s);
    g.give_priority();
    g.prepare_triggers();
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets)
    };
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(g)
    }
}
#[test]
fn snowslope_pays_another_creature_or_artifact_once_on_own_turn_and_plays_exiled_card() {
    {
        let p = Player::P0;
        for fuel_name in ["Grizzly Bears", "Goldvein Pick"] {
            let mut g = scenario(p);
            let c = put(&mut g, p, "Snowslope Hunter", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
            cast(&mut g, c, None);
            settle(&mut g);
            let top = library(&mut g, p, &["Grizzly Bears", "Forest"]);
            let fuel = put(&mut g, p, fuel_name, Zone::Battlefield);
            let wrong = put(&mut g, p, "Pacifism", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), fuel_name, Zone::Battlefield);
            let start = Action::AnnounceActivate {
                card: c,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.active = p.other();
            reject(&mut g, start.clone());
            g.state.active = p;
            activate(&mut g, c);
            for id in [c, wrong, enemy] {
                let object = g.object_ref(id);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            let object = g.object_ref(fuel);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
            assert_eq!(g.object_view(top[0]).zone, Zone::Exile);
            reject(&mut g, start);
            assert!(g.can_play_exiled(p, top[0]));
            assert!(!g.can_play_exiled(p.other(), top[0]));
            let mut expiration = g.clone();
            expiration.finish_cleanup();
            assert!(expiration.can_play_exiled(p, top[0]));
            expiration.finish_cleanup();
            assert!(expiration.can_play_exiled(p, top[0]));
            expiration.finish_cleanup();
            assert!(!expiration.can_play_exiled(p, top[0]));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            act(&mut g, Action::AnnounceExile { card: top[0] });
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(top[0]).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn goblin_town_saga_reveals_nonland_choice_amasses_and_drains_on_both_final_chapters() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let hand = put(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
        let land = put(&mut g, p.other(), "Forest", Zone::Hand);
        let saga = put(&mut g, p, "Down, Down to Goblin-town", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        cast(&mut g, saga, None);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        bind(&mut g, Target::Player(p.other()));
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseRevealedDiscard { .. }))
        });
        let object = g.object_ref(land);
        reject(&mut g, Action::ChooseRevealedDiscard { object });
        let object = g.object_ref(hand);
        act(&mut g, Action::ChooseRevealedDiscard { object });
        settle(&mut g);
        assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert_eq!(g.counter_count(saga, CounterKind::Lore), 1);
        step(&mut g, Step::Main1);
        settle(&mut g);
        let army = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
            .map(|(i, _)| i as CardId)
            .unwrap();
        assert_eq!(stats(&g, army), (1, 1));
        for chapter in [3, 4] {
            step(&mut g, Step::Main1);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Player(p.other()),
                })
            });
            bind(&mut g, Target::Player(p.other()));
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 20 + chapter - 2);
            assert_eq!(g.state.players[p.other().index()].life, 20 - (chapter - 2));
            assert_eq!(
                g.object_view(saga).zone,
                if chapter == 4 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
