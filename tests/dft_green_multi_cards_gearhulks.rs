//! Gearhulk entry effects, successful-entry receipts and max-speed graveyard interactions.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn drain(g: &mut Game) {
    for _ in 0..96 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("bounded gearhulk resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..96 {
        if g.legal_actions().iter().any(&wanted) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("bounded gearhulk choice");
}
fn end(g: &mut Game, active: Player) {
    g.state.active = active;
    g.state.priority = active;
    g.begin_step(Step::End);
    g.prepare_triggers();
    drain(g);
}
fn start(g: &mut Game, card: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn target(g: &mut Game, card: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}
#[test]
fn brightglass_finds_at_most_two_small_artifact_creature_or_enchantment_cards_and_can_decline() {
    for mode in 0..3 {
        let mut g = scenario(Player::P0);
        let artifact = put(&mut g, Player::P0, "Mox Jasper", Zone::Library);
        let creature = put(&mut g, Player::P0, "Llanowar Elves", Zone::Library);
        let enchantment = put(&mut g, Player::P0, "Point the Way", Zone::Library);
        let bad = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
        let instant = put(&mut g, Player::P0, "Defend the Rider", Zone::Library);
        let library = g.state.players[0].library.clone();
        let source = put(&mut g, Player::P0, "Brightglass Gearhulk", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseOptional { .. }));
        act(&mut g, Action::ChooseOptional { accept: mode < 2 });
        if mode < 2 {
            reach(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
            for id in [bad, instant] {
                reject(&mut g, Action::ChooseSearch { card: id });
            }
            let first = if mode == 0 { artifact } else { enchantment };
            act(&mut g, Action::ChooseSearch { card: first });
            act(&mut g, Action::ChooseSearch { card: creature });
            reject(
                &mut g,
                Action::ChooseSearch {
                    card: if mode == 0 { enchantment } else { artifact },
                },
            );
            act(&mut g, Action::FinishSearch);
            drain(&mut g);
            for id in [first, creature] {
                assert_eq!(g.object_view(id).zone, Zone::Hand);
                assert!(g.observe(Player::P1).known_hand.contains(&g.object_ref(id)));
            }
        } else {
            drain(&mut g);
            assert_eq!(g.state.players[0].library, library);
        }
        assert_eq!(stats(&g, source), (4, 4));
        assert!(
            g.keywords(source)
                .contains(Keywords::FIRST_STRIKE.union(Keywords::TRAMPLE))
        );
    }
}
#[test]
fn coalstoke_returns_opponents_creature_with_finality_and_lasting_keywords_until_our_next_end_step()
{
    for sacrifice in [false, true] {
        let mut g = scenario(Player::P0);
        let returned = put(&mut g, Player::P1, "Hill Giant", Zone::Graveyard);
        let large = put(&mut g, Player::P1, "Serra Angel", Zone::Graveyard);
        let source = put(&mut g, Player::P0, "Coalstoke Gearhulk", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(large),
            },
        );
        target(&mut g, returned);
        drain(&mut g);
        assert_eq!(g.object_view(returned).controller, Player::P0);
        assert_eq!(g.counter_count(returned, CounterKind::Finality), 1);
        assert!(
            g.keywords(returned).contains(
                Keywords::MENACE
                    .union(Keywords::DEATHTOUCH)
                    .union(Keywords::HASTE)
            )
        );
        assert!(
            g.keywords(source)
                .contains(Keywords::MENACE.union(Keywords::DEATHTOUCH))
        );
        if sacrifice {
            g.sacrifice_batch(&[(Player::P0, returned)]);
            assert_eq!(g.object_view(returned).zone, Zone::Exile);
        } else {
            end(&mut g, Player::P1);
            assert_eq!(g.object_view(returned).zone, Zone::Battlefield);
            g.finish_cleanup();
            assert!(
                g.keywords(returned).contains(
                    Keywords::MENACE
                        .union(Keywords::DEATHTOUCH)
                        .union(Keywords::HASTE)
                )
            );
            end(&mut g, Player::P0);
            assert_eq!(g.object_view(returned).zone, Zone::Exile);
        }
    }
}
#[test]
fn oildeep_private_hand_look_can_decline_or_replace_one_chosen_card() {
    for accept in [false, true] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P1, "Forest", Zone::Hand);
        let source = put(&mut g, Player::P0, "Oildeep Gearhulk", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(Player::P1),
            },
        );
        reach(&mut g, |a| matches!(a, Action::ChooseHandCard { .. }));
        assert!(g.observe(Player::P1).hand_choice.is_none());
        let object = accept.then(|| g.object_ref(card));
        act(&mut g, Action::ChooseHandCard { object });
        drain(&mut g);
        assert_eq!(
            g.object_view(card).zone,
            if accept { Zone::Graveyard } else { Zone::Hand }
        );
        assert_eq!(g.state.players[1].cards_drawn_this_turn, u64::from(accept));
        assert_eq!(stats(&g, source), (4, 4));
        assert!(g.keywords(source).contains(Keywords::LIFELINK));
    }
}
#[test]
fn oildeep_ward_cost_is_one_mana_and_decline_counters_the_related_targeting_spell() {
    for pay_ward in [false, true] {
        let mut g = scenario(Player::P1);
        let source = put(&mut g, Player::P0, "Oildeep Gearhulk", Zone::Battlefield);
        let spell = put(&mut g, Player::P1, "Lightning Strike", Zone::Hand);
        g.state.players[1].mana = [0, 0, 0, 1, 0, if pay_ward { 2 } else { 1 }];
        cast(&mut g, spell, Some(Target::Card(source)));
        reach(&mut g, |a| matches!(a, Action::DeclineResolutionMana));
        if pay_ward {
            choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
        } else {
            act(&mut g, Action::DeclineResolutionMana);
        }
        drain(&mut g);
        assert_eq!(g.object_view(source).damage, if pay_ward { 3 } else { 0 });
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}
#[test]
fn pyrewood_buffs_only_other_controlled_creatures_and_disables_prevention_through_cleanup() {
    let mut g = scenario(Player::P0);
    let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let source = put(&mut g, Player::P0, "Pyrewood Gearhulk", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(stats(&g, source), (7, 7));
    assert_eq!(stats(&g, own), (4, 4));
    assert_eq!(stats(&g, enemy), (2, 2));
    assert!(
        g.keywords(source)
            .contains(Keywords::VIGILANCE.union(Keywords::MENACE))
    );
    assert!(
        g.keywords(own)
            .contains(Keywords::VIGILANCE.union(Keywords::MENACE))
    );
    assert!(
        !g.keywords(enemy)
            .intersects(Keywords::VIGILANCE.union(Keywords::MENACE))
    );
    assert!(g.state.damage_prevention_prohibited);
    g.finish_cleanup();
    assert_eq!(stats(&g, own), (2, 2));
    assert!(!g.state.damage_prevention_prohibited);
    assert!(
        !g.keywords(own)
            .intersects(Keywords::VIGILANCE.union(Keywords::MENACE))
    );
}
#[test]
fn riptide_inserts_opposing_nonland_in_its_owners_library_third_and_prowess_is_temporary() {
    let mut g = scenario(Player::P0);
    let stolen = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.state.objects[stolen as usize].controller = Player::P1;
    let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut g, Player::P1, "Forest", Zone::Battlefield);
    let enemy_library = g.state.players[1].library.clone();
    let source = put(&mut g, Player::P0, "Riptide Gearhulk", Zone::Hand);
    cast(&mut g, source, None);
    reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
    for id in [own, land] {
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    target(&mut g, stolen);
    drain(&mut g);
    let library = &g.state.players[0].library;
    assert_eq!(library[library.len() - 3], stolen);
    assert_eq!(g.state.players[1].library, enemy_library);
    let spell = put(&mut g, Player::P0, "Lightning Strike", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(Player::P1)));
    drain(&mut g);
    assert_eq!(stats(&g, source), (3, 6));
    assert!(g.keywords(source).contains(Keywords::DOUBLE_STRIKE));
    g.finish_cleanup();
    assert_eq!(stats(&g, source), (2, 5));
}
#[test]
fn oviya_puts_only_creature_or_vehicle_from_private_hand_and_adds_counters_only_if_artifact() {
    for name in [
        Some("Grizzly Bears"),
        Some("Gleaming Barrier"),
        Some("Cultivator's Caravan"),
        None,
    ] {
        let mut g = scenario(Player::P0);
        let source = put(
            &mut g,
            Player::P0,
            "Oviya, Automech Artisan",
            Zone::Battlefield,
        );
        let entrant = put(
            &mut g,
            Player::P0,
            name.unwrap_or("Grizzly Bears"),
            Zone::Hand,
        );
        let bad = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Hand);
        start(&mut g, source);
        pay(&mut g);
        reach(&mut g, |a| matches!(a, Action::DeclineHandEntry));
        assert!(
            g.observe(Player::P1)
                .objects
                .iter()
                .all(|o| o.id != entrant)
        );
        let object = g.object_ref(bad);
        reject(&mut g, Action::PutHandPermanent { object });
        if name.is_some() {
            let object = g.object_ref(entrant);
            act(&mut g, Action::PutHandPermanent { object });
        } else {
            act(&mut g, Action::DeclineHandEntry);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(entrant).zone,
            if name.is_some() {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        let expected = if matches!(name, Some("Gleaming Barrier" | "Cultivator's Caravan")) {
            2
        } else {
            0
        };
        assert_eq!(g.counter_count(entrant, CounterKind::PlusOne), expected);
        assert!(g.object_view(source).tapped);
    }
}
#[test]
fn oviya_grants_trample_only_to_creatures_attacking_its_opponent() {
    for active in [Player::P0, Player::P1] {
        let mut g = scenario(active);
        put(
            &mut g,
            Player::P0,
            "Oviya, Automech Artisan",
            Zone::Battlefield,
        );
        let attacker = put(&mut g, active, "Grizzly Bears", Zone::Battlefield);
        assert!(!g.keywords(attacker).contains(Keywords::TRAMPLE));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        drain(&mut g);
        assert_eq!(
            g.keywords(attacker).contains(Keywords::TRAMPLE),
            active == Player::P0
        );
    }
}
#[test]
fn explosive_getaway_blinks_before_dealing_four_and_returns_under_owner_control_next_end_step() {
    for blink in [false, true] {
        let mut g = scenario(Player::P0);
        let saved = put(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[saved as usize].controller = Player::P0;
        let own = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Explosive Getaway", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        if blink {
            target(&mut g, saved);
        }
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        drain(&mut g);
        for id in [own, enemy] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(
            g.object_view(saved).zone,
            if blink { Zone::Exile } else { Zone::Graveyard }
        );
        if blink {
            end(&mut g, Player::P1);
            assert_eq!(g.object_view(saved).zone, Zone::Battlefield);
            assert_eq!(g.object_view(saved).controller, Player::P1);
        }
    }
}
#[test]
fn caradora_searches_mount_or_vehicle_and_adds_one_per_counter_event_on_creature_or_vehicle() {
    for name in ["Alacrian Jaguar", "Cultivator's Caravan"] {
        let mut g = scenario(Player::P0);
        let found = put(&mut g, Player::P0, name, Zone::Library);
        let bad = put(&mut g, Player::P0, "Grizzly Bears", Zone::Library);
        let source = put(&mut g, Player::P0, "Caradora, Heart of Alacria", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseOptional { .. }));
        act(&mut g, Action::ChooseOptional { accept: true });
        reach(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
        reject(&mut g, Action::ChooseSearch { card: bad });
        act(&mut g, Action::ChooseSearch { card: found });
        act(&mut g, Action::FinishSearch);
        drain(&mut g);
        assert_eq!(g.object_view(found).zone, Zone::Hand);
        let mouse = put(
            &mut g,
            Player::P0,
            "Stampeding Scurryfoot",
            Zone::Battlefield,
        );
        start(&mut g, mouse);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.counter_count(mouse, CounterKind::PlusOne), 2);
        let vehicle = put(
            &mut g,
            Player::P0,
            "Cultivator's Caravan",
            Zone::Battlefield,
        );
        let rex = put(&mut g, Player::P0, "Agonasaur Rex", Zone::Hand);
        start(&mut g, rex);
        pay(&mut g);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        target(&mut g, vehicle);
        drain(&mut g);
        assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), 3);
    }
}
#[test]
fn embalmed_ascendant_makes_a_zombie_and_its_max_speed_death_trigger_includes_itself_and_tokens() {
    for speed in [3, 4] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Embalmed Ascendant", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(g.state.players[0].speed, 1);
        assert_eq!(stats(&g, source), (1, 2));
        let zombie = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && g.has_subtype(id as CardId, "Zombie"))
                .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(stats(&g, zombie), (2, 2));
        assert_eq!(g.object_view(zombie).colors, 4);
        assert!(!g.object_view(zombie).tapped);
        g.state.players[0].speed = speed;
        g.sacrifice_batch(&[(Player::P0, source), (Player::P0, zombie)]);
        drain(&mut g);
        assert_eq!(g.state.players[0].life, if speed == 4 { 22 } else { 20 });
        assert_eq!(g.state.players[1].life, if speed == 4 { 18 } else { 20 });
    }
}
#[test]
fn zahur_sacrifices_another_once_per_turn_surveils_and_only_nontoken_deaths_make_tapped_zombies() {
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Zahur, Glory's Past", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.state.players[0].speed, 1);
    g.state.players[0].speed = 4;
    let bear = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    start(&mut g, source);
    let object = g.object_ref(source);
    reject(&mut g, Action::ChooseAdditionalCost { object });
    let object = g.object_ref(bear);
    act(&mut g, Action::ChooseAdditionalCost { object });
    if g.legal_actions()
        .contains(&Action::FinishAdditionalCostGroup)
    {
        act(&mut g, Action::FinishAdditionalCostGroup);
    }
    pay(&mut g);
    reach(&mut g, |a| matches!(a, Action::ArrangeLibrary { .. }));
    choose(&mut g, |a| {
        matches!(
            a,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Top,
                ..
            }
        )
    });
    drain(&mut g);
    let zombie = g
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(id as CardId, "Zombie"))
                .then_some(id as CardId)
        })
        .unwrap();
    assert_eq!(stats(&g, zombie), (2, 2));
    assert!(g.object_view(zombie).tapped);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    g.finish_cleanup();
    assert_eq!(g.actor(), Some(Player::P1));
    act(&mut g, Action::Pass);
    start(&mut g, source);
    let object = g.object_ref(zombie);
    act(&mut g, Action::ChooseAdditionalCost { object });
    if g.legal_actions()
        .contains(&Action::FinishAdditionalCostGroup)
    {
        act(&mut g, Action::FinishAdditionalCostGroup);
    }
    pay(&mut g);
    reach(&mut g, |a| matches!(a, Action::ArrangeLibrary { .. }));
    choose(&mut g, |a| {
        matches!(
            a,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Top,
                ..
            }
        )
    });
    drain(&mut g);
    assert!(
        !g.state
            .objects
            .iter()
            .enumerate()
            .any(|(id, o)| o.zone == Zone::Battlefield
                && o.is_token()
                && g.has_subtype(id as CardId, "Zombie"))
    );
}
