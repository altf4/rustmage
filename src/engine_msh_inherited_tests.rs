//! Inherited MSH family assurance; no card admissions or shared-rule mutations.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{checked, named, next_turn, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::ChooseActivationCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap_or_else(|| {
                panic!(
                    "payment stalled: {:?} {:?}",
                    g.state.pending,
                    g.legal_actions()
                )
            });
        checked(g, action);
    }
    panic!("unfinished payment");
}
fn cast(g: &mut Game, card: CardId, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card });
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = named(g, p, name, Zone::Hand);
    cast(g, card, &[]);
    settle(g);
    card
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..50 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending);
}
fn activate(g: &mut Game, card: CardId, ability: u16, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn tokens(g: &Game, p: Player, subtype: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            let id = id as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id, subtype))
            .then_some(id)
        })
        .collect()
}
fn visible(g: &Game, viewer: Player, id: CardId) -> bool {
    g.observe(viewer)
        .objects
        .iter()
        .any(|o| o.reference.card == id)
}
fn attackers(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}

fn mana(card: CardId, ability: u16, color: usize, generic: u32) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability,
        payment: [0, 0, 0, 0, 0, generic],
        counter_payment: Counters::default(),
        production,
    }
}
fn source(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, &[]);
    id
}
fn counts(g: &Game, p: Player, name: &str) -> usize {
    tokens(g, p, name).len()
}

#[test]
fn msh_inherited_all_gainlands_entry_controller_departure_and_two_colors() {
    let lands = [
        ("A.I.M. Labs", [1, 2]),
        ("Asgardian Citadel", [3, 0]),
        ("Avengers Hangar", [0, 1]),
        ("Birnin Zana Plaza", [4, 0]),
        ("Fisk Tower", [0, 2]),
        ("Hell's Kitchen", [2, 3]),
        ("Los Diablos Missile Base", [3, 4]),
        ("Pym Technologies", [4, 1]),
        ("Stark Industries", [1, 3]),
        ("Subterranean Cavern", [2, 4]),
    ];
    for p in [Player::P0, Player::P1] {
        for (name, colors) in lands {
            for depart in [false, true] {
                let mut g = setup(p);
                g.state.players[p.index()].mana = [0; 6];
                let id = named(&mut g, p, name, Zone::Hand);
                checked(&mut g, Action::PlayLand { card: id });
                assert!(g.object_view(id).tapped, "{name}");
                assert_eq!(g.life(p), 20);
                reject(&mut g, mana(id, 0, colors[0], 0));
                if depart {
                    g.move_object(id, Zone::Graveyard);
                } else {
                    let object = g.object_ref(id);
                    g.change_controller(object, p.other());
                }
                settle(&mut g);
                assert_eq!(g.life(p), 21, "{name}");
                assert_eq!(g.life(p.other()), 20);
                if depart {
                    continue;
                }
                g.state.priority = p.other();
                g.state.objects[id as usize].tapped = false;
                for color in colors {
                    let mut b = g.clone();
                    checked(&mut b, mana(id, 0, color, 0));
                    let mut expected = [0; 6];
                    expected[color] = 1;
                    assert_eq!(
                        b.state.players[p.other().index()].mana,
                        [
                            30 + expected[0],
                            30 + expected[1],
                            30 + expected[2],
                            30 + expected[3],
                            30 + expected[4],
                            30 + expected[5]
                        ]
                    );
                    assert!(b.object_view(id).tapped);
                    reject(&mut b, mana(id, 0, color, 0));
                }
                let bad = (0..6).find(|c| !colors.contains(c)).unwrap();
                reject(&mut g, mana(id, 0, bad, 0));
            }
        }
    }
}

#[test]
fn msh_inherited_basics_both_seats_play_tap_payment_retap_and_land_limit() {
    for p in [Player::P0, Player::P1] {
        for (name, color) in [
            ("Plains", 0),
            ("Island", 1),
            ("Swamp", 2),
            ("Mountain", 3),
            ("Forest", 4),
        ] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            let id = named(&mut g, p, name, Zone::Hand);
            checked(&mut g, Action::PlayLand { card: id });
            assert!(!g.object_view(id).tapped);
            checked(&mut g, Action::TapMana { card: id });
            let mut expected = [0; 6];
            expected[color] = 1;
            assert_eq!(g.state.players[p.index()].mana, expected);
            reject(&mut g, Action::TapMana { card: id });
            let second = named(&mut g, p, name, Zone::Hand);
            reject(&mut g, Action::PlayLand { card: second });
        }
    }
}

#[test]
fn msh_inherited_landcycling_all_five_paid_private_search_decline_and_reveal() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "A.I.M. Scientists",
            "Borough Backup",
            "Kree Sentinel",
            "Roxxon Brutes",
            "Savage Land Dinosaur",
        ] {
            for decline in [false, true] {
                let mut g = setup(p);
                let id = named(&mut g, p, name, Zone::Hand);
                let found = named(&mut g, p, "Island", Zone::Library);
                let hidden = named(&mut g, p, "Hill Giant", Zone::Library);
                let nonbasic = named(&mut g, p, "A.I.M. Labs", Zone::Library);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
                reject(
                    &mut g,
                    Action::AnnounceActivate {
                        card: id,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
                activate(&mut g, id, 0, None);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectSearch { .. })
                });
                assert!(!visible(&g, p.other(), found));
                assert!(!visible(&g, p.other(), hidden));
                reject(&mut g, Action::ChooseSearch { card: nonbasic });
                if !decline {
                    checked(&mut g, Action::ChooseSearch { card: found });
                }
                let before_shuffle = g.state.rng;
                checked(&mut g, Action::FinishSearch);
                assert_ne!(g.state.rng, before_shuffle);
                settle(&mut g);
                assert_eq!(
                    g.object_view(found).zone,
                    if decline { Zone::Library } else { Zone::Hand }
                );
                assert_eq!(visible(&g, p.other(), found), !decline);
                assert!(!visible(&g, p.other(), hidden));
            }
        }
    }
}

#[test]
fn msh_inherited_entry_connive_three_sources_land_nonland_and_departed_controller() {
    for p in [Player::P0, Player::P1] {
        for name in ["A.I.M. Scientists", "Red Room Recruit", "Madame Masque"] {
            for nonland in [false, true] {
                for depart in [false, true] {
                    let mut g = setup(p);
                    let id = source(&mut g, p, name);
                    until(&mut g, |g| g.object_view(id).zone == Zone::Battlefield);
                    let controller = if depart { p.other() } else { p };
                    if depart {
                        let object = g.object_ref(id);
                        g.change_controller(object, controller);
                        g.move_object(id, Zone::Graveyard);
                    }
                    let discard = named(
                        &mut g,
                        controller,
                        if nonland { "Hill Giant" } else { "Forest" },
                        Zone::Hand,
                    );
                    let secret = named(&mut g, controller, "Aegis Turtle", Zone::Hand);
                    let hand = g.hand_count(controller);
                    until(&mut g, |g| {
                        matches!(g.state.pending, Pending::EffectDiscard { .. })
                    });
                    assert_eq!(g.actor(), Some(controller));
                    assert_eq!(g.hand_count(controller), hand + 1);
                    assert!(!visible(&g, controller.other(), secret));
                    checked(&mut g, Action::Discard { card: discard });
                    settle(&mut g);
                    assert_eq!(g.hand_count(controller), hand);
                    assert_eq!(
                        g.counter_count(id, CounterKind::PlusOne),
                        u64::from(nonland && !depart)
                    );
                    assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
                }
            }
        }
    }
}

#[test]
fn msh_inherited_prowess_noncreature_cast_only_twice_for_thor_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for (name, amount) in [
            ("Agent of Atlas", 1),
            ("Crimson Operative", 1),
            ("Kree Commandos", 1),
            ("Thor Odinson", 2),
        ] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            let base = g.stats(id);
            paid(&mut g, p, "Hill Giant");
            assert_eq!(g.stats(id), base);
            let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, spell, &[Target::Card(own)]);
            settle(&mut g);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: base.power + amount,
                    toughness: base.toughness + amount
                }
            );
            next_turn(&mut g);
            assert_eq!(g.stats(id), base);
            g.state.priority = p.other();
            g.state.players[p.other().index()].mana = [30; 6];
            let spell = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
            cast(&mut g, spell, &[Target::Card(own)]);
            settle(&mut g);
            assert_eq!(g.stats(id), base);
        }
    }
}

#[test]
fn msh_inherited_second_draw_observers_once_per_turn_separate_from_life_gain() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cavalry = paid(&mut g, p, "Atlantean Cavalry");
        let kang = paid(&mut g, p, "Kang, Temporal Tyrant");
        let masque = named(&mut g, p, "Madame Masque", Zone::Battlefield);
        let moon = paid(&mut g, p, "Moon Girl and Devil Dinosaur");
        let tigra = paid(&mut g, p, "Tigra, Feline Fury");
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        paid(&mut g, p, "Divination");
        assert_eq!(g.counter_count(cavalry, CounterKind::PlusOne), 1);
        assert_eq!(
            g.stats(moon),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert!(g.keywords(moon).contains(Keywords::TRAMPLE));
        assert_eq!(counts(&g, p, "Villain"), 1);
        assert_eq!(g.life(p), 21);
        assert_eq!(g.life(p.other()), 19);
        assert_eq!(g.counter_count(tigra, CounterKind::PlusOne), 1);
        paid(&mut g, p, "Divination");
        assert_eq!(g.counter_count(cavalry, CounterKind::PlusOne), 1);
        assert_eq!(counts(&g, p, "Villain"), 1);
        assert_eq!(g.life(p), 21);
        // An opposing draw does not activate any of the controller's observers.
        g.state.active = p.other();
        g.state.priority = p.other();
        paid(&mut g, p.other(), "Divination");
        assert_eq!(g.counter_count(cavalry, CounterKind::PlusOne), 1);
        assert_eq!(g.life(p), 21);
        assert_eq!(g.life(p.other()), 19);
        next_turn(&mut g);
        assert_eq!(
            g.stats(moon),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.keywords(moon).contains(Keywords::TRAMPLE));
        assert_eq!(g.object_view(kang).zone, Zone::Battlefield);
        assert_eq!(g.object_view(masque).zone, Zone::Battlefield);
    }
}

#[test]
fn msh_inherited_artifact_entry_counts_once_per_object_or_once_per_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let machine = paid(&mut g, p, "Machinesmith Automaton");
        let moon = paid(&mut g, p, "Moon Girl and Devil Dinosaur");
        let robot = paid(&mut g, p, "HYDRA Assault Robot");
        assert_eq!(g.counter_count(machine, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let hand = g.hand_count(p);
        let tech = source(&mut g, p, "Stolen Stark Tech");
        // A single artifact that is not a Villain is still one qualifying event.
        for _ in 0..2 {
            until(&mut g, |g| {
                g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseTarget { .. }))
            });
            let action = [
                Action::ChooseTarget {
                    target: Target::Card(machine),
                },
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            ]
            .into_iter()
            .find(|a| g.legal_actions().contains(a))
            .unwrap();
            checked(&mut g, action);
        }
        settle(&mut g);
        assert_eq!(g.life(p.other()), 19);
        assert_eq!(g.hand_count(p), hand); // source() creates then spends its own hand card; Moon already used.
        assert_eq!(g.counter_count(machine, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(tech).attached_to, Some(g.object_ref(machine)));
        assert_eq!(g.object_view(robot).zone, Zone::Battlefield);
        assert_eq!(
            g.stats(moon),
            Stats {
                power: 2,
                toughness: 2
            }
        );
    }
}

#[test]
fn msh_inherited_entry_and_death_tokens_exact_recipe_replacements_and_capture() {
    for p in [Player::P0, Player::P1] {
        for (name, subtype, amount, power, toughness, color, keyword) in [
            ("Borough Backup", "Hero", 2, 3, 2, 1, Keywords::VIGILANCE),
            (
                "S.H.I.E.L.D. Deployment Drone",
                "Soldier",
                1,
                1,
                1,
                1,
                Keywords::NONE,
            ),
            (
                "S.H.I.E.L.D. Helicarrier",
                "Soldier",
                2,
                1,
                1,
                1,
                Keywords::NONE,
            ),
            (
                "Okoye, Dora Milaje Leader",
                "Soldier",
                2,
                1,
                1,
                1,
                Keywords::NONE,
            ),
            (
                "The Unbeatable Squirrel Girl",
                "Squirrel",
                1,
                1,
                1,
                16,
                Keywords::NONE,
            ),
            ("Agents of HYDRA", "Villain", 1, 2, 1, 4, Keywords::MENACE),
        ] {
            for doubled in [false, true] {
                let mut g = setup(p);
                if doubled {
                    named(&mut g, p, "Doubling Season", Zone::Battlefield);
                }
                let id = source(&mut g, p, name);
                if name == "Agents of HYDRA" {
                    settle(&mut g);
                    g.move_object(id, Zone::Graveyard);
                } else if name != "Borough Backup" {
                    until(&mut g, |g| g.object_view(id).zone == Zone::Battlefield);
                    g.move_object(id, Zone::Graveyard);
                }
                settle(&mut g);
                let result = tokens(&g, p, subtype);
                assert_eq!(result.len(), amount * if doubled { 2 } else { 1 }, "{name}");
                for token in result {
                    assert_eq!(g.stats(token), Stats { power, toughness });
                    assert_eq!(g.object_view(token).colors, color);
                    assert!(g.keywords(token).contains(keyword));
                }
                assert_eq!(counts(&g, p.other(), subtype), 0);
            }
        }
    }
}

#[test]
fn msh_inherited_squirrel_scaled_ability_counts_live_controller_and_self_and_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "The Unbeatable Squirrel Girl");
        assert_eq!(counts(&g, p, "Squirrel"), 1);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 3, 1];
        activate(&mut g, id, 0, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(counts(&g, p, "Squirrel"), 3); // The legendary Squirrel itself counts.
        g.state.objects[id as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(counts(&g, p, "Squirrel"), 4);
    }
}

#[test]
fn msh_inherited_simple_growth_spells_exact_target_incarnation_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for (name, dp, dt) in [
            ("Giant Growth", 3, 3),
            ("Dark Deed", -4, -4),
            ("Blazing Crescendo", 3, 1),
        ] {
            for blink in [false, true] {
                let mut g = setup(p);
                let victim = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
                let land = named(&mut g, p, "Forest", Zone::Battlefield);
                let id = named(&mut g, p, name, Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: id });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
                finish(&mut g);
                if blink {
                    g.move_object(victim, Zone::Hand);
                    g.move_object(victim, Zone::Battlefield);
                }
                let top = *g.state.players[p.index()].library.last().unwrap();
                settle(&mut g);
                assert_eq!(
                    g.stats(victim),
                    if blink {
                        Stats {
                            power: 0,
                            toughness: 5,
                        }
                    } else {
                        Stats {
                            power: dp,
                            toughness: 5 + dt,
                        }
                    }
                );
                if name == "Blazing Crescendo" {
                    assert_eq!(
                        g.object_view(top).zone,
                        if blink { Zone::Library } else { Zone::Exile }
                    );
                }
                next_turn(&mut g);
                assert_eq!(
                    g.stats(victim),
                    Stats {
                        power: 0,
                        toughness: 5
                    }
                );
            }
        }
    }
}

#[test]
fn msh_inherited_lightning_any_creature_or_player_and_stale_target() {
    for p in [Player::P0, Player::P1] {
        for choice in 0..3 {
            let mut g = setup(p);
            let victim = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            let id = named(&mut g, p, "Lightning Strike", Zone::Hand);
            let target = if choice == 0 {
                Target::Player(p.other())
            } else {
                Target::Card(victim)
            };
            cast(&mut g, id, &[target]);
            if choice == 2 {
                g.move_object(victim, Zone::Hand);
                g.move_object(victim, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.life(p.other()), if choice == 0 { 17 } else { 20 });
            assert_eq!(
                g.object_view(victim).damage,
                if choice == 1 { 3 } else { 0 }
            );
        }
    }
}

#[test]
fn msh_inherited_pym_particles_vigilant_unblockable_draw_privacy_and_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let top = *g.state.players[p.index()].library.last().unwrap();
        let hand = g.hand_count(p);
        let spell = named(&mut g, p, "Pym Particles", Zone::Hand);
        cast(&mut g, spell, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(!visible(&g, p.other(), top));
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: target });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(target).tapped);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: target,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        settle(&mut g);
        next_turn(&mut g);
        assert!(!g.keywords(target).contains(Keywords::VIGILANCE));
        assert!(!g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
    }
}

#[test]
fn msh_inherited_take_up_shield_counter_replacement_indestructible_lifelink_then_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spell = named(&mut g, p, "Take Up the Shield", Zone::Hand);
        cast(&mut g, spell, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [30; 6];
        let destroy = named(&mut g, p.other(), "Murder", Zone::Hand);
        cast(&mut g, destroy, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: target });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        combat_end(&mut g);
        assert_eq!(g.life(p), 25);
        assert_eq!(g.life(p.other()), 15);
        next_turn(&mut g);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
        assert!(!g
            .keywords(target)
            .contains(Keywords::INDESTRUCTIBLE.union(Keywords::LIFELINK)));
    }
}

#[test]
fn msh_inherited_visions_villain_discount_exact_mana_private_draw_and_life() {
    for p in [Player::P0, Player::P1] {
        for villain in [false, true] {
            let mut g = setup(p);
            if villain {
                named(&mut g, p, "Agents of HYDRA", Zone::Battlefield);
            } else {
                named(&mut g, p.other(), "Agents of HYDRA", Zone::Battlefield);
            }
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, if villain { 1 } else { 2 }];
            let hand = g.hand_count(p);
            let top = *g.state.players[p.index()].library.last().unwrap();
            paid(&mut g, p, "Visions of Villainy");
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.hand_count(p), hand + 2);
            assert_eq!(g.life(p), 18);
            assert!(!visible(&g, p.other(), top));
        }
    }
}

#[test]
fn msh_inherited_futurist_entry_and_paid_sacrifice_draw_capture_and_privacy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hand = g.hand_count(p);
        let top = *g.state.players[p.index()].library.last().unwrap();
        let id = paid(&mut g, p, "Futurist Forge");
        assert_eq!(g.hand_count(p), hand + 1);
        assert!(!visible(&g, p.other(), top));
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        activate(&mut g, id, 0, None);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), hand + 3);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}

#[test]
fn msh_inherited_deathlok_graveyard_payment_ownership_and_stale_source() {
    for p in [Player::P0, Player::P1] {
        for moved in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Project Deathlok Soldier");
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.move_object(id, Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
            activate(&mut g, id, 0, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if moved {
                g.move_object(id, Zone::Exile);
                g.move_object(id, Zone::Graveyard);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(id).zone,
                if moved { Zone::Graveyard } else { Zone::Hand }
            );
            assert!(visible(&g, p.other(), id));
        }
    }
}

#[test]
fn msh_inherited_executive_paid_tap_treasure_any_color_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Stark Industries Executive");
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            ); // summoning sick
            g.state.objects[id as usize].summoning_sick = false;
            activate(&mut g, id, 0, None);
            assert!(g.object_view(id).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.move_object(id, Zone::Graveyard);
            settle(&mut g);
            let treasure = tokens(&g, p, "Treasure");
            assert_eq!(treasure.len(), 1);
            assert_eq!(g.current_kinds(treasure[0]), 16);
            assert_eq!(g.object_view(treasure[0]).colors, 0);
            checked(&mut g, mana(treasure[0], 0, color, 0));
            assert_eq!(g.state.players[p.index()].mana[color], 1);
            assert_eq!(g.object_view(treasure[0]).zone, Zone::Outside);
        }
    }
}

#[test]
fn msh_inherited_coulson_other_own_heroes_counter_replacement_and_vigilant_attack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Agent Phil Coulson");
        let own = named(&mut g, p, "Agent of Atlas", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Agent of Atlas", Zone::Battlefield);
        let villain = named(&mut g, p, "Agents of HYDRA", Zone::Battlefield);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        g.state.objects[id as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(id).tapped);
        until(&mut g, |g| g.state.pending == Pending::Priority);
        activate(&mut g, id, 0, None);
        settle(&mut g);
        assert!(g.object_view(id).tapped);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 2);
        for excluded in [id, enemy, villain] {
            assert_eq!(g.counter_count(excluded, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn msh_inherited_patriot_paid_other_own_target_hexproof_controller_capture_and_expiry() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Patriot, Shield Wielder");
            let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = false;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            for bad in [id, enemy] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bad),
                    },
                );
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.object_view(id).tapped);
            g.move_object(id, Zone::Graveyard);
            if blink {
                g.move_object(own, Zone::Hand);
                g.move_object(own, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.stats(own).power, if blink { 3 } else { 5 });
            assert_eq!(g.keywords(own).contains(Keywords::HEXPROOF), !blink);
            if !blink {
                g.state.priority = p.other();
                let spell = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: spell });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                finish(&mut g);
                settle(&mut g);
            }
            next_turn(&mut g);
            assert_eq!(g.stats(own).power, 3);
            assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
        }
    }
}

#[test]
fn msh_inherited_misty_discard_cost_counts_itself_and_prior_discards_and_hidden_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Misty Knight, Hero for Hire");
        g.state.objects[id as usize].summoning_sick = false;
        for count in 1..=2 {
            let discard = named(&mut g, p, "Hill Giant", Zone::Hand);
            let secret = named(&mut g, p, "Aegis Turtle", Zone::Hand);
            let foreign = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let bad = g.object_ref(foreign);
            reject(&mut g, Action::ChooseAdditionalCost { object: bad });
            let object = g.object_ref(discard);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish(&mut g);
            assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let before = g.hand_count(p);
            settle(&mut g);
            assert_eq!(g.hand_count(p), before + count);
            assert!(!visible(&g, p.other(), secret));
            g.state.objects[id as usize].tapped = false; // Fixture for an independent untap between repeat activations.
        }
    }
}

#[test]
fn msh_inherited_kingpin_sacrifice_artifact_creature_or_self_cost_then_private_draw() {
    for p in [Player::P0, Player::P1] {
        for self_cost in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Kingpin's Enforcers");
            let artifact = named(&mut g, p, "Futurist Forge", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let foreign = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            for bad in [land, foreign] {
                let object = g.object_ref(bad);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            let cost = if self_cost { id } else { artifact };
            let object = g.object_ref(cost);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish(&mut g);
            assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let before = g.hand_count(p);
            let top = *g.state.players[p.index()].library.last().unwrap();
            settle(&mut g);
            assert_eq!(g.hand_count(p), before + 1);
            assert!(!visible(&g, p.other(), top));
        }
    }
}

#[test]
fn msh_inherited_gorilla_sorcery_sacrifice_rejects_creature_artifacts_and_stale_target() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Guerrilla Gorilla");
            let victim = named(&mut g, p.other(), "Futurist Forge", Zone::Battlefield);
            let creature = named(&mut g, p.other(), "Kree Sentinel", Zone::Battlefield);
            g.state.active = p.other();
            g.state.priority = p;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.active = p;
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            finish(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            if blink {
                g.move_object(victim, Zone::Hand);
                g.move_object(victim, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if blink {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn msh_inherited_equipment_paid_attach_retarget_bonus_own_target_and_sorcery() {
    for p in [Player::P0, Player::P1] {
        for (name, dp, dt, blue, generic, entry) in [
            ("Captain America's Shield", 0, 8, 0, 2, false),
            ("Falcon's Wing Harness", 1, 1, 1, 2, true),
            ("Stolen Stark Tech", 1, 0, 0, 1, true),
            ("Vibranium Energy Daggers", 2, 2, 0, 3, false),
        ] {
            let mut g = setup(p);
            let first = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let second = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            if name == "Stolen Stark Tech" {
                g.state.active = p.other();
                g.state.priority = p;
            }
            let id = source(&mut g, p, name);
            if entry {
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(first),
                    })
                });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(first),
                    },
                );
                finish(&mut g);
            }
            settle(&mut g);
            if name == "Falcon's Wing Harness" {
                assert!(g.keywords(first).contains(Keywords::FLYING));
            }
            if name == "Stolen Stark Tech" {
                assert!(g.keywords(first).contains(Keywords::INDESTRUCTIBLE));
                g.state.priority = p;
                let destroy = named(&mut g, p, "Murder", Zone::Hand);
                cast(&mut g, destroy, &[Target::Card(first)]);
                settle(&mut g);
                assert_eq!(g.object_view(first).zone, Zone::Battlefield);
            }
            let activation = Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            };
            g.state.active = p.other();
            g.state.priority = p;
            reject(&mut g, activation.clone());
            g.state.active = p;
            for target in [first, second] {
                g.state.players[p.index()].mana = [0, blue, 0, 0, 0, generic];
                checked(&mut g, activation.clone());
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                finish(&mut g);
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(
                    g.stats(target),
                    Stats {
                        power: 3 + dp,
                        toughness: 3 + dt
                    }
                );
            }
            assert_eq!(
                g.stats(first),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            let object = g.object_ref(second);
            g.change_controller(object, p.other());
            assert_eq!(
                g.stats(second),
                Stats {
                    power: 3 + dp,
                    toughness: 3 + dt
                }
            );
            g.move_object(id, Zone::Exile);
            assert_eq!(
                g.stats(second),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
        }
    }
}

#[test]
fn msh_inherited_indestructible_equipment_survives_paid_destroy() {
    for p in [Player::P0, Player::P1] {
        for name in ["Captain America's Shield", "Vibranium Energy Daggers"] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            let spell = named(&mut g, p, "Disenchant", Zone::Hand);
            cast(&mut g, spell, &[Target::Card(id)]);
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn msh_inherited_shield_attack_defending_target_vigilance_and_departed_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let id = paid(&mut g, p, "Captain America's Shield");
        activate(&mut g, id, 0, Some(Target::Card(own)));
        settle(&mut g);
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: own });
        checked(&mut g, Action::FinishAttackers);
        assert!(!g.object_view(own).tapped);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        g.move_object(id, Zone::Exile);
        settle(&mut g);
        assert!(g.object_view(target).tapped);
    }
}

#[test]
fn msh_inherited_attached_ward_paid_or_declined_by_opponent_after_grantor_departure() {
    for p in [Player::P0, Player::P1] {
        for name in ["Falcon's Wing Harness", "Super Strength"] {
            for pay in [false, true] {
                let mut g = setup(p);
                let creature = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let grantor = named(&mut g, p, name, Zone::Hand);
                if name == "Super Strength" {
                    cast(&mut g, grantor, &[Target::Card(creature)]);
                } else {
                    cast(&mut g, grantor, &[]);
                    until(&mut g, |g| {
                        g.legal_actions().contains(&Action::ChooseTarget {
                            target: Target::Card(creature),
                        })
                    });
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(creature),
                        },
                    );
                    finish(&mut g);
                }
                settle(&mut g);
                g.state.priority = p.other();
                let spell = named(&mut g, p.other(), "Murder", Zone::Hand);
                cast(&mut g, spell, &[Target::Card(creature)]);
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::DeclineResolutionMana)
                });
                g.move_object(grantor, Zone::Exile);
                assert_eq!(g.actor(), Some(p.other()));
                if pay {
                    checked(
                        &mut g,
                        Action::PayResolutionMana {
                            payment: [0, 0, 0, 0, 0, 1],
                        },
                    );
                } else {
                    checked(&mut g, Action::DeclineResolutionMana);
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(creature).zone,
                    if pay {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
        }
    }
}

#[test]
fn msh_inherited_aura_speed_flash_haste_first_strike_expiry_and_frozen_all_untaps() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p, "Kree Commandos", Zone::Battlefield);
        g.state.objects[creature as usize].summoning_sick = true;
        g.state.active = p.other();
        g.state.priority = p;
        let speed = named(&mut g, p, "Super Speed", Zone::Hand);
        cast(&mut g, speed, &[Target::Card(creature)]);
        settle(&mut g);
        assert_eq!(g.stats(creature).power, 4); // Super Speed also triggers prowess.
        assert!(g
            .keywords(creature)
            .contains(Keywords::HASTE.union(Keywords::FIRST_STRIKE)));
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: creature });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        next_turn(&mut g);
        assert!(g.keywords(creature).contains(Keywords::HASTE));
        assert!(!g.keywords(creature).contains(Keywords::FIRST_STRIKE));
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [30; 6];
        let frozen = named(&mut g, p, "Frozen in Ice", Zone::Hand);
        cast(&mut g, frozen, &[Target::Card(creature)]);
        settle(&mut g);
        assert!(g.object_view(creature).tapped);
        assert_eq!(g.keywords(creature), Keywords::NONE);
        let pounce = named(&mut g, p, "Panther Pounce", Zone::Hand);
        cast(&mut g, pounce, &[Target::Player(p), Target::Card(creature)]);
        settle(&mut g);
        assert!(g.object_view(creature).tapped);
        g.move_object(frozen, Zone::Graveyard);
        let pounce = named(&mut g, p, "Panther Pounce", Zone::Hand);
        cast(&mut g, pounce, &[Target::Player(p), Target::Card(creature)]);
        settle(&mut g);
        assert!(!g.object_view(creature).tapped);
        g.move_object(speed, Zone::Graveyard);
        assert!(!g.keywords(creature).contains(Keywords::HASTE));
    }
}

#[test]
fn msh_inherited_exile_enchantments_restrictions_return_and_departed_before_resolution() {
    for p in [Player::P0, Player::P1] {
        for name in ["Web Up", "Super Villain Lockup"] {
            for early in [false, true] {
                let mut g = setup(p);
                let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                g.state.objects[target as usize].tapped = true;
                let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
                if name == "Super Villain Lockup" {
                    g.state.active = p.other();
                    g.state.priority = p;
                }
                let id = source(&mut g, p, name);
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Card(target),
                    })
                });
                let untapped = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                if name == "Super Villain Lockup" {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(untapped),
                        },
                    );
                }
                for bad in [own, land] {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(bad),
                        },
                    );
                }
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                finish(&mut g);
                if early {
                    g.move_object(id, Zone::Graveyard);
                }
                let old = g.object_ref(target);
                settle(&mut g);
                assert_eq!(
                    g.object_view(target).zone,
                    if early {
                        Zone::Battlefield
                    } else {
                        Zone::Exile
                    }
                );
                if !early {
                    g.move_object(id, Zone::Graveyard);
                    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                    assert_ne!(g.object_ref(target), old);
                    assert!(!g.object_view(target).tapped);
                }
            }
        }
    }
}

#[test]
fn msh_inherited_private_scry_surveil_order_and_public_graveyard() {
    for p in [Player::P0, Player::P1] {
        for (name, surveil, amount) in [
            ("A.I.M. Synthoids", true, 2),
            ("Wakandan Drone Flock", false, 2),
            ("Surveillance Room", true, 1),
            ("Hour of Defeat", true, 1),
        ] {
            let mut g = setup(p);
            let low = named(&mut g, p, "Island", Zone::Library);
            let top = named(&mut g, p, "Hill Giant", Zone::Library);
            let id = named(&mut g, p, name, Zone::Hand);
            if name == "Surveillance Room" {
                checked(&mut g, Action::PlayLand { card: id });
            } else if name == "Hour of Defeat" {
                let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                cast(&mut g, id, &[Target::Card(victim)]);
                until(&mut g, |g| g.observe(p).library_choice.is_some());
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            } else {
                cast(&mut g, id, &[]);
            }
            until(&mut g, |g| g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            assert!(!visible(&g, p.other(), top));
            let view = g.observe(p).library_choice.unwrap();
            assert_eq!(view.remaining.len(), amount);
            assert_eq!(view.surveil, surveil);
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: if surveil {
                        LibraryDestination::Graveyard
                    } else {
                        LibraryDestination::Bottom
                    },
                },
            );
            if amount == 2 {
                checked(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: low,
                        destination: LibraryDestination::Top,
                    },
                );
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(top).zone,
                if surveil {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            assert_eq!(visible(&g, p.other(), top), surveil);
            if amount == 2 {
                assert_eq!(g.state.players[p.index()].library.last(), Some(&low));
            }
        }
    }
}

#[test]
fn msh_inherited_surveillance_room_filter_mana_all_colors_and_colorless() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Surveillance Room", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, mana(id, 0, 5, 0));
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        for color in 0..5 {
            let mut b = g.clone();
            b.state.objects[id as usize].tapped = false;
            checked(&mut b, mana(id, 1, color, 1));
            assert_eq!(b.state.players[p.index()].mana[color], 1);
            assert_eq!(b.state.players[p.index()].mana[5], 0);
            assert!(b.object_view(id).tapped);
        }
    }
}

#[test]
fn msh_inherited_herbie_draw_then_private_optional_tapped_land_ignores_land_limit() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            g.state.land_played = true;
            let land = named(&mut g, p, "Island", Zone::Hand);
            let wrong = named(&mut g, p, "Hill Giant", Zone::Hand);
            let foreign = named(&mut g, p.other(), "Forest", Zone::Hand);
            let hand = g.hand_count(p);
            source(&mut g, p, "H.E.R.B.I.E. Scout Unit");
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::DeclineHandEntry)
            });
            assert_eq!(g.hand_count(p), hand + 1);
            for bad in [wrong, foreign] {
                let object = g.object_ref(bad);
                reject(&mut g, Action::PutHandLand { object });
            }
            assert!(!visible(&g, p.other(), wrong));
            if accept {
                let object = g.object_ref(land);
                checked(&mut g, Action::PutHandLand { object });
            } else {
                checked(&mut g, Action::DeclineHandEntry);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(land).zone,
                if accept {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
            if accept {
                assert!(g.object_view(land).tapped);
            }
        }
    }
}

#[test]
fn msh_inherited_hero_in_training_checks_other_hero_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for other in [false, true] {
            let mut g = setup(p);
            if other {
                named(&mut g, p, "Agent of Atlas", Zone::Battlefield);
            } else {
                named(&mut g, p.other(), "Agent of Atlas", Zone::Battlefield);
            }
            let hand = g.hand_count(p);
            paid(&mut g, p, "Hero in Training");
            assert_eq!(g.hand_count(p), hand + 1);
            assert_eq!(g.life(p), if other { 22 } else { 20 });
        }
    }
}

#[test]
fn msh_inherited_wakandan_guard_self_other_hero_any_controller_and_stale_target() {
    for p in [Player::P0, Player::P1] {
        for selection in 0..4 {
            let mut g = setup(p);
            let hero = named(&mut g, p.other(), "Agent of Atlas", Zone::Battlefield);
            let other = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let id = source(&mut g, p, "Wakandan Royal Guard");
            let target = if selection == 0 {
                id
            } else if selection == 1 {
                other
            } else {
                hero
            };
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(target),
                })
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            if selection == 3 {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(target, CounterKind::PlusOne),
                if selection == 3 {
                    0
                } else if selection == 2 {
                    2
                } else {
                    1
                }
            );
        }
    }
}

#[test]
fn msh_inherited_white_widow_both_modes_zero_two_targets_and_public_recovery() {
    for p in [Player::P0, Player::P1] {
        for selection in 0..4 {
            let mut g = setup(p);
            let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let recover = named(&mut g, p, "Futurist Forge", Zone::Graveyard);
            let foreign = named(&mut g, p.other(), "Futurist Forge", Zone::Graveyard);
            let wrong = named(&mut g, p, "Hill Giant", Zone::Graveyard);
            source(&mut g, p, "White Widow, Free Agent");
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
            });
            checked(
                &mut g,
                Action::ChooseMode {
                    mode: if selection == 3 { 1 } else { 0 },
                },
            );
            if selection == 3 {
                for bad in [foreign, wrong] {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(bad),
                        },
                    );
                }
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(recover),
                    },
                );
            } else if selection > 0 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
                if selection == 2 {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(b),
                        },
                    );
                }
            }
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.counter_count(a, CounterKind::PlusOne),
                u64::from(selection == 1 || selection == 2)
            );
            assert_eq!(
                g.counter_count(b, CounterKind::PlusOne),
                u64::from(selection == 2)
            );
            assert_eq!(
                g.object_view(recover).zone,
                if selection == 3 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert!(visible(&g, p.other(), recover));
        }
    }
}

#[test]
fn msh_inherited_black_panther_modes_exclude_self_tokens_enemy_and_snapshot_boost() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "Black Panther, Vanguard");
            assert_eq!(counts(&g, p, "Soldier"), 0);
            paid(&mut g, p, "Borough Backup"); // Hero tokens do not qualify.
            paid(&mut g, p, "Hill Giant"); // A nontoken non-Hero does not qualify.
            g.state.active = p.other();
            g.state.priority = p.other();
            paid(&mut g, p.other(), "Agent of Atlas"); // Opposing Hero does not qualify.
            assert_eq!(counts(&g, p, "Soldier"), 0);
            g.state.active = p;
            g.state.priority = p;

            let hero = source(&mut g, p, "Agent of Atlas");
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseMode { mode })
            });
            checked(&mut g, Action::ChooseMode { mode });
            settle(&mut g);
            if mode == 0 {
                assert_eq!(counts(&g, p, "Soldier"), 1);
            } else {
                assert_eq!(g.stats(id).power, 5);
                assert_eq!(g.stats(hero).power, 3);
                let late = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                assert_eq!(g.stats(late).power, 3);
                next_turn(&mut g);
                assert_eq!(g.stats(id).power, 4);
                assert_eq!(g.stats(hero).power, 2);
            }
        }
    }
}

#[test]
fn msh_inherited_colleen_mockingbird_target_cast_filter_one_trigger_and_scry_privacy() {
    for p in [Player::P0, Player::P1] {
        for own in [false, true] {
            let mut g = setup(p);
            let colleen = paid(&mut g, p, "Colleen Wing, Street Samurai");
            let bird = paid(&mut g, p, "Mockingbird, Ace Agent");
            let victim = named(
                &mut g,
                if own { p } else { p.other() },
                "Hill Giant",
                Zone::Battlefield,
            );
            let top = named(&mut g, p, "Island", Zone::Library);
            let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, spell, &[Target::Card(victim)]);
            if own {
                until(&mut g, |g| g.observe(p).library_choice.is_some());
                assert!(g.observe(p.other()).library_choice.is_none());
                checked(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: top,
                        destination: LibraryDestination::Bottom,
                    },
                );
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(colleen, CounterKind::PlusOne),
                u64::from(own)
            );
            assert_eq!(g.counter_count(bird, CounterKind::PlusOne), u64::from(own));
        }
    }
}

#[test]
fn msh_inherited_madame_hydra_cast_villain_yellowjacket_entry_distinct_and_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let madame = paid(&mut g, p, "Madame Hydra");
        let jacket = paid(&mut g, p, "Yellowjacket, Heartless Marauder");
        assert_eq!(counts(&g, p, "Villain"), 1); // Madame observes the cast; the token and Yellowjacket each enter.
        assert_eq!(g.stats(jacket).power, 1);
        paid(&mut g, p, "Agents of HYDRA");
        assert_eq!(counts(&g, p, "Villain"), 2);
        assert_eq!(g.stats(jacket).power, 3);
        assert!(g.keywords(jacket).contains(Keywords::LIFELINK));
        g.move_object(madame, Zone::Graveyard);
        let before = counts(&g, p, "Villain");
        let id = paid(&mut g, p, "Agents of HYDRA");
        assert_eq!(counts(&g, p, "Villain"), before);
        assert_eq!(g.stats(jacket).power, 4);
        g.move_object(id, Zone::Exile);
        next_turn(&mut g);
        assert_eq!(g.stats(jacket).power, 1);
        assert!(!g.keywords(jacket).contains(Keywords::LIFELINK));
    }
}

#[test]
fn msh_inherited_decoy_all_modes_partial_targets_and_dual_typed_same_card() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for blink in [false, true] {
                let mut g = setup(p);
                let villain = named(&mut g, p, "Agents of HYDRA", Zone::Graveyard);
                let hero = named(&mut g, p, "Agent of Atlas", Zone::Graveyard);
                let wrong = named(&mut g, p.other(), "Agent of Atlas", Zone::Graveyard);
                let spell = named(&mut g, p, "Decoy Ploy", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: spell });
                checked(&mut g, Action::ChooseMode { mode });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(wrong),
                    },
                );
                if mode != 1 {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(villain),
                        },
                    );
                }
                if mode != 0 {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(hero),
                        },
                    );
                }
                finish(&mut g);
                if blink {
                    g.move_object(villain, Zone::Exile);
                    g.move_object(villain, Zone::Graveyard);
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(villain).zone,
                    if mode != 1 && !blink {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.object_view(hero).zone,
                    if mode != 0 {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
            }
        }
    }
}

#[test]
fn msh_inherited_epic_fight_each_mode_doubles_before_fighting_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = named(&mut g, p, "Epic Fight", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseMode { mode });
            if mode != 1 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
            }
            if mode != 0 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.object_view(enemy).zone,
                if mode == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(own).zone,
                if mode == 1 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if mode != 1 {
                assert_eq!(g.stats(own).power, 6);
                next_turn(&mut g);
                assert_eq!(g.stats(own).power, 3);
            }
        }
    }
}

#[test]
fn msh_inherited_flying_ant_flash_modes_nonland_target_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            for blink in [false, true] {
                let mut g = setup(p);
                let target = named(&mut g, p.other(), "Futurist Forge", Zone::Battlefield);
                g.state.objects[target as usize].tapped = mode == 1;
                let land = named(&mut g, p, "Forest", Zone::Battlefield);
                g.state.active = p.other();
                g.state.priority = p;
                let id = source(&mut g, p, "Giant-Sized Flying Ant");
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseMode { mode })
                });
                checked(&mut g, Action::ChooseMode { mode });
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                finish(&mut g);
                if blink {
                    g.move_object(target, Zone::Hand);
                    g.move_object(target, Zone::Battlefield);
                    g.state.objects[target as usize].tapped = mode == 1;
                }
                g.move_object(id, Zone::Graveyard);
                settle(&mut g);
                assert_eq!(
                    g.object_view(target).tapped,
                    if blink { mode == 1 } else { mode == 0 }
                );
            }
        }
    }
}

#[test]
fn msh_inherited_serpent_ward_poison_choice_decline_and_ten_poison_loss() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "The Serpent Society");
            g.state.priority = p.other();
            let spell = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
            cast(&mut g, spell, &[Target::Card(id)]);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
            });
            assert_eq!(g.actor(), Some(p.other()));
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].poison_counters,
                if accept { 5 } else { 0 }
            );
            assert_eq!(g.stats(id).power, if accept { 6 } else { 3 });
            if accept {
                g.state.priority = p.other();
                let spell = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
                cast(&mut g, spell, &[Target::Card(id)]);
                until(&mut g, |g| {
                    g.legal_actions()
                        .contains(&Action::ChooseOptional { accept: true })
                });
                checked(&mut g, Action::ChooseOptional { accept: true });
                settle(&mut g);
                assert_eq!(g.outcome(), Some(Outcome::Winner(p)));
            }
        }
    }
}

#[test]
fn msh_inherited_serpent_other_deathtouch_death_opponent_nontoken_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "The Serpent Society");
        let dead = paid(&mut g, p, "Thornweald Archer");
        let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let token = super::assurance::big_helpers::spawn(
            &mut g,
            p.other(),
            Card::from_name("@token/dba7f52527d069d0ccdbfd5e").unwrap(),
            Zone::Battlefield,
        );
        let spell = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, spell, &[Target::Card(dead)]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::Sacrifice {
                object: g.object_ref(victim),
            })
        });
        assert_eq!(g.actor(), Some(p.other()));
        let object = g.object_ref(token);
        reject(&mut g, Action::Sacrifice { object });
        let object = g.object_ref(victim);
        checked(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.object_view(token).zone, Zone::Battlefield);
    }
}

#[test]
fn msh_inherited_scarlet_live_power_only_large_instants_sorceries_and_control() {
    for p in [Player::P0, Player::P1] {
        for growth in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p, "The Scarlet Witch");
            if growth {
                let spell = named(&mut g, p, "Giant Growth", Zone::Hand);
                cast(&mut g, spell, &[Target::Card(id)]);
                settle(&mut g);
            }
            let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, if growth { 0 } else { 1 }];
            let spell = named(&mut g, p, "Hour of Defeat", Zone::Hand);
            cast(&mut g, spell, &[Target::Card(victim)]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            let creature = named(&mut g, p, "Hill Giant", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            reject(&mut g, Action::AnnounceCast { card: creature });
            let low = named(&mut g, p, "Divination", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            reject(&mut g, Action::AnnounceCast { card: low });
            let object = g.object_ref(id);
            g.change_controller(object, p.other());
            let spell = named(&mut g, p, "Hour of Defeat", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            reject(&mut g, Action::AnnounceCast { card: spell });
        }
    }
}

#[test]
fn msh_inherited_vehicles_crew_exact_power_sick_crewers_mana_and_expiry() {
    for p in [Player::P0, Player::P1] {
        for (name, ability, minimum, power, toughness) in [
            ("Dependable Quinjet", 1, 4, 3, 3),
            ("S.H.I.E.L.D. Helicarrier", 0, 6, 4, 5),
        ] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            let a = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let b = named(
                &mut g,
                p,
                if minimum == 4 {
                    "Llanowar Elves"
                } else {
                    "Hill Giant"
                },
                Zone::Battlefield,
            );
            let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[a as usize].summoning_sick = true;
            g.state.objects[b as usize].summoning_sick = true;
            assert_eq!(g.current_kinds(id) & 1, 0);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability,
                    counter_payment: Counters::default(),
                },
            );
            let object = g.object_ref(enemy);
            reject(&mut g, Action::ChooseCrewer { object });
            reject(&mut g, Action::FinishCrew);
            let object = g.object_ref(a);
            checked(&mut g, Action::ChooseCrewer { object });
            reject(&mut g, Action::FinishCrew);
            let object = g.object_ref(b);
            checked(&mut g, Action::ChooseCrewer { object });
            checked(&mut g, Action::FinishCrew);
            finish(&mut g);
            settle(&mut g);
            assert!(minimum == 4 || minimum == 6);
            assert!(g.object_view(a).tapped && g.object_view(b).tapped);
            assert_eq!(g.stats(id), Stats { power, toughness });
            assert_eq!(g.current_kinds(id) & 1, 1);
            assert!(g.keywords(id).contains(Keywords::FLYING));
            if name == "Dependable Quinjet" {
                g.state.objects[id as usize].summoning_sick = false;
                for color in 0..5 {
                    let mut branch = g.clone();
                    branch.state.players[p.index()].mana = [0; 6];
                    checked(&mut branch, mana(id, 0, color, 0));
                    assert_eq!(branch.state.players[p.index()].mana[color], 1);
                }
            }
            next_turn(&mut g);
            assert_eq!(g.current_kinds(id) & 1, 0);
        }
    }
}

#[test]
fn msh_inherited_kang_attack_connive_second_draw_drain_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Kang, Temporal Tyrant");
        paid(&mut g, p, "Opt"); // First draw, with ordinary private scry decisions.
        let discard = named(&mut g, p, "Hill Giant", Zone::Hand);
        g.state.objects[id as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        checked(&mut g, Action::Discard { card: discard });
        combat_end(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
        assert_eq!(g.life(p), 21);
        assert_eq!(g.life(p.other()), 15); // attack4 + drain1
    }
}

#[test]
fn msh_inherited_thing_simultaneous_heroes_damage_once_and_noncombat_hero_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let thing = paid(&mut g, p, "The Thing, Ben Grimm");
        let a = named(&mut g, p, "Agent of Atlas", Zone::Battlefield);
        let b = named(&mut g, p, "Agent of Atlas", Zone::Battlefield);
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: a });
        checked(&mut g, Action::Attack { card: b });
        checked(&mut g, Action::FinishAttackers);
        combat_end(&mut g);
        assert_eq!(g.life(p.other()), 16);
        assert_eq!(g.counter_count(thing, CounterKind::PlusOne), 2);
    }
}

#[test]
fn msh_inherited_red_guardian_flash_only_opposing_damage_source_current_turn() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let attacker = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let clean = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            attackers(&mut g, p.other());
            checked(&mut g, Action::Attack { card: attacker });
            checked(&mut g, Action::FinishAttackers);
            combat_end(&mut g);
            assert_eq!(g.life(p), 17);
            g.state.priority = p;
            g.state.players[p.index()].mana = [30; 6];
            let id = source(&mut g, p, "Red Guardian, Super-Soldier");
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(attacker),
                })
            });
            for bad in [clean, own] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bad),
                    },
                );
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(attacker),
                },
            );
            finish(&mut g);
            g.move_object(id, Zone::Graveyard);
            if blink {
                g.move_object(attacker, Zone::Hand);
                g.move_object(attacker, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(attacker).zone,
                if blink {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

fn combat_end(g: &mut Game) {
    for _ in 0..120 {
        if g.state.step == Step::Main2
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishBlockers
                        | Action::FinishAttackers
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("combat did not end");
}

#[test]
fn msh_inherited_keyword_flying_vigilance_reach_actual_attack_and_block_legality() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Kree Commandos",
            "Thor Odinson",
            "H.E.R.B.I.E. Scout Unit",
            "Giant-Sized Flying Ant",
            "Wakandan Drone Flock",
            "Yellowjacket, Heartless Marauder",
            "S.H.I.E.L.D. Deployment Drone",
        ] {
            let mut g = setup(p);
            named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let id = paid(&mut g, p, name);
            let ground = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let reach = named(&mut g, p.other(), "Kree Sentinel", Zone::Battlefield);
            let gorilla = named(&mut g, p.other(), "Guerrilla Gorilla", Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = false;
            attackers(&mut g, p);
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            if ["Kree Commandos", "Thor Odinson"].contains(&name) {
                assert!(!g.object_view(id).tapped);
            }
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            reject(
                &mut g,
                Action::Block {
                    blocker: ground,
                    attacker: id,
                },
            );
            assert!(g.legal_actions().contains(&Action::Block {
                blocker: reach,
                attacker: id
            }));
            checked(
                &mut g,
                Action::Block {
                    blocker: gorilla,
                    attacker: id,
                },
            );
        }
    }
}

#[test]
fn msh_inherited_keyword_menace_double_strike_lifelink_trample_and_deathtouch_damage() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Roxxon Brutes",
            "Mockingbird, Ace Agent",
            "Kingpin's Enforcers",
            "Savage Land Dinosaur",
            "The Serpent Society",
        ] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            let base = g.stats(id).power;
            g.state.objects[id as usize].summoning_sick = false;
            let blocker = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            attackers(&mut g, p);
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            if name == "Roxxon Brutes" {
                let second = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
                checked(
                    &mut g,
                    Action::Block {
                        blocker,
                        attacker: id,
                    },
                );
                reject(&mut g, Action::FinishBlockers);
                checked(
                    &mut g,
                    Action::Block {
                        blocker: second,
                        attacker: id,
                    },
                );
                checked(&mut g, Action::FinishBlockers);
            } else if name == "Savage Land Dinosaur" || name == "The Serpent Society" {
                checked(
                    &mut g,
                    Action::Block {
                        blocker,
                        attacker: id,
                    },
                );
                checked(&mut g, Action::FinishBlockers);
            } else {
                checked(&mut g, Action::FinishBlockers);
            }
            combat_end(&mut g);
            match name {
                "Mockingbird, Ace Agent" => assert_eq!(g.life(p.other()), 20 - 2 * base),
                "Kingpin's Enforcers" => {
                    assert_eq!(g.life(p), 20 + base);
                    assert_eq!(g.life(p.other()), 20 - base);
                }
                "Savage Land Dinosaur" => {
                    assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
                    assert_eq!(g.life(p.other()), 18);
                }
                "The Serpent Society" => assert_eq!(g.object_view(blocker).zone, Zone::Graveyard),
                _ => {}
            }
        }
    }
}

#[test]
fn msh_inherited_depower_attack_discount_stale_target_no_draw_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for attacking in [false, true] {
            for blink in [false, true] {
                let mut g = setup(p);
                let id = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                if attacking {
                    attackers(&mut g, p.other());
                    checked(&mut g, Action::Attack { card: id });
                    checked(&mut g, Action::FinishAttackers);
                }
                g.state.priority = p;
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, if attacking { 0 } else { 2 }];
                let hand = g.hand_count(p);
                let spell = named(&mut g, p, "Depower", Zone::Hand);
                cast(&mut g, spell, &[Target::Card(id)]);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                if blink {
                    g.move_object(id, Zone::Hand);
                    g.move_object(id, Zone::Battlefield);
                }
                settle(&mut g);
                assert_eq!(g.stats(id).power, if blink { 3 } else { -1 });
                assert_eq!(g.hand_count(p), hand + usize::from(!blink));
                next_turn(&mut g);
                assert_eq!(g.stats(id).power, 3);
            }
        }
    }
}

#[test]
fn msh_inherited_impulse_permission_next_turn_paid_land_spell_and_exact_expiry() {
    for p in [Player::P0, Player::P1] {
        for name in ["Crimson Operative", "Blazing Crescendo"] {
            for use_spell in [false, true] {
                let mut g = setup(p);
                let top = named(
                    &mut g,
                    p,
                    if use_spell { "Hill Giant" } else { "Island" },
                    Zone::Library,
                );
                let target = named(&mut g, p, "Hill Giant", Zone::Battlefield);
                let source = named(&mut g, p, name, Zone::Hand);
                let targets = if name == "Blazing Crescendo" {
                    vec![Target::Card(target)]
                } else {
                    vec![]
                };
                cast(&mut g, source, &targets);
                settle(&mut g);
                assert_eq!(g.object_view(top).zone, Zone::Exile);
                if g.object_view(source).zone != Zone::Graveyard {
                    g.move_object(source, Zone::Graveyard);
                }
                if use_spell {
                    g.state.players[p.index()].mana = [0; 6];
                    reject(&mut g, Action::AnnounceExile { card: top });
                    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
                    checked(&mut g, Action::AnnounceExile { card: top });
                    finish(&mut g);
                    settle(&mut g);
                    assert_eq!(g.object_view(top).zone, Zone::Battlefield);
                } else {
                    next_turn(&mut g);
                    g.state.priority = p;
                    reject(&mut g, Action::PlayLand { card: top }); // not this player's turn
                    next_turn(&mut g);
                    g.state.priority = p;
                    g.state.step = Step::Main1;
                    assert!(g.legal_actions().contains(&Action::PlayLand { card: top }));
                    next_turn(&mut g);
                    next_turn(&mut g);
                    g.state.priority = p;
                    g.state.step = Step::Main1;
                    reject(&mut g, Action::PlayLand { card: top });
                    assert_eq!(g.object_view(top).zone, Zone::Exile);
                }
            }
        }
    }
}

#[test]
fn msh_inherited_roxxon_second_draw_any_creature_counter_and_stale_target() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let source = paid(&mut g, p, "Roxxon Brutes");
            let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = named(&mut g, p, "Divination", Zone::Hand);
            cast(&mut g, spell, &[]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(target),
                })
            });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            finish(&mut g);
            g.move_object(source, Zone::Graveyard);
            if blink {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(target, CounterKind::PlusOne),
                u64::from(!blink)
            );
        }
    }
}

#[test]
fn msh_inherited_okoye_attacking_tokens_first_strike_combat_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Okoye, Dora Milaje Leader");
        let soldier = tokens(&g, p, "Soldier")[0];
        assert!(!g.keywords(soldier).contains(Keywords::FIRST_STRIKE));
        let blocker = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.state.objects[soldier as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: soldier });
        checked(&mut g, Action::FinishAttackers);
        assert!(g.keywords(soldier).contains(Keywords::FIRST_STRIKE));
        assert!(!g.keywords(id).contains(Keywords::FIRST_STRIKE));
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: soldier,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        combat_end(&mut g);
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(g.object_view(soldier).zone, Zone::Battlefield);
        assert!(!g.keywords(soldier).contains(Keywords::FIRST_STRIKE));
    }
}

#[test]
fn msh_inherited_vigilance_exact_cavalry_guard_and_hero_tokens_attack_untapped() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Atlantean Cavalry",
            "Wakandan Royal Guard",
            "Borough Backup",
        ] {
            let mut g = setup(p);
            let source = paid(&mut g, p, name);
            let id = if name == "Borough Backup" {
                tokens(&g, p, "Hero")[0]
            } else {
                source
            };
            g.state.objects[id as usize].summoning_sick = false;
            attackers(&mut g, p);
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            assert!(!g.object_view(id).tapped);
        }
    }
}

#[test]
fn msh_inherited_tigra_flash_repeated_life_events_and_trample_live_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.priority = p;
        let id = paid(&mut g, p, "Tigra, Feline Fury");
        for expected in [1, 2] {
            if g.actor() != Some(p) {
                checked(&mut g, Action::Pass);
            }
            let spell = named(&mut g, p, "Rapid Rescue", Zone::Hand);
            cast(&mut g, spell, &[]);
            settle(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), expected);
        }
        let blocker = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.state.objects[id as usize].summoning_sick = false;
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: id });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: id,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        combat_end(&mut g);
        assert_eq!(g.life(p.other()), 17);
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_inherited_super_strength_stats_trample_and_aura_target_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let creature = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Super Strength", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(creature),
            Stats {
                power: 7,
                toughness: 7
            }
        );
        let blocker = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        attackers(&mut g, p);
        checked(&mut g, Action::Attack { card: creature });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker,
                attacker: creature,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        combat_end(&mut g);
        assert_eq!(g.life(p.other()), 18);
        g.move_object(creature, Zone::Hand);
        g.state_based_actions();
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn msh_inherited_thing_noncombat_batch_filters_controller_and_counts_each_player_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let thing = paid(&mut g, p, "The Thing, Ben Grimm");
        let a = paid(&mut g, p, "Agent of Atlas");
        let b = named(&mut g, p, "Agent of Atlas", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Agent of Atlas", Zone::Battlefield);
        // Typed noncombat event fixture supplements actual declared-combat coverage.
        let events = [a, b, foreign].map(|id| DamageEvent {
            source: g.damage_source(id),
            target: Target::Player(p.other()),
            amount: 1,
            combat: false,
            preventable: true,
        });
        g.apply_damage_batch(&events);
        settle(&mut g);
        assert_eq!(g.counter_count(thing, CounterKind::PlusOne), 2);
        let event = DamageEvent {
            source: g.damage_source(a),
            target: Target::Player(p),
            amount: 1,
            combat: false,
            preventable: true,
        };
        g.apply_damage_batch(&[event]);
        settle(&mut g);
        assert_eq!(g.counter_count(thing, CounterKind::PlusOne), 4);
    }
}

#[test]
fn msh_inherited_decoy_one_dual_hero_villain_card_can_fill_both_groups() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let both = named(&mut g, p, "White Widow, Free Agent", Zone::Graveyard);
        let spell = named(&mut g, p, "Decoy Ploy", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        for _ in 0..2 {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(both),
                },
            );
        }
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(both).zone, Zone::Hand);
    }
}

#[test]
fn msh_inherited_epic_fight_partial_illegal_relationship_keeps_doubling_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = named(&mut g, p, "Epic Fight", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        for target in [own, own, enemy] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        finish(&mut g);
        let object = g.object_ref(own);
        g.change_controller(object, p.other());
        settle(&mut g);
        assert_eq!(g.stats(own).power, 6);
        assert_eq!(g.object_view(enemy).damage, 0);
        assert_eq!(g.object_view(own).damage, 0);
    }
}

#[test]
fn msh_inherited_hydra_or_predicate_both_types_once_own_villain_only_and_enemy_excluded() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "HYDRA Assault Robot");
        assert_eq!(g.life(p.other()), 20);
        for (name, expected) in [
            ("Kree Sentinel", 19),
            ("Agents of HYDRA", 18),
            ("Agent of Atlas", 18),
        ] {
            paid(&mut g, p, name);
            assert_eq!(g.life(p.other()), expected, "{name}");
        }
        g.state.active = p.other();
        g.state.priority = p.other();
        paid(&mut g, p.other(), "Kree Sentinel");
        assert_eq!(g.life(p.other()), 18);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
}

#[test]
fn msh_inherited_machinesmith_and_thing_intrinsic_trample_with_blocker_damage() {
    for p in [Player::P0, Player::P1] {
        for name in ["Machinesmith Automaton", "The Thing, Ben Grimm"] {
            let mut g = setup(p);
            let id = paid(&mut g, p, name);
            let power = g.stats(id).power;
            let blocker = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = false;
            attackers(&mut g, p);
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            checked(
                &mut g,
                Action::Block {
                    blocker,
                    attacker: id,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            combat_end(&mut g);
            assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
            assert_eq!(g.life(p.other()), 21 - power);
        }
    }
}

#[test]
fn msh_inherited_lightning_can_target_opposing_planeswalker_and_not_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        g.state.priority = p.other();
        let walker = paid(&mut g, p.other(), "Chandra, Flameshaper");
        let before = g.counter_count(walker, CounterKind::Loyalty);
        g.state.active = p;
        g.state.priority = p;
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let spell = named(&mut g, p, "Lightning Strike", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(walker),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), before - 3);
    }
}
