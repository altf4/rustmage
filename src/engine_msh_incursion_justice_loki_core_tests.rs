//! Gated immutable consumers plus real paid interactions; no new card admissions.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const INCURSION: &str = "Multiversal Incursion";
fn priority(game: &mut Game, player: Player) {
    if game.actor() != Some(player) {
        checked(game, Action::Pass);
    }
    assert_eq!(game.actor(), Some(player));
}
fn paid(game: &mut Game, player: Player, name: &str, targets: &[Target]) -> CardId {
    priority(game, player);
    let card = Card::from_name(name).unwrap();
    assert!(
        card.is_supported(),
        "paid fixtures use admitted cards: {name}"
    );
    let id = named(game, player, name, Zone::Hand);
    let (colored, generic) = card.cost();
    game.state.players[player.index()].mana = colored;
    game.state.players[player.index()].mana[5] += generic;
    cast(game, id, None, targets);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(game);
    id
}
fn token_ids(game: &Game, player: Player, name: &str) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            let id = id as CardId;
            (o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && game.object_view(id).name() == name)
                .then_some(id)
        })
        .collect()
}
fn gated_source(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    // Physical fixture only for these owned staged definitions. No definition
    // is overridden and the ordinary supported-card fixture guard stays intact.
    assert!(matches!(
        name,
        INCURSION | "Justice, Vance Astrovik" | "Loki, God of Mischief"
    ));
    let card = Card::from_name(name).unwrap();
    if card.is_supported() {
        return named(game, player, name, zone);
    }
    let id = CardId::try_from(game.state.objects.len()).unwrap();
    game.state.objects.push(Object {
        card,
        copy_kind: CopyKind::None,
        owner: player,
        controller: player,
        zone,
        generation: 0,
        tapped: false,
        summoning_sick: false,
        flashback: false,
        damage: 0,
        damaged_this_turn: false,
        dealt_damage_this_turn: false,
        deathtouch_since_sba: false,
        growth: Stats::default(),
        gained_keywords: Keywords::NONE,
        counters: InitialCounters::default(),
        used_abilities: 0,
        attached_to: None,
    });
    if zone == Zone::Battlefield {
        game.register_battlefield_sources(id);
    }
    id
}
fn bounded_incursion_cast(game: &mut Game, player: Player) {
    // The core checkpoint exercised this same real typed body with an explicit
    // gated physical fixture. Following admission it uses ordinary supported
    // fixture construction and the same checked exact paid cast.
    let card = Card::from_name(INCURSION).unwrap();
    let id = gated_source(game, player, INCURSION, Zone::Hand);
    priority(game, player);
    let (colored, generic) = card.cost();
    game.state.players[player.index()].mana = colored;
    game.state.players[player.index()].mana[5] += generic;
    cast(game, id, None, &[]);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(game);
    assert_eq!(game.state.objects[id as usize].zone, Zone::Graveyard);
}
#[test]
fn msh_incursion_core_frozen_batch_new_and_old_mister_fantastic_observe_once() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        let mister = paid(&mut game, player, "Mister Fantastic, Reed Richards", &[]);
        paid(&mut game, player, "Barkform Harvester", &[]);
        game.copy_controlled_nontoken_creatures_nonlegendary(player);
        game.advance_entries();
        let copies = token_ids(&game, player, "Mister Fantastic, Reed Richards");
        assert_eq!(copies.len(), 1);
        assert!(!game.object_view(copies[0]).is_legendary());
        assert_eq!(token_ids(&game, player, "Barkform Harvester").len(), 1);
        let triggered: Vec<_> = game
            .state
            .pending_triggers
            .iter()
            .filter(|item| {
                item.source_card == Card::from_name("Mister Fantastic, Reed Richards").unwrap()
            })
            .map(|item| item.object)
            .collect();
        assert_eq!(
            triggered.len(),
            2,
            "one whole entry event, observed by old and new source"
        );
        assert!(triggered.contains(&game.object_ref(mister)));
        assert!(triggered.contains(&game.object_ref(copies[0])));
        settle(&mut game);
    }
}
#[test]
fn msh_incursion_core_paid_control_and_physical_token_selection_with_owned_restore() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        paid(&mut game, player, "Mole Man, Moloid Master", &[]);
        let land = named(&mut game, player, "Forest", Zone::Hand);
        checked(&mut game, Action::PlayLand { card: land });
        settle(&mut game);
        assert_eq!(token_ids(&game, player, "Moloid").len(), 1);
        let stolen = named(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let excluded = named(&mut game, player.other(), "Aegis Turtle", Zone::Battlefield);
        paid(&mut game, player, "Confiscate", &[Target::Card(stolen)]);
        assert_eq!(game.state.objects[stolen as usize].controller, player);
        let secret = named(&mut game, player, "Cancel", Zone::Hand);
        let snapshot = game.snapshot();
        let mut orphan = {
            let parent = game.clone();
            parent.clone()
        };
        bounded_incursion_cast(&mut game, player);
        bounded_incursion_cast(&mut orphan, player);
        assert_eq!(game.snapshot(), orphan.snapshot());
        assert_eq!(token_ids(&game, player, "Bear Cub").len(), 1);
        assert_eq!(
            token_ids(&game, player, "Moloid").len(),
            1,
            "physical tokens are not copied"
        );
        assert!(token_ids(&game, player, "Aegis Turtle").is_empty());
        assert_eq!(
            game.state.objects[excluded as usize].controller,
            player.other()
        );
        assert_eq!(game.state.objects[stolen as usize].owner, player.other());
        let copy = token_ids(&game, player, "Bear Cub")[0];
        assert_eq!(game.state.objects[copy as usize].owner, player);
        assert!(
            !game
                .observe(player.other())
                .objects
                .iter()
                .any(|v| v.id == secret)
        );
        orphan.restore(&snapshot).unwrap();
        bounded_incursion_cast(&mut orphan, player);
        assert_eq!(game.object_view(copy), orphan.object_view(copy));
    }
}
#[test]
fn msh_incursion_core_mixed_recipe_replacements_apply_to_one_whole_event() {
    for player in [Player::P0, Player::P1] {
        for replacer in ["Worldwalker Helm", "Draconic Visitor", "Doubling Season"] {
            let mut game = setup(player);
            paid(&mut game, player, replacer, &[]);
            paid(&mut game, player, "Barkform Harvester", &[]);
            paid(&mut game, player, "Bear Cub", &[]);
            bounded_incursion_cast(&mut game, player);
            assert_eq!(
                token_ids(&game, player, "Map Token").len(),
                usize::from(replacer == "Worldwalker Helm")
            );
            assert_eq!(
                token_ids(&game, player, "Barkform Harvester").len(),
                if replacer == "Draconic Visitor" {
                    0
                } else if replacer == "Doubling Season" {
                    2
                } else {
                    1
                }
            );
            assert_eq!(
                token_ids(&game, player, "Dragon Token").len(),
                usize::from(replacer == "Draconic Visitor")
            );
            assert_eq!(
                token_ids(&game, player, "Bear Cub").len(),
                if replacer == "Doubling Season" { 2 } else { 1 }
            );
        }
    }
}
#[test]
fn msh_incursion_core_empty_selection_finishes_gated_immutable_program_without_event() {
    for player in [Player::P0, Player::P1] {
        let mut game = setup(player);
        paid(&mut game, player, "Worldwalker Helm", &[]);
        bounded_incursion_cast(&mut game, player);
        assert!(
            game.state
                .objects
                .iter()
                .all(|o| o.zone != Zone::Battlefield || !o.is_token())
        );
        assert!(game.state.entry_batch.is_none());
        assert!(game.state.replacements.is_none());
    }
}
const JUSTICE: &str = "Justice, Vance Astrovik";
const LOKI: &str = "Loki, God of Mischief";
fn finish_announcement(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseActivationCost { .. }
                )
            })
            .unwrap_or_else(|| panic!("unfinished {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("unfinished announcement");
}
fn announce(g: &mut Game, card: CardId, ability: u16) {
    checked(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn atlantis_return(g: &mut Game, p: Player, targets: &[CardId]) {
    priority(g, p);
    let id = named(g, p, "Atlantis Attacks", Zone::Hand);
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
    checked(g, Action::AnnounceCast { card: id });
    checked(g, Action::ChooseTeamwork { pay: false });
    checked(g, Action::ChooseMode { mode: 1 });
    for &target in targets {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    finish_announcement(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
}
fn counters(g: &Game, id: CardId) -> u64 {
    g.counter_count(id, CounterKind::PlusOne)
}
fn loki_packets(g: &Game) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(g.state.stack.iter())
        .chain(g.state.ordering_triggers.iter())
        .filter(|s| s.source_card.name() == LOKI)
        .count()
}
fn reject(g: &mut Game, a: Action) {
    let snap = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), snap);
    assert_eq!(g.revision(), revision);
}
#[test]
fn msh_return_target_core_justice_paid_returns_count_tokens_and_other_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let justice = gated_source(&mut g, p, JUSTICE, Zone::Battlefield);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        paid(&mut g, p, "Unsummon", &[Target::Card(bear)]);
        assert_eq!(counters(&g, justice), 1);
        paid(&mut g, p, "Mole Man, Moloid Master", &[]);
        let forest = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: forest });
        settle(&mut g);
        let moloid = token_ids(&g, p, "Moloid")[0];
        paid(&mut g, p, "Unsummon", &[Target::Card(moloid)]);
        assert_eq!(
            counters(&g, justice),
            2,
            "actual token return triggers before cessation"
        );
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        paid(&mut g, p, "Unsummon", &[Target::Card(foreign)]);
        assert_eq!(
            counters(&g, justice),
            2,
            "other controller does not qualify"
        );
        paid(&mut g, p, "Unsummon", &[Target::Card(justice)]);
        assert_eq!(g.object_view(justice).zone, Zone::Hand);
        assert_eq!(
            counters(&g, justice),
            0,
            "source never observes its own return"
        );
    }
}
#[test]
fn msh_return_target_core_justice_whole_batch_freezes_control_and_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let justice = gated_source(&mut g, p, JUSTICE, Zone::Battlefield);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let secret = named(&mut g, p.other(), "Cancel", Zone::Hand);
        let aura = paid(&mut g, p, "Confiscate", &[Target::Card(bear)]);
        assert_eq!(g.object_view(bear).controller, p);
        atlantis_return(&mut g, p, &[aura, bear]);
        assert_eq!(
            counters(&g, justice),
            2,
            "both controlled before the simultaneous return"
        );
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert_eq!(g.state.objects[bear as usize].owner, p.other());
        assert!(
            g.observe(p).objects.iter().any(|v| v.id == bear),
            "the returned public identity remains known"
        );
        assert!(!g.observe(p).objects.iter().any(|v| v.id == secret));
        assert!(g.observe(p.other()).objects.iter().any(|v| v.id == bear));
    }
}
#[test]
fn msh_return_target_core_justice_source_batch_departure_queues_old_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let justice = gated_source(&mut g, p, JUSTICE, Zone::Battlefield);
        let old = g.object_ref(justice);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        // Resolve the real paid simultaneous-return spell only as far as the
        // frozen trigger, then inspect its source before ordinary resolution.
        let id = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
        checked(&mut g, Action::AnnounceCast { card: id });
        checked(&mut g, Action::ChooseTeamwork { pay: false });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        for card in [justice, bear] {
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        finish_announcement(&mut g);
        for _ in 0..30 {
            if g.object_view(justice).zone == Zone::Hand {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        let packets: Vec<_> = g
            .state
            .pending_triggers
            .iter()
            .chain(g.state.stack.iter())
            .chain(g.state.ordering_triggers.iter())
            .filter(|s| s.source_card.name() == JUSTICE)
            .collect();
        assert_eq!(packets.len(), 1);
        assert_eq!(packets[0].object, old);
        assert_ne!(g.object_ref(justice), old);
        settle(&mut g);
        assert_eq!(counters(&g, justice), 0);
        assert_eq!(g.object_view(justice).zone, Zone::Hand);
    }
}
#[test]
fn msh_return_target_core_justice_looks_back_at_suppression_before_aura_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let justice = gated_source(&mut g, p, JUSTICE, Zone::Battlefield);
        let protection = paid(&mut g, p, "Witness Protection", &[Target::Card(justice)]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        atlantis_return(&mut g, p, &[protection, bear]);
        assert!(g.object_view(justice).printed_abilities_available());
        assert_eq!(
            counters(&g, justice),
            0,
            "restored abilities did not exist at the event lookback"
        );
        let again = paid(&mut g, p, "Bear Cub", &[]);
        paid(&mut g, p, "Unsummon", &[Target::Card(again)]);
        assert_eq!(counters(&g, justice), 1);
    }
}
#[test]
fn msh_return_target_core_justice_actual_borrowed_maze_return_cost_is_nonland() {
    for p in [Player::P0, Player::P1] {
        for borrowed in [false, true] {
            let mut g = setup(p);
            let justice = gated_source(&mut g, p, JUSTICE, Zone::Battlefield);
            let maze = named(&mut g, p, "Maze's End", Zone::Battlefield);
            let source = if borrowed {
                super::territory_forge_tests::forge(&mut g, p, maze)
            } else {
                maze
            };
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            if borrowed {
                let view = g
                    .borrowed_ability_views()
                    .into_iter()
                    .find(|v| v.source.card == source && v.ability == 1)
                    .unwrap();
                checked(
                    &mut g,
                    Action::AnnounceBorrowedAbility {
                        card: source,
                        exiled: view.exiled,
                        ability: 1,
                        counter_payment: Counters::default(),
                    },
                );
            } else {
                announce(&mut g, source, 1);
            }
            let object = g.object_ref(source);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish_announcement(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                Zone::Hand,
                "return paid before search"
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(counters(&g, justice), u64::from(borrowed));
            assert_eq!(
                g.object_view(maze).zone,
                if borrowed { Zone::Exile } else { Zone::Hand }
            );
        }
    }
}
#[test]
fn msh_return_target_core_loki_paid_equip_acquires_before_payment_once_and_private_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let before = g.hand_count(p);
        let top = *g.state.players[p.index()].library.last().unwrap();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert_eq!(
            loki_packets(&g),
            1,
            "target acquisition precedes cost payment"
        );
        assert_eq!(g.hand_count(p), before);
        let snapshot = g.snapshot();
        let mut orphan = {
            let parent = g.clone();
            parent.clone()
        };
        finish_announcement(&mut g);
        settle(&mut g);
        finish_announcement(&mut orphan);
        settle(&mut orphan);
        assert_eq!(g.snapshot(), orphan.snapshot());
        assert_eq!(g.hand_count(p), before + 1);
        assert_eq!(g.object_view(pole).attached_to, Some(g.object_ref(bear)));
        assert!(g.observe(p).objects.iter().any(|v| v.id == top));
        assert!(!g.observe(p.other()).objects.iter().any(|v| v.id == top));
        assert_eq!(g.object_view(loki).zone, Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish_announcement(&mut g);
        settle(&mut g);
        assert_eq!(
            g.hand_count(p),
            before + 1,
            "same source instance only once this turn"
        );
        orphan.restore(&snapshot).unwrap();
        finish_announcement(&mut orphan);
        settle(&mut orphan);
        assert_eq!(orphan.hand_count(p), before + 1);
    }
}
#[test]
fn msh_return_target_core_loki_real_target_then_sacrificed_observer_keeps_draw_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ayli = paid(&mut g, p, "Ayli, Eternal Pilgrim", &[]);
        let loki = gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        // Bounded valid life-guard setup; activation, target, sacrifice and mana
        // are actual paid actions. This does not claim a life-gain producer.
        g.state.players[p.index()].life = 30;
        g.state.players[p.index()].mana = [1, 0, 1, 0, 0, 1];
        let before = g.hand_count(p);
        announce(&mut g, ayli, 1);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        assert_eq!(loki_packets(&g), 1);
        let object = g.object_ref(loki);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish_announcement(&mut g);
        assert_eq!(g.object_view(loki).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
    }
}
#[test]
fn msh_return_target_core_loki_spells_opposing_abilities_and_nontargets_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let before = g.hand_count(p);
        paid(&mut g, p, "Unsummon", &[Target::Card(bear)]);
        assert_eq!(
            g.hand_count(p),
            before + 1,
            "only the returned Bear enters hand"
        );
        assert_eq!(loki_packets(&g), 0);
        let foreign_pole = named(&mut g, p.other(), "Fishing Pole", Zone::Battlefield);
        let foreign_bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        // Bounded packet exercises the actual common hook with the opposing
        // ability controller; not a claim to pay sorcery equip off turn.
        let item = g.activation_item(p.other(), foreign_pole, 0);
        let bound = g.bind_target(Target::Card(foreign_bear));
        g.target_becomes_bound(&item, bound);
        assert_eq!(loki_packets(&g), 0);
        let archive = paid(&mut g, p, "Hedron Archive", &[]);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceMana {
                card: archive,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 2],
            },
        );
        finish_announcement(&mut g);
        settle(&mut g);
        assert_eq!(loki_packets(&g), 0);
    }
}
#[test]
fn msh_return_target_core_loki_failed_payment_is_atomic_and_owned_restore_replays_acquisition() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let before = g.snapshot();
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        assert_eq!(loki_packets(&g), 1);
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(loki_packets(&g), 1);
        finish_announcement(&mut g);
        settle(&mut g);
        let mut orphan = {
            let parent = g.clone();
            parent.clone()
        };
        orphan.restore(&before).unwrap();
        orphan.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut orphan, pole, 0);
        checked(
            &mut orphan,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert_eq!(loki_packets(&orphan), 1);
        finish_announcement(&mut orphan);
        settle(&mut orphan);
        assert_eq!(g.object_view(pole), orphan.object_view(pole));
        assert_eq!(g.hand_count(p), orphan.hand_count(p));
    }
}
#[test]
fn msh_return_target_core_loki_bounded_copied_ability_player_and_permanent_entity_acquisition() {
    for p in [Player::P0, Player::P1] {
        for player_target in [false, true] {
            let mut g = setup(p);
            gated_source(&mut g, p, LOKI, Zone::Battlefield);
            let bullseye = named(&mut g, p, "Bullseye, Death Dealer", Zone::Battlefield);
            let target = if player_target {
                Target::Player(p.other())
            } else {
                Target::Card(named(&mut g, p.other(), "Bear Cub", Zone::Battlefield))
            };
            // No admitted ability-copy producer exists. This bounded fixture
            // supplies a copied real activated packet to the production final
            // acquisition hook, independently of the spell-only allocator.
            let mut copy = g.activation_item(p, bullseye, 0);
            copy.is_copy = true;
            let mut bound = g.bind_target(target);
            bound.group = 0;
            copy.targets = vec![bound];
            let mut duplicate = bound;
            duplicate.group = 1;
            copy.targets.push(duplicate);
            g.acquire_stack_target_set(&copy, &[]);
            assert_eq!(loki_packets(&g), 1);
            let count = g.state.pending_triggers.len();
            g.acquire_stack_target_set(&copy, &copy.targets);
            assert_eq!(
                g.state.pending_triggers.len(),
                count,
                "unchanged entity is not reacquired"
            );
            let before = g.hand_count(p);
            settle(&mut g);
            assert_eq!(g.hand_count(p), before + 1);
        }
    }
}
fn bullseye_activation(g: &mut Game, p: Player, target: Target) -> CardId {
    // Physical admitted source setup; every printed activation cost below is
    // paid through the normal announcement. No ETB/cast claim for this source.
    priority(g, p);
    let source = named(g, p, "Bullseye, Death Dealer", Zone::Battlefield);
    let discard = named(g, p, "Bear Cub", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    announce(g, source, 0);
    checked(g, Action::ChooseTarget { target });
    let object = g.object_ref(discard);
    checked(g, Action::ChooseAdditionalCost { object });
    finish_announcement(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    assert!(g.object_view(source).tapped);
    source
}
#[test]
fn msh_return_target_core_loki_real_paid_player_target_and_source_independence() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let before = g.hand_count(p);
        let life = g.state.players[p.other().index()].life;
        let source = bullseye_activation(&mut g, p, Target::Player(p.other()));
        assert_eq!(
            loki_packets(&g),
            1,
            "Player target reaches global observer before Card-only logic"
        );
        // Actual paid Unsummon removes the causing ability's source without
        // countering either independent ability.
        paid(&mut g, p, "Unsummon", &[Target::Card(source)]);
        assert_eq!(g.hand_count(p), before + 2, "draw plus returned source");
        assert_eq!(g.state.players[p.other().index()].life, life - 2);
    }
}
#[test]
fn msh_return_target_core_loki_graveyard_targets_and_funding_queries_are_pure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let harvester = paid(&mut g, p, "Barkform Harvester", &[]);
        let grave = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let snapshot = g.snapshot();
        let state_bytes = g.state_bytes();
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 1, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        for _ in 0..3 {
            assert!(g.can_fund(goal.clone()));
            let _ = g.legal_actions();
            let _ = g.observe(p);
            let _ = g.observe(p.other());
        }
        assert_eq!(g.snapshot(), snapshot);
        assert_eq!(g.state_bytes(), state_bytes);
        assert_eq!(loki_packets(&g), 0);
        assert!(!g.object_view(forest).tapped);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, harvester, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grave),
            },
        );
        assert_eq!(loki_packets(&g), 0);
        finish_announcement(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(grave).zone, Zone::Library);
    }
}
#[test]
fn msh_return_target_core_loki_actual_retarget_uses_ability_controller_not_chooser() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let source = bullseye_activation(&mut g, p, Target::Card(a));
        let victim = g
            .state
            .stack
            .iter()
            .find(|s| s.card == source && s.kind == StackKind::ActivatedAbility { ability: 0 })
            .unwrap()
            .id;
        // Late observer setup makes the genuine retarget its first acquisition.
        gated_source(&mut g, p, LOKI, Zone::Battlefield);
        gated_source(&mut g, p.other(), LOKI, Zone::Battlefield);
        priority(&mut g, p.other());
        let bolt = named(&mut g, p.other(), "Bolt Bend", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 3];
        cast(&mut g, bolt, None, &[Target::Stack(victim)]);
        assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::Retarget { .. }) {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        assert_eq!(loki_packets(&g), 0, "stack target is not a permanent");
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        assert_eq!(
            loki_packets(&g),
            0,
            "provisional edits are not acquisitions"
        );
        checked(&mut g, Action::RestoreStackTarget { index: 0 });
        reject(&mut g, Action::FinishStackTargets);
        assert_eq!(
            loki_packets(&g),
            0,
            "restoring the old entity does not reacquire it"
        );
        checked(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        checked(&mut g, Action::FinishStackTargets);
        let packets: Vec<_> = g
            .state
            .pending_triggers
            .iter()
            .chain(g.state.stack.iter())
            .chain(g.state.ordering_triggers.iter())
            .filter(|s| s.source_card.name() == LOKI)
            .collect();
        assert_eq!(packets.len(), 1);
        assert_eq!(
            packets[0].controller, p,
            "original ability controller determines you"
        );
        settle(&mut g);
        assert_eq!(g.object_view(a).damage, 0);
        assert_eq!(g.object_view(b).damage, 2);
    }
}
#[test]
fn msh_return_target_core_loki_copy_origin_acquisition_and_expiry_restore_once_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let before = g.hand_count(p);
        for acquisition in 0..3 {
            if acquisition == 1 {
                // Bounded copy-layer setup isolates the already accepted
                // origin contract. No additional copy producer is admitted.
                let object = g.object_ref(loki);
                g.apply_temporary_copy(
                    object,
                    CopyRecipe::new(Card::from_name(LOKI).unwrap()),
                    CopyDeadline::EndOfTurn,
                );
            } else if acquisition == 2 {
                assert!(g.expire_temporary_copies(true));
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            announce(&mut g, pole, 0);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            assert_eq!(loki_packets(&g), usize::from(acquisition < 2));
            finish_announcement(&mut g);
            settle(&mut g);
        }
        assert_eq!(
            g.hand_count(p),
            before + 2,
            "new copy origin triggers; restored used original does not"
        );
        assert!(
            g.state
                .ability_target_sources
                .as_ref()
                .is_some_and(|s| s.contains(&loki))
        );
        g.move_object(loki, Zone::Hand);
        assert!(g.state.ability_target_sources.is_none());
    }
}
#[test]
fn msh_return_target_core_loki_suppression_does_not_consume_once_and_restoration_registers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let loki = gated_source(&mut g, p, LOKI, Zone::Battlefield);
        let pole = paid(&mut g, p, "Fishing Pole", &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(loki)]);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert_eq!(loki_packets(&g), 0);
        finish_announcement(&mut g);
        settle(&mut g);
        atlantis_return(&mut g, p, &[aura]);
        let before = g.hand_count(p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, pole, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        assert_eq!(loki_packets(&g), 1);
        finish_announcement(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_return_target_core_loki_exact_incarnation_reacquisition_and_stale_reference() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Bullseye, Death Dealer", Zone::Battlefield);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let old = g.bind_target(Target::Card(bear));
        let mut packet = g.activation_item(p, source, 0);
        packet.targets = vec![old];
        paid(&mut g, p, "Unsummon", &[Target::Card(bear)]);
        // Bounded immediate reentry isolates exact target identity; both paid
        // spells above are real, and no flash/recast permission is invented.
        g.move_object(bear, Zone::Battlefield);
        settle(&mut g);
        gated_source(&mut g, p, LOKI, Zone::Battlefield);
        g.target_becomes_bound(&packet, old);
        assert_eq!(
            loki_packets(&g),
            0,
            "stale reference is not current permanent acquisition"
        );
        packet.targets = vec![g.bind_target(Target::Card(bear))];
        assert_ne!(packet.targets[0].object, old.object);
        g.acquire_stack_target_set(&packet, &[old]);
        assert_eq!(loki_packets(&g), 1, "same slot, new exact permanent");
        let before = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 1);
    }
}
#[test]
fn msh_incursion_core_nonlegendary_preserves_flat_replacement_types_after_paid_taskmaster() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Worldwalker Helm", &[]);
        let blueprint = paid(&mut g, p, "Barkform Harvester", &[]);
        let task = paid(&mut g, p, "Taskmaster, Mercenary Mimic", &[]);
        let turn = g.state.turn;
        for _ in 0..240 {
            if g.state.turn > turn && g.state.active == p && g.state.step == Step::Main1 {
                break;
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
                            | Action::Discard { .. }
                    )
                })
                .unwrap();
            checked(&mut g, a);
        }
        g.prepare_triggers();
        let choose = Action::ChooseTarget {
            target: Target::Card(blueprint),
        };
        for _ in 0..30 {
            if g.legal_actions().contains(&choose) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. } | Action::Pass))
                .unwrap();
            checked(&mut g, a);
        }
        checked(&mut g, choose);
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.object_view(task).kinds, 1);
        assert!(!g.keywords(task).contains(Keywords::CHANGELING));
        bounded_incursion_cast(&mut g, p);
        let copies = token_ids(&g, p, "Taskmaster, Mercenary Mimic");
        assert_eq!(copies.len(), 1);
        let view = g.object_view(copies[0]);
        assert_eq!(view.kinds, 1);
        assert!(!view.is_legendary());
        assert!(!view.keywords.contains(Keywords::CHANGELING));
        for subtype in ["Human", "Mercenary", "Villain"] {
            assert!(view.has_subtype(subtype));
        }
        assert!(!view.has_subtype("Shapeshifter"));
        assert_eq!(
            token_ids(&g, p, "Map Token").len(),
            1,
            "only actual artifact Barkform copy receives Helm"
        );
    }
}
#[test]
fn msh_incursion_core_paid_dfc_blueprint_keeps_both_faces_and_nonlegendary_exception() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let original = paid(&mut g, p, "Monica Rambeau", &[]);
        bounded_incursion_cast(&mut g, p);
        let copies = token_ids(&g, p, "Monica Rambeau");
        assert_eq!(copies.len(), 1);
        let token = copies[0];
        let before = g.object_ref(token);
        assert!(g.object_view(token).face.is_some());
        assert!(!g.object_view(token).is_legendary());
        assert_eq!(g.object_view(original).zone, Zone::Battlefield);
        g.state.players[p.index()].mana = [2, 0, 0, 1, 0, 2];
        announce(&mut g, token, 0);
        finish_announcement(&mut g);
        settle(&mut g);
        assert_eq!(g.object_ref(token), before);
        assert_eq!(g.object_view(token).name(), "Photon, Living Light");
        assert!(!g.object_view(token).is_legendary());
        assert!(g.object_view(token).face.as_ref().unwrap().back);
    }
}
