//! Final combinations of independently tested MSH mechanics through paid legal actions.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    let bytes = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn finish(g: &mut Game) {
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unfinished announcement");
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..240 {
        if stop(g) {
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
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::AssignDamage { .. }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("stalled");
}
fn main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
    priority(g, p);
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
fn orphan(g: Game, a: Action) -> Game {
    let p = g.actor().unwrap();
    let i = g.legal_actions().iter().position(|x| *x == a).unwrap();
    owned_checked_step(g, p, i, a)
}
#[test]
fn msh_final_shared_serum_legendary_feeds_adaptoid_and_bypasses_mjolnir_equip() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let turtle = paid(&mut g, p, "Aegis Turtle", &[]);
        let adaptoid = super::msh_super_adaptoid_tests::paid(&mut g, p, Some(turtle));
        assert_eq!(g.object_view(adaptoid).power, 0);
        let serum = paid(&mut g, p, "Super-Soldier Serum", &[Target::Card(turtle)]);
        assert!(g.object_view(turtle).is_legendary());
        assert_eq!(g.object_view(adaptoid).power, 1);
        let hammer = paid(&mut g, p, "Mjölnir, Hammer of Thor", &[]);
        assert_eq!(g.object_view(hammer).attached_to, None);
        let mut probe = g.clone();
        let equip = Action::AnnounceActivate {
            card: hammer,
            ability: 0,
            counter_payment: Counters::default(),
        };
        probe.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        if probe.legal_actions().contains(&equip) {
            checked(&mut probe, equip);
            reject(
                &mut probe,
                Action::ChooseTarget {
                    target: Target::Card(turtle),
                },
            );
        } else {
            reject(&mut probe, equip);
        }
        next_turn(&mut g);
        next_turn(&mut g);
        main(&mut g, p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: turtle });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == serum)
        });
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(hammer),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.object_view(hammer).attached_to,
            Some(g.object_ref(turtle))
        );
        assert_eq!(g.object_view(turtle).colors, 2);
        until(&mut g, |g| {
            g.state.step == Step::EndCombat && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.players[p.other().index()].life, 16);
        paid(&mut g, p, "Disenchant", &[Target::Card(serum)]);
        assert!(!g.object_view(turtle).is_legendary());
        assert_eq!(g.object_view(adaptoid).power, 0);
        assert_eq!(
            g.object_view(hammer).attached_to,
            Some(g.object_ref(turtle))
        );
    }
}
#[test]
fn msh_final_shared_shang_allows_fresh_iron_fist_self_grant_without_attack_haste() {
    for p in [Player::P0, Player::P1] {
        for suppression in [false, true] {
            let mut g = setup(p);
            let fist = paid(&mut g, p, "Iron Fist, Living Weapon", &[]);
            let turtle = paid(&mut g, p, "Aegis Turtle", &[]);
            paid(&mut g, p, "Shock", &[Target::Card(turtle)]);
            let grant = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::AnnounceSelfGrantedAbility { .. }));
            assert!(grant.is_none()); // Summoning sickness blocks the acquired tap ability.
            let timestamp = g
                .state
                .self_granted_abilities
                .as_ref()
                .unwrap()
                .iter()
                .find(|r| r.recipient == g.object_ref(fist))
                .unwrap()
                .timestamp;
            let action = Action::AnnounceSelfGrantedAbility {
                source: g.object_ref(fist),
                grant_timestamp: timestamp,
            };
            reject(&mut g, action.clone());
            let shang = paid(&mut g, p, "Shang-Chi, Master of Kung Fu", &[]);
            assert!(!g.keywords(fist).contains(Keywords::HASTE));
            if suppression {
                let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(shang)]);
                reject(&mut g, action.clone());
                paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
            }
            g.state.players[p.index()].mana = [0; 6];
            g = orphan(g, action);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert!(g.object_view(fist).tapped);
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert!(!g.keywords(fist).contains(Keywords::HASTE));
            // A separate fresh creature still cannot attack while Shang is present.
            let fresh = paid(&mut g, p, "Grizzly Bears", &[]);
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            reject(&mut g, Action::Attack { card: fresh });
            checked(&mut g, Action::FinishAttackers);
        }
    }
}
#[test]
fn msh_final_shared_vision_x_counter_doubling_does_not_increase_crew_mode_budget() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Doubling Season", &[]);
        let bot = named(&mut g, p, "A.I.M. Synthoids", Zone::Library);
        let library_crew = named(&mut g, p, "The Ruinous Wrecking Crew", Zone::Library);
        let quest = named(&mut g, p, "Vision Quest", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 1, 0, 4];
        checked(&mut g, Action::AnnounceCast { card: quest });
        g = orphan(g, Action::ChooseX { value: 4 });
        finish(&mut g);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseSearchZones {
                library: true,
                graveyard: false,
            })
        });
        checked(
            &mut g,
            Action::ChooseSearchZones {
                library: true,
                graveyard: false,
            },
        );
        reject(&mut g, Action::ChooseSearch { card: library_crew }); // Crew is not an artifact.
        checked(&mut g, Action::ChooseSearch { card: bot });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.counter_count(bot, CounterKind::PlusOne), 8);
        assert!(g.keywords(bot).contains(Keywords::HASTE));
        priority(&mut g, p);
        let crew = named(&mut g, p, "The Ruinous Wrecking Crew", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: crew });
        checked(&mut g, Action::ChooseX { value: 1 });
        finish(&mut g);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseMode { mode: 2 })
        });
        assert_eq!(g.counter_count(crew, CounterKind::PlusOne), 2);
        reject(&mut g, Action::ChooseMode { mode: 3 }); // Two modes exceed this spell's captured X=1.
        g = orphan(g, Action::ChooseMode { mode: 2 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.counter_count(bot, CounterKind::PlusOne), 8);
        assert_eq!(g.counter_count(crew, CounterKind::PlusOne), 2);
    }
}
