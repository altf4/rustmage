//! Independent bounded scenarios for stored MSH identities 71..141.
//! Outcomes below are literals transcribed from Oracle, not card-definition data.
use super::super::ability_world::CharacteristicWorld;
use super::*;

#[track_caller]
fn act(g: &mut Game, action: Action) {
    if let Err(error) = g.apply(g.actor().unwrap(), action.clone()) {
        panic!("{action:?}: {error:?}, pending {:?}", g.state.pending);
    }
}
fn mana(g: &mut Game, p: Player, value: Mana) {
    g.state.players[p.index()].mana = value;
}
fn fresh(p: Player) -> Game {
    let mut g = scenario(p);
    mana(&mut g, p, [0; 6]);
    mana(&mut g, p.other(), [0; 6]);
    g
}
fn reference(g: &Game, id: CardId) -> ObjectRef {
    g.object_ref(id)
}
fn target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn decision(g: &mut Game) {
    for _ in 0..32 {
        if g.state.pending != Pending::Priority {
            return;
        }
        assert!(
            !g.state.stack.is_empty() || g.state.resolving.is_some(),
            "expected resolution choice"
        );
        act(g, Action::Pass);
    }
    panic!("32 passes without the expected choice");
}
fn activate(g: &mut Game, id: CardId, ability: u16, targets: &[Target]) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &t in targets {
        act(g, Action::ChooseTarget { target: t });
    }
    finish_targets(g);
    pay(g);
}
fn go(g: &mut Game, player: Player, step: Step) {
    for _ in 0..80 {
        if g.state.active == player
            && g.state.step == step
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
        {
            return;
        }
        match g.state.pending {
            Pending::Priority => act(g, Action::Pass),
            Pending::Attackers => act(g, Action::FinishAttackers),
            Pending::Blockers => act(g, Action::FinishBlockers),
            _ => panic!(
                "explicit choice required advancing to {step:?}: {:?}",
                g.state.pending
            ),
        }
    }
    panic!("80-action phase budget");
}
fn attacks(g: &mut Game, ids: &[CardId]) {
    go(g, g.state.active, Step::BeginCombat);
    for _ in 0..4 {
        if g.state.pending == Pending::Attackers {
            break;
        }
        act(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Attackers);
    for &card in ids {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
}
fn token(g: &Game, name: &str) -> CardId {
    g.state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token() && o.card.name() == name)
        .unwrap_or_else(|| panic!("missing token {name}"))
        .0 as CardId
}
fn produce(g: &mut Game, card: CardId, production: Mana) {
    choose(
        g,
        |a| matches!(a,Action::ActivateMana{card:c,production:p,..}if *c==card&&*p==production),
    );
}
fn teamwork_payment(g: &mut Game, worker: CardId) {
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    let clause = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| matches!(r.clause, CastCostClause::Teamwork(_)))
        .unwrap() as u16;
    if matches!(g.state.pending, Pending::CastCosts { .. }) {
        act(g, Action::ChooseCastCost { clause });
    }
    let object = reference(g, worker);
    act(g, Action::ChooseTeammate { object });
    act(g, Action::FinishTeamwork);
}
fn modal(g: &mut Game, card: CardId, mode: u16, targets: &[Target]) {
    act(g, Action::AnnounceCast { card });
    act(g, Action::ChooseMode { mode });
    for &t in targets {
        act(g, Action::ChooseTarget { target: t });
    }
    finish_targets(g);
    pay(g);
}

#[test]
fn msh_b_lands_have_real_entry_and_mana_boundaries() {
    for (name, colors) in [
        ("Fisk Tower", [0, 2]),
        ("Hell's Kitchen", [2, 3]),
        ("Los Diablos Missile Base", [3, 4]),
    ] {
        let p = Player::P0;
        let mut g = fresh(p);
        let land = put(&mut g, p, name, Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        settle(&mut g);
        assert!(g.state.objects[land as usize].tapped);
        assert_eq!(g.life(p), 21);
        reject(&mut g, Action::TapMana { card: land });
        go(&mut g, p.other(), Step::Main1);
        go(&mut g, p, Step::Main1);
        for color in colors {
            // Independent alternate branches need only the tapped flag reset.
            g.state.objects[land as usize].tapped = false;
            mana(&mut g, p, [0; 6]);
            let mut production = [0; 6];
            production[color] = 1;
            produce(&mut g, land, production);
            assert_eq!(g.state.players[p.index()].mana, production);
        }
    }
    for (name, colors) in [
        ("Gathering Place", [4, 0]),
        ("Gleaming Bastion", [0, 1]),
        ("Hidden Lair", [1, 2]),
    ] {
        let p = Player::P1;
        let mut g = fresh(p);
        let id = put(&mut g, p, name, Zone::Hand);
        act(&mut g, Action::PlayLand { card: id });
        let mut color = [0; 6];
        color[colors[0]] = 1;
        produce(&mut g, id, color);
        assert_eq!(g.state.players[p.index()].mana, color);
        go(&mut g, p.other(), Step::Main1);
        go(&mut g, p, Step::Main1);
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==id&&*production==color)
        ));
        let mut c = [0; 6];
        c[5] = 1;
        produce(&mut g, id, c);
        assert_eq!(g.state.players[p.index()].mana[5], 1);
        put(&mut g, p, "Forest", Zone::Battlefield);
        g.state.objects[id as usize].tapped = false;
        mana(&mut g, p, [0; 6]);
        let mut c = [0; 6];
        c[colors[1]] = 1;
        produce(&mut g, id, c);
        assert_eq!(g.state.players[p.index()].mana, c);
    }
    for (name, color) in [("Forest", 4), ("Island", 1)] {
        let mut g = fresh(Player::P0);
        let id = put(&mut g, Player::P0, name, Zone::Hand);
        act(&mut g, Action::PlayLand { card: id });
        act(&mut g, Action::TapMana { card: id });
        let mut expected = [0; 6];
        expected[color] = 1;
        assert_eq!(g.state.players[0].mana, expected);
    }
}

#[test]
fn msh_b_instant_pump_damage_and_surveil() {
    let p = Player::P0;
    let mut g = fresh(p);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
    cast(&mut g, growth, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(stats(&g, bear), (5, 5));
    go(&mut g, p.other(), Step::Main1);
    assert_eq!(stats(&g, bear), (2, 2));
    // The same any-target spell can hit a player or a creature, with exactly 3 damage.
    for player_target in [false, true] {
        let mut g = fresh(p);
        let enemy = put(&mut g, p.other(), "Kree Sentinel", Zone::Battlefield);
        let bolt = put(&mut g, p, "Lightning Strike", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
        cast(
            &mut g,
            bolt,
            Some(if player_target {
                Target::Player(p.other())
            } else {
                Target::Card(enemy)
            }),
        );
        settle(&mut g);
        if player_target {
            assert_eq!(g.life(p.other()), 17);
        } else {
            assert_eq!(g.state.objects[enemy as usize].damage, 3);
        }
    }
    let mut g = fresh(p);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, p, "Hour of Defeat", Zone::Hand);
    let top = *g.state.players[0].library.last().unwrap();
    mana(&mut g, p, [0, 0, 4, 0, 0, 0]);
    cast(&mut g, spell, Some(Target::Card(enemy)));
    decision(&mut g);
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    settle(&mut g);
    assert_eq!(g.state.objects[top as usize].zone, Zone::Graveyard);
}

#[test]
fn msh_b_epic_and_teamwork_choose_printed_modes() {
    let p = Player::P0;
    let mut g = fresh(p);
    let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Hydraulic Helper", Zone::Battlefield);
    let spell = put(&mut g, p, "Epic Fight", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 3, 0]);
    modal(
        &mut g,
        spell,
        2,
        &[Target::Card(own), Target::Card(own), Target::Card(enemy)],
    );
    settle(&mut g);
    assert_eq!(stats(&g, own), (4, 4));
    assert_eq!(g.state.objects[own as usize].damage, 2);
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    for (name, paid, minimum) in [
        ("Go Nuts!", false, 3),
        ("Go Nuts!", true, 3),
        ("HULK SMASH!", false, 4),
        ("HULK SMASH!", true, 4),
    ] {
        let mut g = fresh(p);
        let own = put(&mut g, p, "Hydraulic Helper", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let worker = put(
            &mut g,
            p,
            if minimum == 3 {
                "Hill Giant"
            } else {
                "Kree Sentinel"
            },
            Zone::Battlefield,
        );
        let artifact = put(&mut g, p.other(), "Futurist Forge", Zone::Battlefield);
        let spell = put(&mut g, p, name, Zone::Hand);
        mana(
            &mut g,
            p,
            if minimum == 3 {
                [0, 0, 0, 0, 1, 0]
            } else {
                [0, 0, 0, 2, 0, 0]
            },
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseTeamwork { pay: paid });
        act(
            &mut g,
            Action::ChooseMode {
                mode: if paid { 2 } else { 0 },
            },
        );
        if minimum == 3 {
            target(&mut g, own);
        } else {
            target(&mut g, artifact);
        }
        if paid {
            target(&mut g, own);
            target(&mut g, enemy);
        }
        finish_targets(&mut g);
        if paid {
            teamwork_payment(&mut g, worker);
        } else {
            pay(&mut g);
        }
        settle(&mut g);
        if minimum == 3 {
            assert_eq!(stats(&g, own), (3, 4));
        } else {
            assert_eq!(g.state.objects[artifact as usize].zone, Zone::Graveyard);
        }
        assert_eq!(g.state.objects[worker as usize].tapped, paid);
        assert_eq!(
            g.state.objects[enemy as usize].zone,
            if paid {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}

#[test]
fn msh_b_draw_entries_and_explicit_optional_land() {
    let p = Player::P0;
    for other in [false, true] {
        let mut g = fresh(p);
        if other {
            put(&mut g, p, "Iron Lad, Diverging Destiny", Zone::Battlefield);
        }
        let hero = put(&mut g, p, "Hero in Training", Zone::Hand);
        mana(&mut g, p, [3, 0, 0, 0, 0, 0]);
        cast(&mut g, hero, None);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.life(p), if other { 22 } else { 20 });
        assert_eq!(stats(&g, hero), (2, 2));
    }
    let mut g = fresh(p);
    let forge = put(&mut g, p, "Futurist Forge", Zone::Hand);
    mana(&mut g, p, [0, 2, 0, 0, 0, 0]);
    cast(&mut g, forge, None);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 1);
    mana(&mut g, p, [0, 4, 0, 0, 0, 0]);
    activate(&mut g, forge, 0, &[]);
    assert_eq!(g.state.objects[forge as usize].zone, Zone::Graveyard);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 3);
    for accept in [false, true] {
        let mut g = fresh(p);
        let herbie = put(&mut g, p, "H.E.R.B.I.E. Scout Unit", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 0, 0, 4]);
        cast(&mut g, herbie, None);
        decision(&mut g);
        assert_eq!(g.hand_count(p), 1);
        let land = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Hand)
            .unwrap()
            .0 as CardId;
        if accept {
            let object = reference(&g, land);
            act(&mut g, Action::PutHandLand { object });
        } else {
            act(&mut g, Action::DeclineHandEntry);
        }
        settle(&mut g);
        assert_eq!(
            g.state.objects[land as usize].zone,
            if accept {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        if accept {
            assert!(g.state.objects[land as usize].tapped);
        }
    }
}

#[test]
fn msh_b_tokens_enter_with_their_printed_abilities() {
    let p = Player::P0;
    let mut g = fresh(p);
    let falcon = put(&mut g, p, "Falcon, Winged Wonder", Zone::Hand);
    mana(&mut g, p, [0, 5, 0, 0, 0, 0]);
    cast(&mut g, falcon, None);
    settle(&mut g);
    let redwing = token(&g, "Redwing");
    assert!(g.object_view(redwing).is_legendary());
    assert!(g.has_subtype(redwing, "Bird") && g.has_subtype(redwing, "Scout"));
    assert_eq!(g.object_view(redwing).colors, 2);
    assert_eq!(stats(&g, redwing), (1, 1));
    assert!(g.keywords(redwing).contains(Keywords::FLYING));
    g.state.objects[redwing as usize].summoning_sick = false;
    attacks(&mut g, &[redwing]);
    decision(&mut g);
    let top = *g.state.players[0].library.last().unwrap();
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    settle(&mut g);
    assert_eq!(g.state.objects[top as usize].zone, Zone::Graveyard);
    for enough in [false, true] {
        let mut g = fresh(p);
        if enough {
            put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
            put(&mut g, p, "Hydraulic Helper", Zone::Graveyard);
        }
        let before = g.state.players[0].library.len();
        let troopers = put(&mut g, p, "HYDRA Troopers", Zone::Hand);
        mana(&mut g, p, [0, 0, 3, 0, 0, 0]);
        cast(&mut g, troopers, None);
        settle(&mut g);
        if enough {
            let v = token(&g, "Villain Token");
            assert!(g.has_subtype(v, "Villain"));
            assert_eq!(g.object_view(v).colors, 4);
            assert_eq!(stats(&g, v), (2, 1));
            assert!(g.state.objects[v as usize].tapped);
            assert!(g.keywords(v).contains(Keywords::MENACE));
        } else {
            assert_eq!(g.state.players[0].library.len(), before - 2);
        }
    }
    let mut g = fresh(p);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let crew = put(&mut g, p, "Hire a Crew", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 3, 0, 0]);
    cast(&mut g, crew, None);
    settle(&mut g);
    let v = token(&g, "Villain Token");
    assert!(g.has_subtype(v, "Villain"));
    assert_eq!(g.object_view(v).colors, 4);
    assert_eq!(stats(&g, v), (3, 1));
    assert_eq!(stats(&g, bear), (3, 2));
    let later = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&g, later), (2, 2));
    go(&mut g, p.other(), Step::Main1);
    assert_eq!(stats(&g, v), (2, 1));
    assert_eq!(stats(&g, bear), (2, 2));
    go(&mut g, p, Step::Main1);
    let first = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let second = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    attacks(&mut g, &[v]);
    for _ in 0..4 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(
        &mut g,
        Action::Block {
            blocker: first,
            attacker: v,
        },
    );
    reject(&mut g, Action::FinishBlockers);
    act(
        &mut g,
        Action::Block {
            blocker: second,
            attacker: v,
        },
    );
    act(&mut g, Action::FinishBlockers);
    assert_eq!(
        g.state.blocks.len(),
        2,
        "generated menace token requires two actual blockers"
    );
}

#[test]
fn msh_b_hydra_entry_and_alone_attack_are_distinct_events() {
    let p = Player::P0;
    let mut g = fresh(p);
    let robot = put(&mut g, p, "HYDRA Assault Robot", Zone::Battlefield);
    let second = put(&mut g, p, "HYDRA Assault Robot", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, second, None);
    decision(&mut g);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(
        g.life(p.other()),
        19,
        "artifact Villain qualifies once, not twice"
    );
    assert_eq!(stats(&g, robot), (1, 3));
    for (name, price) in [
        ("Hawkeye's Bow", [0, 0, 0, 1, 0, 0]),
        ("Kingpin's Enforcers", [0, 0, 3, 0, 0, 0]),
    ] {
        let mut g = fresh(p);
        put(&mut g, p, "HYDRA Assault Robot", Zone::Battlefield);
        let entry = put(&mut g, p, name, Zone::Hand);
        mana(&mut g, p, price);
        cast(&mut g, entry, None);
        decision(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish_targets(&mut g);
        settle(&mut g);
        assert_eq!(g.life(p.other()), 19);
    }
    let mut g = fresh(p);
    let a = put(&mut g, p.other(), "Forest", Zone::Hand);
    let b = put(&mut g, p.other(), "Island", Zone::Hand);
    let enchant = put(&mut g, p, "HYDRA Infiltration", Zone::Hand);
    mana(&mut g, p, [0, 0, 4, 0, 0, 0]);
    cast(&mut g, enchant, None);
    decision(&mut g);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    decision(&mut g);
    act(&mut g, Action::Discard { card: a });
    act(&mut g, Action::Discard { card: b });
    settle(&mut g);
    assert_eq!(g.hand_count(p.other()), 0);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    attacks(&mut g, &[bear]);
    decision(&mut g);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!((g.life(p), g.life(p.other())), (21, 19));
}

#[test]
fn msh_b_equipment_attaches_pays_and_animates() {
    let p = Player::P0;
    for (name, payment, bonus, equip_slot, equip_payment) in [
        (
            "Falcon's Wing Harness",
            [0, 2, 0, 0, 0, 0],
            (1, 1),
            0,
            [0, 3, 0, 0, 0, 0],
        ),
        (
            "Iron Man Armor",
            [0, 0, 0, 0, 0, 3],
            (2, 1),
            1,
            [0, 0, 0, 0, 0, 2],
        ),
    ] {
        let mut g = fresh(p);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let second = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let eq = put(&mut g, p, name, Zone::Hand);
        mana(&mut g, p, payment);
        cast(&mut g, eq, None);
        decision(&mut g);
        target(&mut g, bear);
        finish_targets(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[eq as usize].attached_to,
            Some(reference(&g, bear))
        );
        assert_eq!(stats(&g, bear), (2 + bonus.0, 2 + bonus.1));
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        mana(&mut g, p, equip_payment);
        activate(&mut g, eq, equip_slot, &[Target::Card(second)]);
        settle(&mut g);
        assert_eq!(stats(&g, bear), (2, 2));
        assert_eq!(stats(&g, second), (3 + bonus.0, 3 + bonus.1));
        if name == "Falcon's Wing Harness" {
            for paid in [false, true] {
                let bolt = put(&mut g, p.other(), "Lightning Strike", Zone::Hand);
                g.state.priority = p.other();
                mana(&mut g, p.other(), [0, 0, 0, if paid { 3 } else { 2 }, 0, 0]);
                cast(&mut g, bolt, Some(Target::Card(second)));
                decision(&mut g);
                if paid {
                    choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
                } else {
                    act(&mut g, Action::DeclineResolutionMana);
                }
                settle(&mut g);
                assert_eq!(
                    g.state.objects[second as usize].damage,
                    if paid { 3 } else { 0 }
                );
            }
        }
        if name == "Iron Man Armor" {
            go(&mut g, p.other(), Step::Main1);
            go(&mut g, p, Step::Main1);
            mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
            activate(&mut g, eq, 0, &[]);
            settle(&mut g);
            assert_eq!(stats(&g, eq), (1, 1));
            assert_eq!(g.state.objects[eq as usize].attached_to, None);
            put(&mut g, p, "Futurist Forge", Zone::Battlefield);
            assert_eq!(stats(&g, eq), (2, 2));
            mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
            activate(&mut g, eq, 0, &[]);
            settle(&mut g);
            assert_eq!(
                stats(&g, eq),
                (2, 2),
                "already-creature Equipment does not acquire the bonus twice"
            );
            let ground = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            attacks(&mut g, &[eq]);
            for _ in 0..4 {
                if g.state.pending == Pending::Blockers {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            reject(
                &mut g,
                Action::Block {
                    blocker: ground,
                    attacker: eq,
                },
            );
            act(&mut g, Action::FinishBlockers);
            go(&mut g, p, Step::Main2);
            assert_eq!(g.life(p.other()), 18);
            go(&mut g, p.other(), Step::Main1);
            assert!(!g.is_creature(eq));
        }
    }
    let mut g = fresh(p);
    let creature = put(&mut g, p, "Kingpin's Enforcers", Zone::Battlefield);
    let bow = put(&mut g, p, "Hawkeye's Bow", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
    cast(&mut g, bow, None);
    settle(&mut g);
    mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
    activate(&mut g, bow, 0, &[Target::Card(creature)]);
    settle(&mut g);
    assert_eq!(stats(&g, creature), (3, 3));
    assert!(g.keywords(creature).contains(Keywords::REACH));
    attacks(&mut g, &[creature]);
    settle(&mut g);
    assert_eq!(
        (g.life(p), g.life(p.other())),
        (21, 19),
        "equipped creature is damage actor, so its lifelink applies"
    );
}

#[test]
fn msh_b_tap_untap_aura_and_flash_boundaries() {
    let p = Player::P0;
    let mut g = fresh(p);
    let victim = put(&mut g, p.other(), "Kree Commandos", Zone::Battlefield);
    let ice = put(&mut g, p, "Frozen in Ice", Zone::Hand);
    mana(&mut g, p, [0, 3, 0, 0, 0, 0]);
    cast(&mut g, ice, Some(Target::Card(victim)));
    settle(&mut g);
    assert!(g.state.objects[victim as usize].tapped);
    assert!(!g.keywords(victim).contains(Keywords::FLYING));
    go(&mut g, p.other(), Step::Main1);
    assert!(g.state.objects[victim as usize].tapped);
    for untap in [false, true] {
        let mut g = fresh(p);
        let object = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
        g.state.objects[object as usize].tapped = untap;
        // Flash is exercised during the opponent's turn, not just observed as a flag.
        g.state.active = p.other();
        g.state.priority = p;
        let ant = put(&mut g, p, "Giant-Sized Flying Ant", Zone::Hand);
        mana(&mut g, p, [0, 4, 0, 0, 0, 0]);
        cast(&mut g, ant, None);
        decision(&mut g);
        act(
            &mut g,
            Action::ChooseMode {
                mode: u16::from(untap),
            },
        );
        target(&mut g, object);
        finish_targets(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[object as usize].tapped, !untap);
        assert_eq!(stats(&g, ant), (3, 2));
    }
    let mut g = fresh(p);
    g.state.active = p.other();
    g.state.priority = p;
    let ghost = put(&mut g, p, "Ghost, Spectral Saboteur", Zone::Hand);
    mana(&mut g, p, [0, 0, 3, 0, 0, 0]);
    cast(&mut g, ghost, None);
    settle(&mut g);
    go(&mut g, p, Step::Main1);
    let blocker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    attacks(&mut g, &[ghost]);
    for _ in 0..4 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    reject(
        &mut g,
        Action::Block {
            blocker,
            attacker: ghost,
        },
    );
    act(&mut g, Action::FinishBlockers);
    go(&mut g, p, Step::Main2);
    assert_eq!(g.life(p.other()), 18);
}

#[test]
fn msh_b_power_up_prices_lifetime_and_expiry() {
    let p = Player::P0;
    for (name, slot, price, expected) in [
        ("Hercules, Prince of Power", 0, [0, 0, 0, 0, 5, 0], (4, 4)),
        ("Hulk, Gamma Goliath", 0, [0, 0, 0, 1, 7, 0], (11, 10)),
        ("Human Torch, Johnny Storm", 0, [0, 0, 0, 7, 0, 0], (5, 5)),
        ("Loki Laufeyson", 1, [0, 0, 0, 5, 0, 0], (4, 3)),
    ] {
        let mut g = fresh(p);
        let id = put(&mut g, p, name, Zone::Battlefield);
        go(&mut g, p.other(), Step::Main1);
        go(&mut g, p, Step::Main1);
        mana(&mut g, p, price);
        activate(&mut g, id, slot, &[]);
        settle(&mut g);
        assert_eq!(stats(&g, id), expected);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        mana(&mut g, p, price);
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a,Action::AnnounceActivate{card,ability,..}if *card==id&&*ability==slot)
        ));
        if name == "Hercules, Prince of Power" {
            assert!(g.keywords(id).contains(Keywords::HASTE));
            assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
            for _ in 0..2 {
                let bolt = put(&mut g, p, "Lightning Strike", Zone::Hand);
                mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
                cast(&mut g, bolt, Some(Target::Card(id)));
                settle(&mut g);
            }
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            assert_eq!(g.state.objects[id as usize].damage, 6);
            attacks(&mut g, &[id]);
            assert!(!g.state.objects[id as usize].tapped);
        }
        go(&mut g, p.other(), Step::Main1);
        if name == "Hercules, Prince of Power" {
            assert!(!g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        }
    }
    for other in [false, true] {
        let mut g = fresh(p);
        put(&mut g, p, "Human Torch, Johnny Storm", Zone::Battlefield);
        if other {
            put(&mut g, p, "Iron Lad, Diverging Destiny", Zone::Battlefield);
        }
        let forge = put(&mut g, p, "Futurist Forge", Zone::Hand);
        mana(&mut g, p, [0, 2, 0, 0, 0, 0]);
        cast(&mut g, forge, None);
        if other {
            decision(&mut g);
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            finish_targets(&mut g);
        }
        settle(&mut g);
        assert_eq!(g.life(p.other()), if other { 19 } else { 20 });
    }
    let mut entry = fresh(p);
    let h = put(&mut entry, p, "Hercules, Prince of Power", Zone::Hand);
    mana(&mut entry, p, [0, 0, 0, 0, 3, 0]);
    cast(&mut entry, h, None);
    settle(&mut entry);
    mana(&mut entry, p, [0, 0, 0, 0, 0, 2]);
    activate(&mut entry, h, 0, &[]);
    settle(&mut entry);
    assert_eq!(stats(&entry, h), (4, 4));
    assert_eq!(entry.state.players[p.index()].mana, [0; 6]);
    attacks(&mut entry, &[h]);
    assert!(!entry.state.objects[h as usize].tapped);
    // Hulk reduces another creature's activation, not its own.
    let mut g = fresh(p);
    put(&mut g, p, "Hulk, Gamma Goliath", Zone::Battlefield);
    let hero = put(&mut g, p, "Hercules, Prince of Power", Zone::Battlefield);
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    mana(&mut g, p, [0, 0, 0, 0, 2, 0]);
    activate(&mut g, hero, 0, &[]);
    settle(&mut g);
    assert_eq!(stats(&g, hero), (4, 4));
    assert_eq!(g.state.players[0].mana, [0; 6]);
}

#[test]
fn msh_b_kang_schedules_exact_power_up_free_turn() {
    let p = Player::P1;
    let mut g = fresh(p);
    let kang = put(&mut g, p, "Kang the Conqueror", Zone::Battlefield);
    let herc = put(&mut g, p, "Hercules, Prince of Power", Zone::Battlefield);
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    mana(&mut g, p, [0, 8, 0, 0, 0, 0]);
    activate(&mut g, kang, 0, &[]);
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(&g, kang), (5, 6));
    let turn = g.state.turn;
    // Continue through cleanup: the next turn is ours again.
    go(&mut g, p, Step::Main2);
    go(&mut g, p, Step::Upkeep);
    assert!(g.state.turn > turn);
    assert!(g.power_up_prohibited_this_turn());
    mana(&mut g, p, [0, 0, 0, 0, 5, 0]);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==herc))
    );
    go(&mut g, p.other(), Step::Main1);
    assert!(!g.power_up_prohibited_this_turn());
}

#[test]
fn msh_b_hulkling_rechecks_greater_power_or_toughness() {
    let p = Player::P0;
    for (name, expected) in [
        ("Grizzly Bears", (2, 3)),
        ("Hill Giant", (3, 4)),
        ("Hydraulic Helper", (2, 3)),
        ("Iron Man, Master of Machines", (3, 4)),
    ] {
        let mut g = fresh(p);
        let h = put(&mut g, p, "Hulkling, Burgeoning Bruiser", Zone::Battlefield);
        let entrant = put(&mut g, p, name, Zone::Hand);
        mana(
            &mut g,
            p,
            if name == "Iron Man, Master of Machines" {
                [0, 3, 0, 1, 0, 0]
            } else if name == "Hydraulic Helper" {
                [0, 2, 0, 0, 0, 0]
            } else if name == "Hill Giant" {
                [0, 0, 0, 4, 0, 0]
            } else {
                [0, 0, 0, 0, 2, 0]
            },
        );
        cast(&mut g, entrant, None);
        settle(&mut g);
        assert_eq!(stats(&g, h), expected);
        attacks(&mut g, &[h]);
        assert!(!g.state.objects[h as usize].tapped);
    }
    let mut g = fresh(p);
    let h = put(&mut g, p, "Hulkling, Burgeoning Bruiser", Zone::Battlefield);
    let entrant = put(&mut g, p, "Hill Giant", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 4, 0, 0]);
    cast(&mut g, entrant, None);
    // Let the creature resolve, then pump Hulkling before its intervening-if trigger.
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    let pump = put(&mut g, p, "Giant Growth", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
    cast(&mut g, pump, Some(Target::Card(h)));
    settle(&mut g);
    assert_eq!(stats(&g, h), (5, 6));
}

#[test]
fn msh_b_top_library_look_draw_and_land_allowance() {
    let p = Player::P0;
    for artifact in [false, true] {
        let mut g = fresh(p);
        let lad = put(&mut g, p, "Iron Lad, Diverging Destiny", Zone::Battlefield);
        let top = put(
            &mut g,
            p,
            if artifact { "Futurist Forge" } else { "Forest" },
            Zone::Library,
        );
        assert_eq!(g.visible_library_top(p, p), Some(reference(&g, top)));
        assert_eq!(g.visible_library_top(p.other(), p), None);
        activate(&mut g, lad, 0, &[]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[top as usize].zone,
            if artifact { Zone::Hand } else { Zone::Library }
        );
        assert_eq!(g.hand_count(p), usize::from(artifact));
    }
    let mut empty = fresh(p);
    let lad = put(
        &mut empty,
        p,
        "Iron Lad, Diverging Destiny",
        Zone::Battlefield,
    );
    empty.state.players[p.index()].library.clear();
    activate(&mut empty, lad, 0, &[]);
    settle(&mut empty);
    assert_eq!(empty.outcome(), None);
    assert_eq!(empty.hand_count(p), 0);
    let mut g = fresh(p);
    let k = put(&mut g, p, "Ka-Zar of the Savage Land", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 5, 0]);
    cast(&mut g, k, None);
    settle(&mut g);
    let z = token(&g, "Zabu");
    assert!(g.object_view(z).is_legendary());
    assert!(g.has_subtype(z, "Cat"));
    assert_eq!(g.object_view(z).colors, 16);
    assert_eq!(stats(&g, z), (2, 2));
    let top = put(&mut g, p, "Island", Zone::Library);
    assert_eq!(g.visible_library_top(p, p), Some(reference(&g, top)));
    act(&mut g, Action::PlayLand { card: top });
    settle(&mut g);
    assert_eq!(stats(&g, z), (3, 3));
    let second = put(&mut g, p, "Forest", Zone::Library);
    reject(&mut g, Action::PlayLand { card: second });
}

#[test]
fn msh_b_exile_draw_permissions_and_stun() {
    let p = Player::P0;
    let mut g = fresh(p);
    let hex = put(&mut g, p, "Hex Magic", Zone::Hand);
    let land = put(&mut g, p, "Island", Zone::Hand);
    let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
    let saved = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 3, 0, 0]);
    cast(&mut g, hex, None);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 3);
    assert_eq!(g.state.objects[land as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
    act(&mut g, Action::PlayLand { card: land });
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
    act(&mut g, Action::AnnounceExile { card: spell });
    target(&mut g, bear);
    finish_targets(&mut g);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, bear), (5, 5));
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceExile { card: saved })
    );
    go(&mut g, p.other(), Step::Main1);
    g.state.priority = p;
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    reject(&mut g, Action::AnnounceExile { card: saved });
    let mut g = fresh(p);
    let j = put(&mut g, p, "Jessica Jones, Private Eye", Zone::Battlefield);
    let a = put(&mut g, p, "Island", Zone::Library);
    let b = put(&mut g, p, "Giant Growth", Zone::Library);
    activate(&mut g, j, 0, &[]);
    settle(&mut g);
    assert_eq!(g.counter_count(j, CounterKind::Stun), 1);
    assert_eq!(g.state.objects[a as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[b as usize].zone, Zone::Exile);
    act(&mut g, Action::PlayLand { card: a });
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    assert!(g.state.objects[j as usize].tapped);
    assert_eq!(g.counter_count(j, CounterKind::Stun), 0);
    assert!(
        !g.legal_actions()
            .contains(&Action::AnnounceExile { card: b })
    );
}

fn nuts(g: &mut Game, p: Player, recipient: CardId) {
    let spell = put(g, p, "Go Nuts!", Zone::Hand);
    mana(g, p, [0, 0, 0, 0, 1, 0]);
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseTeamwork { pay: false });
    act(g, Action::ChooseMode { mode: 0 });
    target(g, recipient);
    finish_targets(g);
    pay(g);
}

#[test]
fn msh_b_life_counters_optional_wall_and_once_limit() {
    let p = Player::P0;
    let mut g = fresh(p);
    let feast = put(&mut g, p, "Heroic Feast", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 3, 0]);
    cast(&mut g, feast, None);
    settle(&mut g);
    let food = token(&g, "Food Token");
    let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let b = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
    activate(&mut g, food, 0, &[]);
    decision(&mut g);
    target(&mut g, a);
    target(&mut g, b);
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(g.life(p), 23);
    assert_eq!(stats(&g, a), (3, 3));
    assert_eq!(stats(&g, b), (4, 4));
    assert_eq!(g.state.objects[food as usize].zone, Zone::Outside);
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    let tower = put(&mut g, p, "Fisk Tower", Zone::Hand);
    act(&mut g, Action::PlayLand { card: tower });
    decision(&mut g);
    target(&mut g, a);
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(g.life(p), 24);
    assert_eq!(
        stats(&g, a),
        (4, 4),
        "permanent Feast still triggers next turn for the one life gained"
    );
    assert_eq!(stats(&g, b), (4, 4));
    let mut g = fresh(p);
    let sue = put(&mut g, p, "Invisible Woman, Sue Storm", Zone::Battlefield);
    let hero = put(&mut g, p, "Iron Lad, Diverging Destiny", Zone::Battlefield);
    nuts(&mut g, p, hero);
    decision(&mut g);
    act(&mut g, Action::ChooseOptional { accept: true });
    settle(&mut g);
    let wall = token(&g, "Wall Token");
    assert!(g.has_subtype(wall, "Wall"));
    assert_eq!(g.object_view(wall).colors, 0);
    assert_eq!(stats(&g, wall), (0, 4));
    assert!(g.keywords(wall).contains(Keywords::DEFENDER));
    nuts(&mut g, p, sue);
    settle(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.is_token())
            .count(),
        1,
        "self counters are excluded"
    );
    let mut batch = fresh(p);
    put(
        &mut batch,
        p,
        "Invisible Woman, Sue Storm",
        Zone::Battlefield,
    );
    let a = put(
        &mut batch,
        p,
        "Iron Lad, Diverging Destiny",
        Zone::Battlefield,
    );
    let b = put(
        &mut batch,
        p,
        "Hercules, Prince of Power",
        Zone::Battlefield,
    );
    let feast = put(&mut batch, p, "Heroic Feast", Zone::Hand);
    mana(&mut batch, p, [0, 0, 0, 0, 3, 0]);
    cast(&mut batch, feast, None);
    settle(&mut batch);
    let food = token(&batch, "Food Token");
    mana(&mut batch, p, [0, 0, 0, 0, 0, 2]);
    activate(&mut batch, food, 0, &[]);
    decision(&mut batch);
    target(&mut batch, a);
    reject(
        &mut batch,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    target(&mut batch, b);
    finish_targets(&mut batch);
    decision(&mut batch);
    act(&mut batch, Action::ChooseOptional { accept: true });
    settle(&mut batch);
    assert_eq!(
        batch
            .state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.is_token())
            .count(),
        1
    );
    assert_eq!(stats(&batch, a), (3, 3));
    assert_eq!(stats(&batch, b), (4, 4));
    let mut g = fresh(p);
    let knight = put(&mut g, p, "Knight of Wundagore", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    nuts(&mut g, p, other);
    settle(&mut g);
    assert_eq!(stats(&g, knight), (3, 2));
    nuts(&mut g, p, other);
    settle(&mut g);
    assert_eq!(stats(&g, knight), (3, 2));
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    nuts(&mut g, p, other);
    settle(&mut g);
    assert_eq!(stats(&g, knight), (4, 3));
    let mut g = fresh(p);
    let self_knight = put(&mut g, p, "Knight of Wundagore", Zone::Battlefield);
    nuts(&mut g, p, self_knight);
    settle(&mut g);
    assert_eq!(
        stats(&g, self_knight),
        (3, 2),
        "its own counter does not trigger another counter"
    );
    let kid = put(&mut g, p, "Kid Loki", Zone::Battlefield);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    nuts(&mut g, p, bear);
    settle(&mut g);
    assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
    assert!(!g.keywords(kid).contains(Keywords::HEXPROOF));
    let forge = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    mana(&mut g, p, [0, 4, 0, 0, 0, 0]);
    activate(&mut g, forge, 0, &[]);
    settle(&mut g);
    assert_eq!(stats(&g, kid), (2, 2));
    assert!(g.keywords(kid).contains(Keywords::HEXPROOF));
    go(&mut g, p.other(), Step::Main1);
    assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
}

#[test]
fn msh_b_cast_target_events_copy_and_grant_without_recasting() {
    let p = Player::P0;
    let mut g = fresh(p);
    let fin = put(&mut g, p, "Fin Fang Foom", Zone::Battlefield);
    let other = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    let artifact = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    let spell = put(&mut g, p, "I Am Iron Man", Zone::Hand);
    mana(&mut g, p, [0, 3, 0, 0, 0, 0]);
    cast(&mut g, spell, Some(Target::Card(artifact)));
    decision(&mut g);
    act(
        &mut g,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(other),
        },
    );
    act(&mut g, Action::FinishStackTargets);
    settle(&mut g);
    assert_eq!(stats(&g, other), (4, 4));
    assert_eq!(stats(&g, fin), (5, 7));
    assert_eq!(g.hand_count(p), 2);
    assert_eq!(stats(&g, artifact), (4, 4));
    let mut land_game = fresh(p);
    let fin = put(&mut land_game, p, "Fin Fang Foom", Zone::Battlefield);
    let land = put(&mut land_game, p, "Soulstone Sanctuary", Zone::Battlefield);
    mana(&mut land_game, p, [0, 0, 0, 0, 0, 4]);
    activate(&mut land_game, land, 1, &[]);
    settle(&mut land_game);
    let growth = put(&mut land_game, p, "Giant Growth", Zone::Hand);
    mana(&mut land_game, p, [0, 0, 0, 0, 1, 0]);
    cast(&mut land_game, growth, Some(Target::Card(land)));
    decision(&mut land_game);
    act(&mut land_game, Action::FinishStackTargets);
    settle(&mut land_game);
    assert_eq!(stats(&land_game, fin), (5, 7));
    assert_eq!(stats(&land_game, land), (9, 9));
    let mut g = fresh(p);
    let loki = put(&mut g, p, "Loki Laufeyson", Zone::Battlefield);
    mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
    activate(&mut g, loki, 0, &[]);
    settle(&mut g);
    let artifact = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    let high = put(&mut g, p, "I Am Iron Man", Zone::Hand);
    mana(&mut g, p, [0, 3, 0, 0, 0, 0]);
    cast(&mut g, high, Some(Target::Card(artifact)));
    settle(&mut g);
    assert_eq!(
        g.hand_count(p),
        1,
        "MV3 exceeds current power2 and must not consume delayed copy"
    );
    let strike = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, strike, Some(Target::Player(p.other())));
    decision(&mut g);
    act(&mut g, Action::FinishStackTargets);
    settle(&mut g);
    assert_eq!(g.life(p.other()), 14);
    let second = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, second, Some(Target::Player(p.other())));
    settle(&mut g);
    assert_eq!(
        g.life(p.other()),
        11,
        "only the next qualifying cast is copied"
    );
    let mut g = fresh(p);
    let fist = put(&mut g, p, "Iron Fist, Living Weapon", Zone::Battlefield);
    put(&mut g, p, "Loki, God of Mischief", Zone::Battlefield);
    let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
    cast(&mut g, growth, Some(Target::Card(fist)));
    settle(&mut g);
    assert_eq!(
        g.hand_count(p),
        0,
        "targeted spell is not a targeted ability"
    );
    choose(&mut g, |a| {
        matches!(a, Action::AnnounceSelfGrantedAbility { .. })
    });
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(fist)
    }));
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.life(p.other()), 15);
    assert_eq!(
        g.hand_count(p),
        1,
        "own ability targeting a player triggers Loki"
    );
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    assert!(!g.state.objects[fist as usize].tapped);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::AnnounceSelfGrantedAbility { .. })),
        "Iron Fist's granted tap activation must expire after cleanup"
    );
    let mut g = fresh(p);
    let mischief = put(&mut g, p, "Loki, God of Mischief", Zone::Battlefield);
    let bow = put(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
    activate(&mut g, bow, 0, &[Target::Card(bear)]);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 1);
    mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
    activate(&mut g, bow, 0, &[Target::Card(mischief)]);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 1);
    let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
    cast(&mut g, growth, Some(Target::Card(bear)));
    settle(&mut g);
    assert_eq!(
        g.hand_count(p),
        1,
        "spells do not satisfy ability target observer"
    );
    go(&mut g, p.other(), Step::Main1);
    go(&mut g, p, Step::Main1);
    let before = g.hand_count(p);
    mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
    activate(&mut g, bow, 0, &[Target::Card(bear)]);
    settle(&mut g);
    assert_eq!(g.hand_count(p), before + 1);
}

#[test]
fn msh_b_sacrifice_discard_and_reflexive_targets() {
    let p = Player::P0;
    for route in 0..3 {
        let mut g = fresh(p);
        let artifact = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
        let discard = put(&mut g, p, "Forest", Zone::Hand);
        let warrior = put(&mut g, p, "K'un-Lun Warrior", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
        cast(&mut g, warrior, None);
        decision(&mut g);
        match route {
            0 => act(&mut g, Action::DeclineEffectPayment),
            1 => {
                let object = reference(&g, artifact);
                act(&mut g, Action::Sacrifice { object });
            }
            _ => act(&mut g, Action::Discard { card: discard }),
        };
        settle(&mut g);
        assert_eq!(g.hand_count(p), if route == 1 { 2 } else { 1 });
        assert_eq!(
            g.state.objects[artifact as usize].zone,
            if route == 1 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.state.objects[discard as usize].zone,
            if route == 2 {
                Zone::Graveyard
            } else {
                Zone::Hand
            }
        );
    }
    for name in ["Futurist Forge", "HYDRA Infiltration"] {
        let mut g = fresh(p);
        let gorilla = put(&mut g, p, "Guerrilla Gorilla", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), name, Zone::Battlefield);
        let creature = put(&mut g, p.other(), "Hydraulic Helper", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: gorilla,
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
        target(&mut g, enemy);
        finish_targets(&mut g);
        pay(&mut g);
        assert_eq!(g.state.objects[gorilla as usize].zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    }
    for cost_name in ["Futurist Forge", "Grizzly Bears"] {
        let mut g = fresh(p);
        let enforcer = put(&mut g, p, "Kingpin's Enforcers", Zone::Battlefield);
        let sacrifice = put(&mut g, p, cost_name, Zone::Battlefield);
        mana(&mut g, p, [0, 0, 3, 0, 0, 0]);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: enforcer,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = reference(&g, sacrifice);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1, "{cost_name}");
        assert_eq!(
            g.state.objects[sacrifice as usize].zone,
            Zone::Graveyard,
            "{cost_name}"
        );
    }
    let mut g = fresh(p);
    put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let sacrifice = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Futurist Forge", Zone::Battlefield);
    let kill = put(&mut g, p, "Killmonger, Scourge of Wakanda", Zone::Hand);
    mana(&mut g, p, [0, 0, 3, 0, 1, 0]);
    cast(&mut g, kill, None);
    decision(&mut g);
    assert!(!g.legal_actions().contains(&Action::Sacrifice {
        object: reference(&g, kill)
    }));
    assert_eq!(stats(&g, kill), (3, 3));
    let object = reference(&g, sacrifice);
    act(&mut g, Action::Sacrifice { object });
    decision(&mut g);
    target(&mut g, enemy);
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, kill), (5, 4));
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
}

#[test]
fn msh_b_sneak_and_attack_reanimation_use_real_combat() {
    let p = Player::P0;
    let mut g = fresh(p);
    let attacker = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let elektra = put(&mut g, p, "Elektra, Daughter of the Hand", Zone::Hand);
    attacks(&mut g, &[attacker]);
    for _ in 0..4 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(&mut g, Action::FinishBlockers);
    mana(&mut g, p, [0, 0, 3, 0, 0, 0]);
    act(
        &mut g,
        Action::AnnounceSneak {
            card: elektra,
            permission: None,
        },
    );
    let object = reference(&g, attacker);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    decision(&mut g);
    target(&mut g, enemy);
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[attacker as usize].zone, Zone::Hand);
    assert!(g.state.objects[elektra as usize].tapped);
    assert!(g.state.attackers.iter().any(|a| a.card == elektra));
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    let mut g = fresh(p);
    let reaper = put(
        &mut g,
        p,
        "Grim Reaper, Lethal Legionnaire",
        Zone::Battlefield,
    );
    let dead = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    attacks(&mut g, &[reaper]);
    mana(&mut g, p, [0, 0, 4, 0, 0, 0]);
    decision(&mut g);
    choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
    decision(&mut g);
    target(&mut g, dead);
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[dead as usize].zone, Zone::Battlefield);
    assert!(g.state.objects[dead as usize].tapped);
    assert!(g.state.attackers.iter().any(|a| a.card == dead));
    assert_eq!(g.counter_count(dead, CounterKind::Finality), 1);
    let bolt = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, bolt, Some(Target::Card(dead)));
    settle(&mut g);
    assert_eq!(g.state.objects[dead as usize].zone, Zone::Exile);
}

#[test]
fn msh_b_hellcat_return_is_blank_and_justice_uses_owner() {
    let p = Player::P0;
    let mut g = fresh(p);
    let cat = put(&mut g, p, "Hellcat, Undying Vigilante", Zone::Battlefield);
    let bolt = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, bolt, Some(Target::Card(cat)));
    settle(&mut g);
    assert_eq!(g.state.objects[cat as usize].zone, Zone::Battlefield);
    assert_eq!(stats(&g, cat), (3, 3));
    assert!(g.keywords(cat).contains(Keywords::HASTE));
    attacks(&mut g, &[cat]);
    let bolt = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, bolt, Some(Target::Card(cat)));
    settle(&mut g);
    assert_eq!(
        g.state.objects[cat as usize].zone,
        Zone::Graveyard,
        "lost death ability does not return twice"
    );
    let mut stolen = fresh(p);
    let cat = put(
        &mut stolen,
        p.other(),
        "Hellcat, Undying Vigilante",
        Zone::Battlefield,
    );
    stolen.state.objects[cat as usize].controller = p;
    let bolt = put(&mut stolen, p, "Lightning Strike", Zone::Hand);
    mana(&mut stolen, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut stolen, bolt, Some(Target::Card(cat)));
    settle(&mut stolen);
    assert_eq!(stolen.state.objects[cat as usize].controller, p.other());
    assert_eq!(stats(&stolen, cat), (3, 3));
    let mut g = fresh(p);
    let own = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    let j = put(&mut g, p, "Justice, Vance Astrovik", Zone::Hand);
    mana(&mut g, p, [0, 3, 0, 0, 0, 0]);
    cast(&mut g, j, None);
    decision(&mut g);
    target(&mut g, own);
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(g.state.objects[own as usize].zone, Zone::Hand);
    assert_eq!(stats(&g, j), (3, 3));
    for self_target in [false, true] {
        let mut optional = fresh(p);
        let j = put(&mut optional, p, "Justice, Vance Astrovik", Zone::Hand);
        mana(&mut optional, p, [0, 3, 0, 0, 0, 0]);
        cast(&mut optional, j, None);
        decision(&mut optional);
        if self_target {
            target(&mut optional, j);
        }
        finish_targets(&mut optional);
        settle(&mut optional);
        assert_eq!(
            optional.state.objects[j as usize].zone,
            if self_target {
                Zone::Hand
            } else {
                Zone::Battlefield
            }
        );
    }
    // Returning an opponent's permanent does not satisfy the controlled lookback.
    let enemy = put(&mut g, p.other(), "Futurist Forge", Zone::Battlefield);
    let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
    mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
    let creature = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    cast(&mut g, bounce, Some(Target::Card(creature)));
    settle(&mut g);
    assert_eq!(stats(&g, j), (3, 3));
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Battlefield);
    let stolen = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    g.state.objects[stolen as usize].controller = p;
    let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
    mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
    cast(&mut g, bounce, Some(Target::Card(stolen)));
    settle(&mut g);
    assert_eq!(g.state.objects[stolen as usize].zone, Zone::Hand);
    assert_eq!(g.state.objects[stolen as usize].owner, p.other());
    assert_eq!(stats(&g, j), (4, 4));
}

#[test]
fn msh_b_klaw_victim_reveals_controller_discards() {
    let p = Player::P0;
    let mut g = fresh(p);
    put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let a = put(&mut g, p.other(), "Island", Zone::Hand);
    let b = put(&mut g, p.other(), "Giant Growth", Zone::Hand);
    let unrevealed = put(&mut g, p.other(), "Forest", Zone::Hand);
    let klaw = put(&mut g, p, "Klaw, Sonic Subjugator", Zone::Hand);
    mana(&mut g, p, [0, 0, 3, 0, 0, 0]);
    cast(&mut g, klaw, None);
    decision(&mut g);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    decision(&mut g);
    assert_eq!(g.actor(), Some(p.other()));
    for id in [a, b] {
        let object = reference(&g, id);
        act(
            &mut g,
            Action::ChooseHandCard {
                object: Some(object),
            },
        );
    }
    assert_eq!(g.actor(), Some(p));
    assert!(!g.legal_actions().contains(&Action::ChooseHandCard {
        object: Some(reference(&g, unrevealed))
    }));
    let object = reference(&g, b);
    act(
        &mut g,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
    settle(&mut g);
    assert_eq!(g.state.objects[b as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[a as usize].zone, Zone::Hand);
}

#[test]
fn msh_b_connive_draw_ordinals_and_life_payment() {
    let p = Player::P0;
    let mut g = fresh(p);
    let kang = put(&mut g, p, "Kang, Temporal Tyrant", Zone::Battlefield);
    let discard = put(&mut g, p, "Lightning Strike", Zone::Hand);
    attacks(&mut g, &[kang]);
    decision(&mut g);
    act(&mut g, Action::Discard { card: discard });
    settle(&mut g);
    assert_eq!(stats(&g, kang), (4, 5));
    assert_eq!(g.life(p.other()), 20);
    let forge = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    mana(&mut g, p, [0, 4, 0, 0, 0, 0]);
    activate(&mut g, forge, 0, &[]);
    settle(&mut g);
    assert_eq!((g.life(p), g.life(p.other())), (21, 19));
    let mut g = fresh(p);
    let leader = put(&mut g, p, "Leader, Super-Genius", Zone::Battlefield);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let discard = put(&mut g, p, "Lightning Strike", Zone::Hand);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    decision(&mut g);
    target(&mut g, bear);
    finish_targets(&mut g);
    decision(&mut g);
    assert_eq!(g.hand_count(p), 3);
    act(&mut g, Action::Discard { card: discard });
    settle(&mut g);
    assert_eq!(stats(&g, bear), (3, 3));
    assert_eq!(g.hand_count(p), 2);
    assert_eq!(stats(&g, leader), (1, 3));
    let mut g = fresh(p);
    let modok = put(&mut g, p, "M.O.D.O.K.", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let discard = put(&mut g, p, "Lightning Strike", Zone::Hand);
    assert_eq!(stats(&g, enemy), (1, 1));
    activate(&mut g, modok, 0, &[]);
    assert_eq!(g.life(p), 17);
    decision(&mut g);
    act(&mut g, Action::Discard { card: discard });
    settle(&mut g);
    assert_eq!(stats(&g, modok), (3, 3));
    go(&mut g, p.other(), Step::Main1);
    g.state.priority = p;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==modok))
    );
}

#[test]
fn msh_b_transform_faces_have_distinct_rules_and_damage() {
    let p = Player::P0;
    let mut g = fresh(p);
    let jen = put(
        &mut g,
        p,
        "Jennifer Walters // The Sensational She-Hulk",
        Zone::Hand,
    );
    mana(&mut g, p, [2, 0, 0, 0, 0, 0]);
    cast(&mut g, jen, None);
    settle(&mut g);
    assert_eq!(stats(&g, jen), (2, 3));
    let enemyspell = put(&mut g, p.other(), "Lightning Strike", Zone::Hand);
    mana(&mut g, p.other(), [0, 0, 0, 2, 0, 0]);
    g.state.priority = p.other();
    reject(&mut g, Action::AnnounceCast { card: enemyspell });
    g.state.priority = p;
    g.state.active = p.other();
    mana(&mut g, p, [2, 0, 0, 0, 4, 0]);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: jen,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    g.state.active = p;
    mana(&mut g, p, [2, 0, 0, 0, 4, 0]);
    activate(&mut g, jen, 0, &[]);
    settle(&mut g);
    assert_eq!(stats(&g, jen), (6, 6));
    assert!(g.keywords(jen).contains(Keywords::REACH));
    assert!(g.keywords(jen).contains(Keywords::TRAMPLE));
    assert_eq!(g.object_name(jen), "The Sensational She-Hulk");
    g.state.priority = p.other();
    reject(&mut g, Action::AnnounceCast { card: enemyspell });
    g.state.priority = p;
    let bolt = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, bolt, Some(Target::Card(jen)));
    decision(&mut g);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    decision(&mut g);
    act(&mut g, Action::ChooseOptional { accept: true });
    settle(&mut g);
    assert_eq!(g.life(p.other()), 17);
    let victim = put(&mut g, p, "Kree Sentinel", Zone::Battlefield);
    let second = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, second, Some(Target::Card(victim)));
    for _ in 0..16 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            break;
        }
        match g.state.pending {
            Pending::Priority => act(&mut g, Action::Pass),
            Pending::Targets { .. } => {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                finish_targets(&mut g);
            }
            _ => panic!("second damage event must not offer an unused optional action"),
        }
    }
    assert_eq!(g.life(p.other()), 17);
    let mut draws = fresh(p);
    put(
        &mut draws,
        p,
        "King T'Challa // Black Panther, Hope Enduring",
        Zone::Battlefield,
    );
    let forge = put(&mut draws, p.other(), "Futurist Forge", Zone::Battlefield);
    draws.state.priority = p.other();
    mana(&mut draws, p.other(), [0, 4, 0, 0, 0, 0]);
    activate(&mut draws, forge, 0, &[]);
    settle(&mut draws);
    assert_eq!(draws.hand_count(p.other()), 2);
    assert_eq!(draws.hand_count(p), 1);
    let mut g = fresh(p);
    g.state.active = p.other();
    g.state.priority = p;
    let king = put(
        &mut g,
        p,
        "King T'Challa // Black Panther, Hope Enduring",
        Zone::Hand,
    );
    mana(&mut g, p, [1, 2, 0, 0, 0, 0]);
    cast(&mut g, king, None);
    settle(&mut g);
    assert_eq!(stats(&g, king), (3, 2));
    go(&mut g, p, Step::Main1);
    mana(&mut g, p, [1, 5, 0, 0, 0, 0]);
    activate(&mut g, king, 0, &[]);
    settle(&mut g);
    assert_eq!(stats(&g, king), (3, 3));
    let bolt = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, bolt, Some(Target::Card(king)));
    settle(&mut g);
    assert_eq!(g.state.objects[king as usize].damage, 0);
    let before = g.hand_count(p);
    attacks(&mut g, &[king]);
    go(&mut g, p, Step::Main2);
    assert_eq!(g.life(p.other()), 14);
    assert_eq!(
        g.hand_count(p),
        before + 2,
        "one draw for each double-strike combat hit"
    );
}

#[test]
fn msh_b_artifact_live_stats_restricted_mana_and_improvise() {
    let p = Player::P0;
    let mut g = fresh(p);
    let man = put(&mut g, p, "Iron Man, Master of Machines", Zone::Battlefield);
    assert_eq!(stats(&g, man), (1, 4));
    let forge = put(&mut g, p, "Futurist Forge", Zone::Hand);
    mana(&mut g, p, [0, 2, 0, 0, 0, 0]);
    cast(&mut g, forge, None);
    settle(&mut g);
    assert_eq!(stats(&g, man), (2, 4));
    let before = g.hand_count(p);
    attacks(&mut g, &[man]);
    settle(&mut g);
    assert_eq!(g.hand_count(p), before + 1);
    assert!(!g.state.objects[man as usize].tapped);
    let mut g = fresh(p);
    let helper = put(&mut g, p, "Hydraulic Helper", Zone::Battlefield);
    mana(&mut g, p, [0, 0, 0, 0, 0, 1]);
    produce(&mut g, helper, [0, 1, 0, 0, 0, 0]);
    let opt = put(&mut g, p, "Opt", Zone::Hand);
    reject(&mut g, Action::AnnounceCast { card: opt });
    let forge = put(&mut g, p, "Futurist Forge", Zone::Hand);
    cast(&mut g, forge, None);
    settle(&mut g);
    assert_eq!(g.state.objects[forge as usize].zone, Zone::Battlefield);
    // Noncreature artifacts become 4/4; the cantrip remains one ordinary draw.
    let spell = put(&mut g, p, "I Am Iron Man", Zone::Hand);
    mana(&mut g, p, [0, 3, 0, 0, 0, 0]);
    let before = g.hand_count(p);
    cast(&mut g, spell, Some(Target::Card(forge)));
    settle(&mut g);
    assert_eq!(stats(&g, forge), (4, 4));
    assert!(g.keywords(forge).contains(Keywords::FLYING));
    assert_eq!(g.hand_count(p), before);
    go(&mut g, p.other(), Step::Main1);
    assert!(!g.is_creature(forge));
    let mut g = fresh(p);
    let heart = put(&mut g, p, "Ironheart, Clever Champion", Zone::Hand);
    let mut rocks = Vec::new();
    for _ in 0..4 {
        rocks.push(put(&mut g, p, "Futurist Forge", Zone::Battlefield));
    }
    mana(&mut g, p, [0, 1, 0, 0, 0, 0]);
    act(&mut g, Action::AnnounceCast { card: heart });
    act(&mut g, Action::BeginImprovise);
    for id in rocks {
        let object = reference(&g, id);
        act(&mut g, Action::ChooseImproviser { object });
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, heart), (3, 4));
    let freshrock = put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    let strike = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 1, 0, 0]);
    act(&mut g, Action::AnnounceCast { card: strike });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    act(&mut g, Action::BeginImprovise);
    let object = reference(&g, freshrock);
    act(&mut g, Action::ChooseImproviser { object });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.life(p.other()), 17);
    assert!(g.state.objects[freshrock as usize].tapped);
    put(&mut g, p, "Futurist Forge", Zone::Battlefield);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
    reject(&mut g, Action::AnnounceCast { card: bear });
}

#[test]
fn msh_b_damage_replacement_arrows_and_alone_attack() {
    let p = Player::P0;
    let mut g = fresh(p);
    put(&mut g, p, "Hawkeye, Young Avenger", Zone::Battlefield);
    let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 2, 0, 0]);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    settle(&mut g);
    assert_eq!(g.life(p.other()), 15);
    let mut g = fresh(p);
    let luke = put(&mut g, p, "Luke Cage, Power Man", Zone::Battlefield);
    attacks(&mut g, &[luke]);
    settle(&mut g);
    assert_eq!(stats(&g, luke), (4, 5));
    assert!(g.keywords(luke).contains(Keywords::INDESTRUCTIBLE));
    go(&mut g, p, Step::Main2);
    assert_eq!(g.life(p.other()), 16);
    go(&mut g, p.other(), Step::Main1);
    assert_eq!(stats(&g, luke), (2, 5));
    assert!(!g.keywords(luke).contains(Keywords::INDESTRUCTIBLE));
    let mut multiple = fresh(p);
    let luke = put(&mut multiple, p, "Luke Cage, Power Man", Zone::Battlefield);
    let bear = put(&mut multiple, p, "Grizzly Bears", Zone::Battlefield);
    put(&mut multiple, p, "HYDRA Infiltration", Zone::Battlefield);
    attacks(&mut multiple, &[luke, bear]);
    settle(&mut multiple);
    assert_eq!(stats(&multiple, luke), (2, 5));
    assert_eq!((multiple.life(p), multiple.life(p.other())), (20, 20));
    let mut g = fresh(p);
    let marksman = put(&mut g, p, "Hawkeye, Master Marksman", Zone::Battlefield);
    let discard = put(&mut g, p, "Forest", Zone::Hand);
    let blocker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    attacks(&mut g, &[marksman]);
    mana(&mut g, p, [0, 0, 0, 0, 0, 3]);
    decision(&mut g);
    act(&mut g, Action::ChooseX { value: 3 });
    choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
    decision(&mut g);
    act(&mut g, Action::ChooseMode { mode: 7 });
    target(&mut g, blocker);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    finish_targets(&mut g);
    decision(&mut g);
    act(&mut g, Action::Discard { card: discard });
    settle(&mut g);
    assert_eq!(g.life(p.other()), 18);
    assert!(g.keywords(blocker).contains(Keywords::CANT_BLOCK));
    assert_eq!(g.hand_count(p), 1);
    for _ in 0..4 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    reject(
        &mut g,
        Action::Block {
            blocker,
            attacker: marksman,
        },
    );
    act(&mut g, Action::FinishBlockers);
}

#[test]
fn msh_b_control_duration_and_combat_only_target_damage() {
    let p = Player::P0;
    for long in [false, true] {
        let mut g = fresh(p);
        if long {
            put(&mut g, p, "Kree Sentinel", Zone::Battlefield);
        }
        let victim = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.objects[victim as usize].tapped = true;
        let spell = put(&mut g, p, "Evil's Thrall", Zone::Hand);
        mana(&mut g, p, [0, 0, 0, 3, 0, 0]);
        cast(&mut g, spell, Some(Target::Card(victim)));
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].controller, p);
        assert!(!g.state.objects[victim as usize].tapped);
        assert!(g.keywords(victim).contains(Keywords::HASTE));
        go(&mut g, p.other(), Step::Main1);
        assert_eq!(
            g.state.objects[victim as usize].controller,
            if long { p } else { p.other() }
        );
        assert!(!g.keywords(victim).contains(Keywords::HASTE));
        if long {
            go(&mut g, p, Step::Main1);
            go(&mut g, p.other(), Step::Main1);
            assert_eq!(g.state.objects[victim as usize].controller, p.other());
        }
    }
    for teamwork in [false, true] {
        let mut g = fresh(p);
        let attacker = put(&mut g, p, "Savage Land Dinosaur", Zone::Battlefield);
        let idle = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let teammate = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Helicarrier Strike", Zone::Hand);
        attacks(&mut g, &[attacker]);
        mana(&mut g, p, [1, 0, 0, 0, 0, 0]);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseTeamwork { pay: teamwork });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(idle),
            },
        );
        target(&mut g, attacker);
        finish_targets(&mut g);
        if teamwork {
            teamwork_payment(&mut g, teammate);
        } else {
            pay(&mut g);
        }
        settle(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Battlefield);
        assert_eq!(
            g.object_view(attacker).damage,
            if teamwork { 4 } else { 2 },
            "Strike has exact printed damage in both branches"
        );
    }
}

#[test]
fn msh_b_keyword_combat_prowess_and_basic_landcycling() {
    let p = Player::P0;
    let mut g = fresh(p);
    let commando = put(&mut g, p, "Kree Commandos", Zone::Battlefield);
    let blocker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
    mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
    cast(&mut g, spell, Some(Target::Card(commando)));
    settle(&mut g);
    assert_eq!(stats(&g, commando), (6, 5));
    attacks(&mut g, &[commando]);
    assert!(!g.state.objects[commando as usize].tapped);
    for _ in 0..4 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    reject(
        &mut g,
        Action::Block {
            blocker,
            attacker: commando,
        },
    );
    act(&mut g, Action::FinishBlockers);
    go(&mut g, p, Step::Main2);
    assert_eq!(g.life(p.other()), 14);
    let mut g = fresh(p);
    let sentinel = put(&mut g, p, "Kree Sentinel", Zone::Hand);
    let chosen = put(&mut g, p, "Island", Zone::Library);
    mana(&mut g, p, [0, 0, 0, 0, 0, 2]);
    activate(&mut g, sentinel, 0, &[]);
    decision(&mut g);
    act(&mut g, Action::ChooseSearch { card: chosen });
    act(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.state.objects[sentinel as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[chosen as usize].zone, Zone::Hand);
    assert_eq!(g.hand_count(p), 1);
    // Actual blocking restrictions, rather than checking only keyword bits.
    for (name, power) in [
        ("Falcon, Winged Wonder", 3),
        ("Giant-Sized Flying Ant", 3),
        ("Fin Fang Foom", 3),
        ("H.E.R.B.I.E. Scout Unit", 2),
        ("Human Torch, Johnny Storm", 2),
        ("Iron Lad, Diverging Destiny", 2),
        ("Iron Man, Master of Machines", 1),
        ("Ironheart, Clever Champion", 3),
        ("Kang the Conqueror", 4),
        ("Justice, Vance Astrovik", 2),
        ("M.O.D.O.K.", 2),
    ] {
        let mut g = fresh(p);
        let flyer = put(&mut g, p, name, Zone::Battlefield);
        let ground = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        attacks(&mut g, &[flyer]);
        if name == "Iron Lad, Diverging Destiny" {
            assert!(!g.state.objects[flyer as usize].tapped);
        }
        settle(&mut g);
        for _ in 0..4 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: flyer,
            },
        );
        act(&mut g, Action::FinishBlockers);
        go(&mut g, p, Step::Main2);
        assert_eq!(g.life(p.other()), 20 - power, "{name}");
    }
    for name in [
        "Guerrilla Gorilla",
        "Kree Sentinel",
        "Hulk, Gamma Goliath",
        "Hawkeye, Young Avenger",
        "Hawkeye, Master Marksman",
    ] {
        let mut g = fresh(p);
        let reach = put(&mut g, p.other(), name, Zone::Battlefield);
        let flyer = put(&mut g, p, "Falcon, Winged Wonder", Zone::Battlefield);
        attacks(&mut g, &[flyer]);
        for _ in 0..4 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::Block {
                blocker: reach,
                attacker: flyer,
            },
        );
        act(&mut g, Action::FinishBlockers);
        go(&mut g, p, Step::Main2);
        assert_eq!(g.life(p.other()), 20);
        if name == "Hawkeye, Young Avenger" {
            assert_eq!(g.state.objects[flyer as usize].zone, Zone::Battlefield);
            assert_eq!(
                g.state.objects[flyer as usize].damage, 2,
                "combat damage is not increased"
            );
        }
    }
    for name in [
        "Invisible Woman, Sue Storm",
        "M.O.D.O.K.",
        "Kingpin's Enforcers",
    ] {
        let mut g = fresh(p);
        let source = put(&mut g, p, name, Zone::Battlefield);
        attacks(&mut g, &[source]);
        go(&mut g, p, Step::Main2);
        assert_eq!((g.life(p), g.life(p.other())), (22, 18));
    }
    for name in ["Hulk, Gamma Goliath", "Knight of Wundagore"] {
        let mut g = fresh(p);
        let source = put(&mut g, p, name, Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let power = if name == "Knight of Wundagore" {
            let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
            mana(&mut g, p, [0, 0, 0, 0, 1, 0]);
            cast(&mut g, growth, Some(Target::Card(source)));
            settle(&mut g);
            5
        } else {
            6
        };
        attacks(&mut g, &[source]);
        for _ in 0..4 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::Block {
                blocker,
                attacker: source,
            },
        );
        act(&mut g, Action::FinishBlockers);
        for _ in 0..4 {
            if g.state.pending == Pending::Damage {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::AssignDamage {
                target: Target::Card(blocker),
                amount: 2,
            },
        );
        go(&mut g, p, Step::Main2);
        assert_eq!(g.life(p.other()), 22 - power);
    }
    let mut g = fresh(p);
    let helper = put(&mut g, p, "Hydraulic Helper", Zone::Battlefield);
    go(&mut g, p, Step::BeginCombat);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    reject(&mut g, Action::Attack { card: helper });
    let mut g = fresh(p);
    let marksman = put(&mut g, p, "Hawkeye, Master Marksman", Zone::Battlefield);
    let bear = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    attacks(&mut g, &[marksman]);
    decision(&mut g);
    act(&mut g, Action::ChooseX { value: 0 });
    settle(&mut g);
    for _ in 0..4 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(
        &mut g,
        Action::Block {
            blocker: bear,
            attacker: marksman,
        },
    );
    act(&mut g, Action::FinishBlockers);
    go(&mut g, p, Step::Main2);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[marksman as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[marksman as usize].damage, 0);
}

#[test]
fn msh_b_literal_paid_creature_costs_and_base_bodies() {
    // Header facts supplement the separate complete printed-ability scenarios.
    // Supply exactly the independently transcribed printed mana and check spent
    // mana plus the unmodified body before any optional entry decisions resolve.
    for (name, mana, power, toughness) in [
        ("Elektra, Daughter of the Hand", [0, 0, 2, 0, 0, 2], 3, 3),
        ("Falcon, Winged Wonder", [0, 1, 0, 0, 0, 4], 3, 4),
        ("Fin Fang Foom", [0, 0, 0, 2, 0, 2], 3, 5),
        ("Ghost, Spectral Saboteur", [0, 1, 0, 0, 0, 2], 2, 2),
        ("Giant-Sized Flying Ant", [0, 1, 0, 0, 0, 3], 3, 2),
        ("Grim Reaper, Lethal Legionnaire", [0, 0, 1, 0, 0, 3], 3, 4),
        ("Guerrilla Gorilla", [0, 0, 0, 0, 1, 1], 2, 2),
        ("H.E.R.B.I.E. Scout Unit", [0, 0, 0, 0, 0, 4], 2, 1),
        ("HYDRA Assault Robot", [0, 0, 0, 1, 0, 1], 1, 3),
        ("HYDRA Troopers", [0, 0, 1, 0, 0, 2], 3, 2),
        ("Hawkeye, Master Marksman", [0, 0, 0, 1, 0, 1], 2, 2),
        ("Hawkeye, Young Avenger", [0, 0, 0, 1, 0, 3], 2, 4),
        ("Hellcat, Undying Vigilante", [0, 0, 0, 0, 2, 0], 2, 2),
        ("Hercules, Prince of Power", [0, 0, 0, 0, 1, 2], 3, 3),
        ("Hero in Training", [1, 0, 0, 0, 0, 2], 2, 2),
        ("Hulk, Gamma Goliath", [0, 0, 0, 1, 1, 3], 6, 5),
        ("Hulkling, Burgeoning Bruiser", [0, 0, 0, 0, 1, 2], 2, 3),
        ("Human Torch, Johnny Storm", [0, 0, 0, 1, 0, 2], 2, 2),
        ("Hydraulic Helper", [0, 1, 0, 0, 0, 1], 2, 3),
        ("Invisible Woman, Sue Storm", [1, 0, 0, 0, 0, 4], 2, 5),
        ("Iron Fist, Living Weapon", [0, 0, 0, 1, 0, 2], 2, 3),
        ("Iron Lad, Diverging Destiny", [0, 1, 0, 0, 0, 2], 2, 2),
        ("Iron Man, Master of Machines", [0, 1, 0, 1, 0, 2], 1, 4),
        ("Ironheart, Clever Champion", [0, 1, 0, 0, 0, 4], 3, 4),
        ("Jennifer Walters", [1, 0, 0, 0, 0, 1], 2, 3),
        ("Jessica Jones, Private Eye", [0, 0, 0, 1, 0, 2], 2, 3),
        ("Justice, Vance Astrovik", [0, 1, 0, 0, 0, 2], 2, 2),
        ("K'un-Lun Warrior", [0, 0, 0, 1, 0, 1], 2, 2),
        ("Ka-Zar of the Savage Land", [0, 0, 0, 0, 1, 4], 3, 2),
        ("Kang the Conqueror", [0, 2, 0, 0, 0, 2], 4, 5),
        ("Kang, Temporal Tyrant", [0, 1, 1, 0, 0, 2], 3, 4),
        ("Kid Loki", [0, 1, 0, 0, 0, 0], 1, 1),
        ("Killmonger, Scourge of Wakanda", [0, 0, 1, 0, 1, 2], 3, 3),
        ("King T'Challa", [1, 1, 0, 0, 0, 1], 3, 2),
        ("Kingpin's Enforcers", [0, 0, 1, 0, 0, 2], 2, 3),
        ("Klaw, Sonic Subjugator", [0, 0, 1, 0, 0, 2], 2, 2),
        ("Knight of Wundagore", [0, 0, 0, 0, 1, 1], 2, 1),
        ("Kree Commandos", [1, 0, 0, 0, 0, 2], 2, 1),
        ("Kree Sentinel", [0, 0, 0, 1, 0, 4], 5, 5),
        ("Leader, Super-Genius", [0, 2, 0, 0, 0, 2], 1, 3),
        ("Loki Laufeyson", [0, 0, 0, 1, 0, 1], 2, 1),
        ("Loki, God of Mischief", [0, 1, 0, 0, 0, 1], 2, 1),
        ("Luke Cage, Power Man", [1, 0, 0, 0, 0, 3], 2, 5),
        ("M.O.D.O.K.", [0, 0, 2, 0, 0, 3], 2, 2),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(&mut g, p, name, Zone::Hand);
        g.state.players[p.index()].mana = mana;
        cast(&mut g, id, None);
        assert_eq!(
            g.state.players[p.index()].mana,
            [0; 6],
            "{name}: literal paid cast cost"
        );
        super::msh_a::until(&mut g, |g| g.object_view(id).zone == Zone::Battlefield);
        assert_eq!(
            stats(&g, id),
            (power, toughness),
            "{name}: independent printed base body"
        );
        assert!(g.is_creature(id), "{name}");
    }
}
