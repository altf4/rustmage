//! Paid-action audit of inherited TMT admissions. Every checked action compares
//! raw/index owned branches, both observations, rejected actors and restore.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, settle_on, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            .unwrap();
        checked(g, choice);
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    let original = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..8 {
        if g.state.announcing.as_ref().map(|a| a.item.id) != original
            || g.state.announcing.is_none()
        {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn activate_ability(g: &mut Game, id: CardId, ability: u16, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
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
    panic!("unpaid activation");
}
fn until(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if condition(g) {
            return;
        }
        g.prepare_triggers();
        if condition(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("condition not reached {:?}", g.state.pending);
}
fn combat(g: &mut Game, id: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
}
fn own_main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.active == p
            && g.state.step == Step::Main1
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
    });
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
fn activate(g: &mut Game, id: CardId, targets: &[Target]) {
    activate_ability(g, id, 0, targets);
}
fn count_subtype(g: &Game, p: Player, subtype: &str) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && g.has_subtype(*id as CardId, subtype)
        })
        .count()
}

#[test]
fn tmt_inherited_four_sally_resolution_nontoken_count_and_attack_counters_every_owned_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let old_token = named(
            &mut g,
            p,
            "@token/667dad32c9f20b70d5e9bda3",
            Zone::Battlefield,
        );
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let sally = paid(&mut g, p, "Sally Pride, Lioness Leader", &[]);
        settle(&mut g);
        assert_eq!(count_subtype(&g, p, "Mutant"), 3);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        combat(&mut g, sally);
        for card in [sally, bear, old_token] {
            assert_eq!(g.counter_count(card, CounterKind::PlusOne), 1);
        }
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
    }
    // The count is evaluated on resolution and excludes a departed source.
    let p = Player::P0;
    let mut g = setup(p);
    named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let sally = paid(&mut g, p, "Sally Pride, Lioness Leader", &[]);
    until(&mut g, |g| {
        g.state.objects[sally as usize].zone == Zone::Battlefield
    });
    g.move_object(sally, Zone::Hand);
    settle(&mut g);
    assert_eq!(count_subtype(&g, p, "Mutant"), 1);
}
#[test]
fn tmt_inherited_four_saved_shell_turtle_discount_own_targets_permanent_counter_temporary_keywords()
{
    for turtle in [false, true] {
        let p = Player::P0;
        let q = p.other();
        let mut g = setup(p);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, q, "Aegis Turtle", Zone::Battlefield);
        if turtle {
            named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        }
        let spell = named(&mut g, p, "Saved by the Shell", Zone::Hand);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before - if turtle { 1 } else { 2 }
        );
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert!(
            g.keywords(bear).contains(
                Keywords::TRAMPLE
                    .union(Keywords::HEXPROOF)
                    .union(Keywords::INDESTRUCTIBLE)
            )
        );
        paid(&mut g, p, "Murder", &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
        checked(&mut g, Action::Pass);
        let hostile = named(&mut g, q, "Murder", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: hostile });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        next_turn(&mut g);
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
    }
}
#[test]
fn tmt_inherited_four_shredders_revenge_each_mode_targets_either_player_private_discard() {
    for p in [Player::P0, Player::P1] {
        for target in [p, p.other()] {
            let mut g = setup(p);
            let first = named(&mut g, target, "Shock", Zone::Hand);
            let second = named(&mut g, target, "Island", Zone::Hand);
            let spell = named(&mut g, p, "Shredder's Revenge", Zone::Hand);
            cast(&mut g, spell, Some(0), &[Target::Player(target)]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::Discard { card: first })
            });
            assert_eq!(g.actor(), Some(target));
            assert!(g.observe(target.other()).actions.is_empty());
            checked(&mut g, Action::Discard { card: first });
            checked(&mut g, Action::Discard { card: second });
            settle(&mut g);
            assert_eq!(g.state.objects[first as usize].zone, Zone::Graveyard);
            assert_eq!(g.state.objects[second as usize].zone, Zone::Graveyard);
            let before = g.state.players[target.index()].library.len();
            let life = g.state.players[target.index()].life;
            let spell = named(&mut g, p, "Shredder's Revenge", Zone::Hand);
            cast(&mut g, spell, Some(1), &[Target::Player(target)]);
            settle(&mut g);
            assert_eq!(g.state.players[target.index()].library.len(), before - 2);
            assert_eq!(g.state.players[target.index()].life, life - 2);
        }
    }
}
#[test]
fn tmt_inherited_four_skateboard_any_permanent_etb_tap_and_equipment_haste() {
    for tap_self in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let enemy_land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        let bear = paid(&mut g, p, "Grizzly Bears", &[]);
        settle(&mut g);
        let board = paid(&mut g, p, "Skateboard", &[]);
        settle_on(
            &mut g,
            Some(Target::Card(if tap_self { board } else { enemy_land })),
        );
        assert!(g.state.objects[if tap_self { board } else { enemy_land } as usize].tapped);
        activate(&mut g, board, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (3, 2));
        assert!(g.keywords(bear).contains(Keywords::HASTE));
        combat(&mut g, bear);
        g.move_object(board, Zone::Hand);
        assert_eq!(g.stats(bear).power, 2);
        assert!(!g.keywords(bear).contains(Keywords::HASTE));
    }
}
#[test]
fn tmt_inherited_four_slash_attack_token_is_not_attacking_and_alliance_damage_per_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let slash = paid(&mut g, p, "Slash, Reptile Rampager", &[]);
        settle(&mut g);
        let life = g.state.players[p.other().index()].life;
        paid(&mut g, p, "Mechanized Ninja Cavalry", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life - 4);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        let life = g.state.players[p.other().index()].life;
        combat(&mut g, slash);
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card.has_subtype("Mutant")
            })
            .unwrap()
            .0 as CardId;
        assert!(!g.state.objects[token as usize].tapped);
        assert!(!g.state.attackers.iter().any(|a| a.card == token));
        assert_eq!((g.stats(token).power, g.stats(token).toughness), (2, 2));
    }
}
#[test]
fn tmt_inherited_four_slithering_cryptid_both_hybrid_costs_and_mutagen_activation() {
    for color in [1, 4] {
        let p = Player::P0;
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].mana[color] = 3;
        let cryptid = paid(&mut g, p, "Slithering Cryptid", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!((g.stats(cryptid).power, g.stats(cryptid).toughness), (2, 3));
        assert_eq!(count_subtype(&g, p, "Mutagen"), 1);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Mutagen")
            })
            .unwrap()
            .0 as CardId;
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        activate(&mut g, token, &[Target::Card(cryptid)]);
        settle(&mut g);
        assert_eq!(g.counter_count(cryptid, CounterKind::PlusOne), 1);
    }
}
#[test]
fn tmt_inherited_four_squirrelanoids_deathtouch_actual_combat_and_tenderize_source_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let squirrel = paid(&mut g, p, "Squirrelanoids", &[]);
        settle(&mut g);
        let victim = named(&mut g, p.other(), "Primordial Pachyderm", Zone::Battlefield);
        let spell = named(&mut g, p, "Tenderize", Zone::Hand);
        cast(
            &mut g,
            spell,
            None,
            &[Target::Card(squirrel), Target::Card(victim)],
        );
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[squirrel as usize].damage, 0);
        next_turn(&mut g);
        own_main(&mut g, p.other());
        let big = named(&mut g, p.other(), "Grasping Longneck", Zone::Battlefield);
        combat(&mut g, big);
        checked(
            &mut g,
            Action::Block {
                blocker: squirrel,
                attacker: big,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        assert_eq!(g.state.objects[big as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[squirrel as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tmt_inherited_four_stockman_paid_draw_then_private_discard_newly_drawn_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let top = named(&mut g, p, "Shock", Zone::Library);
        let before = g.state.players[p.index()].library.len();
        let stockman = paid(&mut g, p, "Stockman, Mad Fly-entist", &[]);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::Discard { card: top })
        });
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        assert!(g.observe(p.other()).actions.is_empty());
        checked(&mut g, Action::Discard { card: top });
        settle(&mut g);
        assert_eq!(g.state.objects[top as usize].zone, Zone::Graveyard);
        assert!(g.keywords(stockman).contains(Keywords::FLYING));
        assert_eq!(
            (g.stats(stockman).power, g.stats(stockman).toughness),
            (3, 4)
        );
    }
}
#[test]
fn tmt_inherited_four_island_and_mountain_cycling_paid_discard_filter_reveal_and_fail_find() {
    for (name, land) in [
        ("Stockman, Mad Fly-entist", "Island"),
        ("Zog, Triceraton Castaway", "Mountain"),
    ] {
        for choose in [false, true] {
            let p = Player::P0;
            let mut g = setup(p);
            let card = named(&mut g, p, name, Zone::Hand);
            let hit = named(&mut g, p, land, Zone::Library);
            let forest = *g.state.players[p.index()].library.first().unwrap();
            let library = g.state.players[p.index()].library.clone();
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            activate(&mut g, card, &[]);
            assert_eq!(g.state.objects[card as usize].zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                before - 2
            );
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p.other()).actions.is_empty());
            reject(&mut g, Action::ChooseSearch { card: forest });
            if choose {
                checked(&mut g, Action::ChooseSearch { card: hit });
            } else {
                checked(&mut g, Action::FinishSearch);
            }
            settle(&mut g);
            assert_eq!(
                g.state.objects[hit as usize].zone,
                if choose { Zone::Hand } else { Zone::Library }
            );
            if choose {
                assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(hit)));
            }
            assert_ne!(g.state.players[p.index()].library, library);
        }
    }
}
fn activate_technodrome(g: &mut Game, source: CardId, artifact: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseAdditionalCost {
            object: g.object_ref(artifact),
        })
    });
    let object = g.object_ref(source);
    reject(g, Action::ChooseAdditionalCost { object });
    let object = g.object_ref(artifact);
    checked(g, Action::ChooseAdditionalCost { object });
    until(g, |g| g.state.announcing.is_none());
}
#[test]
fn tmt_inherited_four_technodrome_other_artifact_sacrifice_cost_draw_counter_and_power_threshold() {
    let p = Player::P0;
    let mut g = setup(p);
    let tech = paid(&mut g, p, "Technodrome", &[]);
    settle(&mut g);
    let food = named(
        &mut g,
        p,
        "@token/283fd83a9aec83a51b736926",
        Zone::Battlefield,
    );
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: tech,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    for n in 1..=3 {
        let artifact = if n == 1 {
            food
        } else {
            named(
                &mut g,
                p,
                "@token/283fd83a9aec83a51b736926",
                Zone::Battlefield,
            )
        };
        let before = g.state.players[p.index()].library.len();
        activate_technodrome(&mut g, tech, artifact);
        assert!(g.state.objects[tech as usize].tapped);
        assert_ne!(g.state.objects[artifact as usize].zone, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        assert_eq!(g.counter_count(tech, CounterKind::PlusOne), n);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
    }
    assert_eq!((g.stats(tech).power, g.stats(tech).toughness), (6, 6));
    combat(&mut g, tech);
}
#[test]
fn tmt_inherited_four_technodrome_attack_and_block_restrictions_use_current_power() {
    let p = Player::P0;
    let mut g = setup(p);
    let tech = paid(&mut g, p, "Technodrome", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    reject(&mut g, Action::Attack { card: tech });
    next_turn(&mut g);
    own_main(&mut g, p.other());
    let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    combat(&mut g, enemy);
    reject(
        &mut g,
        Action::Block {
            blocker: tech,
            attacker: enemy,
        },
    );
    g.set_counters(tech, Counters::of(CounterKind::PlusOne, 3));
    checked(
        &mut g,
        Action::Block {
            blocker: tech,
            attacker: enemy,
        },
    );
    assert!(
        g.keywords(tech)
            .contains(Keywords::REACH.union(Keywords::TRAMPLE))
    );
}
#[test]
fn tmt_inherited_four_tenderize_own_opponent_groups_and_either_departed_target_no_damage() {
    for disappear_source in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Primordial Pachyderm", Zone::Battlefield);
        let spell = named(&mut g, p, "Tenderize", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        if !g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(enemy),
        }) {
            checked(&mut g, Action::FinishTargets);
        }
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
        until(&mut g, |g| g.state.announcing.is_none());
        g.move_object(if disappear_source { own } else { enemy }, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].damage, 0);
        assert_eq!(g.state.objects[enemy as usize].damage, 0);
    }
}
#[test]
fn tmt_inherited_four_bovine_summoning_sickness_each_single_color_two_mana_no_split() {
    for color in 0..5 {
        let p = Player::P0;
        let mut g = setup(p);
        let bovine = paid(&mut g, p, "Transdimensional Bovine", &[]);
        settle(&mut g);
        let mut production = [0; 6];
        production[color] = 2;
        let a = Action::ActivateMana {
            card: bovine,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        };
        reject(&mut g, a.clone());
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::ActivateMana {
                card: bovine,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [1, 1, 0, 0, 0, 0],
            },
        );
        checked(&mut g, a);
        assert_eq!(g.state.players[p.index()].mana, production);
        assert!(g.state.objects[bovine as usize].tapped);
        assert!(g.keywords(bovine).contains(Keywords::FLYING));
        assert_eq!((g.stats(bovine).power, g.stats(bovine).toughness), (0, 4));
    }
}
#[test]
fn tmt_inherited_four_tunnel_rats_paid_graveyard_ability_enemy_turn_returns_tapped_exact_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let rats = paid(&mut g, p, "Tunnel Rats", &[]);
        settle(&mut g);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: rats,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        paid(&mut g, p, "Murder", &[Target::Card(rats)]);
        settle(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p.other());
        checked(&mut g, Action::Pass);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, rats, &[]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 5
        );
        assert_eq!(g.state.objects[rats as usize].zone, Zone::Battlefield);
        assert!(g.state.objects[rats as usize].tapped);
        assert!(g.state.objects[rats as usize].summoning_sick);
    }
    let p = Player::P0;
    let mut g = setup(p);
    let rats = named(&mut g, p, "Tunnel Rats", Zone::Graveyard);
    activate(&mut g, rats, &[]);
    g.move_object(rats, Zone::Exile);
    g.move_object(rats, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.state.objects[rats as usize].zone, Zone::Graveyard);
}
#[test]
fn tmt_inherited_four_turtle_blimp_token_crews_while_sick_tap_is_cost_and_animation_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let blimp = paid(&mut g, p, "Turtle Blimp", &[]);
        settle(&mut g);
        assert!(!g.is_creature(blimp));
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card.has_subtype("Mutant")
            })
            .unwrap()
            .0 as CardId;
        assert_eq!((g.stats(token).power, g.stats(token).toughness), (2, 2));
        assert!(g.state.objects[token as usize].summoning_sick);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: blimp,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        until(&mut g, |g| matches!(g.state.pending, Pending::Crew { .. }));
        let object = g.object_ref(blimp);
        reject(&mut g, Action::ChooseCrewer { object });
        let object = g.object_ref(token);
        checked(&mut g, Action::ChooseCrewer { object });
        checked(&mut g, Action::FinishCrew);
        until(&mut g, |g| g.state.announcing.is_none());
        assert!(g.state.objects[token as usize].tapped);
        assert!(!g.is_creature(blimp));
        settle(&mut g);
        assert!(g.is_creature(blimp));
        assert_eq!((g.stats(blimp).power, g.stats(blimp).toughness), (3, 4));
        assert!(g.keywords(blimp).contains(Keywords::FLYING));
        next_turn(&mut g);
        assert!(!g.is_creature(blimp));
    }
}
#[test]
fn tmt_inherited_four_turtle_power_flash_dynamic_own_turtle_boost_and_source_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        next_turn(&mut g);
        own_main(&mut g, p.other());
        checked(&mut g, Action::Pass);
        let power = paid(&mut g, p, "Turtle Power!", &[]);
        settle(&mut g);
        assert_eq!((g.stats(own).power, g.stats(own).toughness), (2, 7));
        assert_eq!((g.stats(enemy).power, g.stats(enemy).toughness), (0, 5));
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        next_turn(&mut g);
        own_main(&mut g, p);
        let late = paid(&mut g, p, "Donatello, Turtle Techie", &[]);
        settle(&mut g);
        assert_eq!((g.stats(late).power, g.stats(late).toughness), (5, 6));
        paid(&mut g, p, "Make Your Move", &[Target::Card(power)]);
        settle(&mut g);
        assert_eq!((g.stats(own).power, g.stats(own).toughness), (0, 5));
        assert_eq!((g.stats(late).power, g.stats(late).toughness), (3, 4));
    }
}
#[test]
fn tmt_inherited_four_utrom_optional_tap_stun_and_actual_next_untap_replacement() {
    for choose in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        paid(&mut g, p, "Utrom Scientists", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        if choose {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
        } else {
            checked(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::Stun), u64::from(choose));
        assert_eq!(g.state.objects[enemy as usize].tapped, choose);
        next_turn(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::Stun), 0);
        assert_eq!(g.state.objects[enemy as usize].tapped, choose);
        next_turn(&mut g);
        next_turn(&mut g);
        assert!(!g.state.objects[enemy as usize].tapped);
    }
}
#[test]
fn tmt_inherited_four_zog_mandatory_target_cannot_block_this_turn_and_keyword_body() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let attacker = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let blocker = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let zog = paid(&mut g, p, "Zog, Triceraton Castaway", &[]);
        settle_on(&mut g, Some(Target::Card(blocker)));
        assert_eq!((g.stats(zog).power, g.stats(zog).toughness), (5, 4));
        assert!(
            g.keywords(zog)
                .contains(Keywords::REACH.union(Keywords::TRAMPLE))
        );
        combat(&mut g, attacker);
        reject(&mut g, Action::Block { blocker, attacker });
        next_turn(&mut g);
        assert!(!g.keywords(blocker).contains(Keywords::CANT_BLOCK));
    }
}
