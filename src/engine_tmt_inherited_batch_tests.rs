//! Paid-action audit of inherited TMT admissions. Every checked action compares
//! raw/index owned branches, both observations, rejected actors and restore.
use super::ability_timeline_tests::{checked, named, next_turn, settle, settle_on, setup};
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
fn activate(g: &mut Game, id: CardId, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
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
#[test]
fn tmt_inherited_bishop_zero_two_targets_partial_invalid_and_turn_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bishop = paid(&mut g, p, "Agent Bishop, Man in Black", &[]);
        settle(&mut g);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
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
        // Exact target incarnation departs after selection; surviving target works.
        g.move_object(own, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(bishop, CounterKind::PlusOne), 0);
        next_turn(&mut g);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        assert!(g.state.stack.is_empty());
        next_turn(&mut g);
        own_main(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        checked(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
    }
}
#[test]
fn tmt_inherited_pizza_destroy_and_paid_food_sacrifice_with_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let pizza = paid(&mut g, p, "Anchovy & Banana Pizza", &[]);
        settle_on(&mut g, Some(Target::Card(victim)));
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
        let life = g.state.players[p.index()].life;
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        activate(&mut g, pizza, &[]);
        assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
    }
}
#[test]
fn tmt_inherited_pizza_illegalated_target_does_not_destroy_new_incarnation() {
    let p = Player::P0;
    let mut g = setup(p);
    let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    paid(&mut g, p, "Anchovy & Banana Pizza", &[]);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    until(&mut g, |g| {
        g.state.pending == Pending::Priority && g.state.announcing.is_none()
    });
    g.move_object(victim, Zone::Hand);
    g.move_object(victim, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.state.objects[victim as usize].zone, Zone::Battlefield);
}
#[test]
fn tmt_inherited_april_kunoichi_private_scry_order_and_actual_power_block_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bottom = named(&mut g, p, "Shock", Zone::Library);
        let top = named(&mut g, p, "Island", Zone::Library);
        let april = paid(&mut g, p, "April O'Neil, Kunoichi Trainee", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        assert!(g.observe(p).library_choice.is_some());
        assert!(g.observe(p.other()).library_choice.is_none());
        assert!(g.observe(p.other()).actions.is_empty());
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: bottom,
                destination: LibraryDestination::Bottom,
            },
        );
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library[0], bottom);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&top));
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        let small = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let large = named(&mut g, p.other(), "Grasping Longneck", Zone::Battlefield);
        combat(&mut g, april);
        reject(
            &mut g,
            Action::Block {
                blocker: large,
                attacker: april,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: small,
                attacker: april,
            },
        );
    }
}
#[test]
fn tmt_inherited_april_reporter_draws_actual_combat_damage_then_private_discard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let april = paid(&mut g, p, "April, Reporter of the Weird", &[]);
        settle(&mut g);
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        let before = g.state.players[p.index()].library.len();
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == p && o.zone == Zone::Hand)
            .count();
        combat(&mut g, april);
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::Discard { .. }))
        });
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert!(g.observe(p.other()).actions.is_empty());
        let discard = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        checked(&mut g, discard);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count(),
            hand + 1
        );
    }
}
#[test]
fn tmt_inherited_bespoke_bounce_other_nonland_equip_three_own_creature_vigilance() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let own = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let bo = paid(&mut g, p, "Bespoke Bō", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [bo, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Hand);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: bo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!((g.stats(own).power, g.stats(own).toughness), (4, 3));
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        combat(&mut g, own);
        assert!(!g.state.objects[own as usize].tapped);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: bo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.move_object(bo, Zone::Hand);
        assert_eq!((g.stats(own).power, g.stats(own).toughness), (2, 2));
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
    }
}
#[test]
fn tmt_inherited_bot_bashing_lethal_exiles_and_no_buzz_death_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        // Cast Buzz Bots legally under the active player's control, then kill it.
        let bot = paid(&mut g, p, "Buzz Bots", &[]);
        settle(&mut g);
        let before = g.state.players[p.index()].library.len();
        paid(&mut g, p, "Bot Bashing Time", &[Target::Card(bot)]);
        settle(&mut g);
        assert_eq!(g.state.objects[bot as usize].zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}
#[test]
fn tmt_inherited_bot_bashing_survivor_later_death_and_expiration() {
    let p = Player::P0;
    for expire in [false, true] {
        let mut g = setup(p);
        let victim = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.set_counters(victim, Counters::of(CounterKind::PlusOne, 8));
        paid(&mut g, p, "Bot Bashing Time", &[Target::Card(victim)]);
        settle(&mut g);
        assert_eq!(g.state.objects[victim as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.objects[victim as usize].damage, 6);
        if expire {
            next_turn(&mut g);
            next_turn(&mut g);
            own_main(&mut g, p);
        }
        paid(&mut g, p, "Murder", &[Target::Card(victim)]);
        settle(&mut g);
        assert_eq!(
            g.state.objects[victim as usize].zone,
            if expire { Zone::Graveyard } else { Zone::Exile }
        );
    }
}
#[test]
fn tmt_inherited_buzz_actual_flying_vigilance_and_death_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bot = paid(&mut g, p, "Buzz Bots", &[]);
        settle(&mut g);
        assert!(
            g.keywords(bot)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        let ground = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        combat(&mut g, bot);
        assert!(!g.state.objects[bot as usize].tapped);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: bot,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::Main2 && g.state.pending == Pending::Priority
        });
        for player in &mut g.state.players {
            player.mana = [30; 6];
        }
        let before = g.state.players[p.index()].library.len();
        paid(&mut g, p, "Murder", &[Target::Card(bot)]);
        settle(&mut g);
        assert_eq!(g.state.objects[bot as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
fn library_pick(g: &mut Game, p: Player, chosen: Option<CardId>) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
    assert!(g.observe(p.other()).actions.is_empty());
    assert!(g.observe(p.other()).library_selection.is_none());
    if let Some(card) = chosen {
        checked(g, Action::SelectLibrary { card });
    }
    checked(g, Action::FinishLibrarySelection);
    settle(g);
    if let Some(card) = chosen {
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(card))
        );
    }
}
#[test]
fn tmt_inherited_casey_artifact_only_private_top_four_pick_random_bottom_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fifth = named(&mut g, p, "Anchovy & Banana Pizza", Zone::Library);
        let land = named(&mut g, p, "Mountain", Zone::Library);
        let artifact = named(&mut g, p, "Buzz Bots", Zone::Library);
        named(&mut g, p, "Shock", Zone::Library);
        named(&mut g, p, "Grizzly Bears", Zone::Library);
        let casey = paid(&mut g, p, "Casey Jones, Jury-Rig Justiciar", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        reject(&mut g, Action::SelectLibrary { card: fifth });
        reject(&mut g, Action::SelectLibrary { card: land });
        library_pick(&mut g, p, Some(artifact));
        assert_eq!(g.state.objects[artifact as usize].zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&fifth));
        combat(&mut g, casey);
        assert!(g.state.objects[casey as usize].tapped);
    }
}
#[test]
fn tmt_inherited_cowabunga_all_four_filter_branches_and_optional_decline() {
    for name in [
        "Crustacean Commando",
        "April O'Neil, Kunoichi Trainee",
        "Aegis Turtle",
        "Forest",
    ] {
        let p = Player::P0;
        let mut g = setup(p);
        let bad = named(&mut g, p, "Shock", Zone::Library);
        let chosen = named(&mut g, p, name, Zone::Library);
        paid(&mut g, p, "Cowabunga!", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
        });
        reject(&mut g, Action::SelectLibrary { card: bad });
        library_pick(&mut g, p, Some(chosen));
        assert_eq!(g.state.objects[chosen as usize].zone, Zone::Hand);
        assert!(g.state.players[p.index()].library[..3].contains(&bad));
    }
    for name in ["Cowabunga!", "Casey Jones, Jury-Rig Justiciar"] {
        let p = Player::P1;
        let mut g = setup(p);
        let eligible = named(
            &mut g,
            p,
            if name == "Cowabunga!" {
                "Forest"
            } else {
                "Buzz Bots"
            },
            Zone::Library,
        );
        paid(&mut g, p, name, &[]);
        library_pick(&mut g, p, None);
        assert_eq!(g.state.objects[eligible as usize].zone, Zone::Library);
        assert!(g.state.players[p.index()].library[..4].contains(&eligible));
    }
}
#[test]
fn tmt_inherited_crustacean_mutagen_real_sorcery_target_any_creature_and_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let crab = paid(&mut g, p, "Crustacean Commando", &[]);
        settle(&mut g);
        assert_eq!((g.stats(crab).power, g.stats(crab).toughness), (0, 3));
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
        let enemy = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        activate(&mut g, token, &[Target::Card(enemy)]);
        settle(&mut g);
        assert_ne!(g.state.objects[token as usize].zone, Zone::Battlefield);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 1);
    }
}
#[test]
fn tmt_inherited_death_family_cost_boundary_and_exile_no_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let three = named(
            &mut g,
            p.other(),
            "Donatello, Way with Machines",
            Zone::Battlefield,
        );
        let four = named(
            &mut g,
            p.other(),
            "Donatello, Turtle Techie",
            Zone::Battlefield,
        );
        let spell = named(&mut g, p, "Death in the Family", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(four),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(three),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[three as usize].zone, Zone::Exile);
        let bot = paid(&mut g, p, "Buzz Bots", &[]);
        settle(&mut g);
        let before = g.state.players[p.index()].library.len();
        paid(&mut g, p, "Death in the Family", &[Target::Card(bot)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}
#[test]
fn tmt_inherited_techie_intervening_if_checks_entry_and_resolution() {
    for presence in [0, 1, 2] {
        let p = Player::P0;
        let mut g = setup(p);
        let artifact = if presence > 0 {
            Some(named(
                &mut g,
                p,
                "@token/283fd83a9aec83a51b736926",
                Zone::Battlefield,
            ))
        } else {
            None
        };
        let before = g.state.players[p.index()].library.len();
        paid(&mut g, p, "Donatello, Turtle Techie", &[]);
        until(&mut g, |g| {
            g.state
                .objects
                .iter()
                .any(|o| o.card.name() == "Donatello, Turtle Techie" && o.zone == Zone::Battlefield)
        });
        if presence == 2 {
            g.move_object(artifact.unwrap(), Zone::Graveyard);
        }
        if presence == 0 {
            named(
                &mut g,
                p,
                "@token/283fd83a9aec83a51b736926",
                Zone::Battlefield,
            );
        }
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].library.len(),
            before - usize::from(presence == 1)
        );
    }
}
#[test]
fn tmt_inherited_way_with_machines_tokens_creature_artifact_and_opponent_exclusion() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let don = paid(&mut g, p, "Donatello, Way with Machines", &[]);
        settle(&mut g);
        assert!(g.keywords(don).contains(Keywords::FLYING));
        paid(&mut g, p, "Buzz Bots", &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(don, CounterKind::PlusOne), 1);
        paid(&mut g, p, "Crustacean Commando", &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(don, CounterKind::PlusOne), 2);
        next_turn(&mut g);
        own_main(&mut g, p.other());
        paid(&mut g, p.other(), "Buzz Bots", &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(don, CounterKind::PlusOne), 2);
    }
}
#[test]
fn tmt_inherited_dream_beavers_life_and_private_scry_one() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let top = named(&mut g, p, "Shock", Zone::Library);
        let own = g.state.players[p.index()].life;
        let other = g.state.players[p.other().index()].life;
        let beaver = paid(&mut g, p, "Dream Beavers", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectLibrary { .. })
        });
        assert_eq!(g.state.players[p.index()].life, own + 1);
        assert_eq!(g.state.players[p.other().index()].life, other - 1);
        assert!(g.keywords(beaver).contains(Keywords::FLYING));
        assert!(g.observe(p.other()).library_choice.is_none());
        checked(
            &mut g,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library[0], top);
    }
}
#[test]
fn tmt_inherited_epf_all_hybrid_costs_alliance_not_self_or_enemy() {
    for mana in [
        [0, 0, 0, 3, 0, 0],
        [1, 0, 0, 2, 0, 0],
        [2, 0, 0, 1, 0, 0],
        [3, 0, 0, 0, 0, 0],
    ] {
        let p = Player::P0;
        let mut g = setup(p);
        g.state.players[p.index()].mana = mana;
        let squad = paid(&mut g, p, "EPF Point Squad", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.counter_count(squad, CounterKind::PlusOne), 0);
        g.state.players[p.index()].mana = [30; 6];
        paid(&mut g, p, "Crustacean Commando", &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(squad, CounterKind::PlusOne), 1);
        // Mutagen is a noncreature and does not trigger alliance.
        next_turn(&mut g);
        own_main(&mut g, p.other());
        paid(&mut g, p.other(), "Grizzly Bears", &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(squad, CounterKind::PlusOne), 1);
    }
}
