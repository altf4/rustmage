use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}
fn pt(game: &Game, id: CardId) -> (i128, i128) {
    let obj = game.object_view(id);
    (obj.power, obj.toughness)
}
fn activate(game: &mut Game, source: CardId) {
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate {card,..} if *card==source))
        .unwrap();
    act(game, action);
    pass_pair(game);
}

#[test]
fn static_anthem_is_dynamic_stacks_and_restores_without_mutable_cache() {
    let mut game = scenario();
    let bear = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let hand = named(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    let first = named(
        &mut game,
        Player::P0,
        "Anthem of Champions",
        Zone::Battlefield,
    );
    let second = named(
        &mut game,
        Player::P0,
        "Anthem of Champions",
        Zone::Battlefield,
    );
    assert_eq!(pt(&game, bear), (4, 4));
    assert_eq!(pt(&game, enemy), (2, 2));
    assert_eq!(pt(&game, hand), (2, 2));
    assert_eq!(pt(&game, first), (0, 0));
    let saved = game.snapshot();
    let revision = game.revision();
    let state = game.state.clone();
    game.observe(Player::P0);
    game.legal_actions();
    assert_eq!(game.revision(), revision);
    assert_eq!(game.state, state);
    game.move_object(second, Zone::Hand);
    assert_eq!(pt(&game, bear), (3, 3));
    game.move_object(hand, Zone::Battlefield);
    assert_eq!(pt(&game, hand), (3, 3));
    game.move_object(first, Zone::Graveyard);
    assert_eq!(pt(&game, bear), (2, 2));
    game.restore(&saved).unwrap();
    assert_eq!(pt(&game, bear), (4, 4));
    assert_eq!(game.snapshot(), saved);
}

#[test]
fn lords_use_exact_subtypes_other_and_controller_and_buff_tokens() {
    let mut game = scenario();
    let first = named(
        &mut game,
        Player::P0,
        "Imperious Perfect",
        Zone::Battlefield,
    );
    let second = named(
        &mut game,
        Player::P0,
        "Imperious Perfect",
        Zone::Battlefield,
    );
    let elf = named(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let enemy = named(&mut game, Player::P1, "Llanowar Elves", Zone::Battlefield);
    let bear = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(pt(&game, first), (3, 3));
    assert_eq!(pt(&game, second), (3, 3));
    assert_eq!(pt(&game, elf), (3, 3));
    assert_eq!(pt(&game, enemy), (1, 1));
    assert_eq!(pt(&game, bear), (2, 2));
    game.state.players[0].mana[4] = 1;
    activate(&mut game, first);
    let token = game
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
        .unwrap()
        .0 as CardId;
    assert_eq!(pt(&game, token), (3, 3));
    game.move_object(first, Zone::Hand);
    assert_eq!(pt(&game, second), (2, 2));
    assert_eq!(pt(&game, token), (2, 2));
}

#[test]
fn flying_filter_and_attacking_filter_follow_current_battlefield() {
    let mut game = scenario();
    let eagle = named(&mut game, Player::P0, "Empyrean Eagle", Zone::Battlefield);
    let flyer = named(&mut game, Player::P0, "Healer's Hawk", Zone::Battlefield);
    let bear = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let flag = named(&mut game, Player::P0, "Goblin Oriflamme", Zone::Battlefield);
    assert_eq!(pt(&game, eagle), (2, 3));
    assert_eq!(pt(&game, flyer), (2, 2));
    assert_eq!(pt(&game, bear), (2, 2));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: bear });
    act(&mut game, Action::FinishAttackers);
    assert_eq!(pt(&game, bear), (3, 2));
    assert_eq!(pt(&game, flyer), (2, 2));
    game.state.step = Step::EndCombat;
    game.advance_step();
    assert_eq!(pt(&game, bear), (2, 2));
    game.move_object(flag, Zone::Hand);
    assert_eq!(pt(&game, flag), (0, 0));
}

#[test]
fn removing_lord_repeats_sbas_and_captures_whole_simultaneous_lki_batch() {
    let mut game = scenario();
    let lord = named(
        &mut game,
        Player::P0,
        "Imperious Perfect",
        Zone::Battlefield,
    );
    let elf = named(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    game.state.objects[lord as usize].damage = 2;
    game.state.objects[elf as usize].damage = 1;
    game.state_based_actions();
    assert_eq!(game.state.objects[lord as usize].zone, Zone::Graveyard);
    assert_eq!(game.state.objects[elf as usize].zone, Zone::Graveyard);
    // A separate simultaneous batch must preserve the lord's contribution even
    // for a creature moved after that lord in the vector.
    let lord = named(
        &mut game,
        Player::P0,
        "Imperious Perfect",
        Zone::Battlefield,
    );
    let elf = named(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let events = game.move_objects_raw(&[lord, elf], Zone::Graveyard);
    assert_eq!((events[1].before.power, events[1].before.toughness), (2, 2));
    assert_eq!(pt(&game, elf), (1, 1));
}

#[test]
fn resolving_mass_bonus_locks_set_but_static_bonus_tracks_later_entrants() {
    let mut game = scenario();
    let bear = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let anthem = named(
        &mut game,
        Player::P0,
        "Anthem of Champions",
        Zone::Battlefield,
    );
    let spell = named(&mut game, Player::P0, "On the Job", Zone::Hand);
    game.state.players[0].mana = [4, 0, 0, 0, 0, 0];
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(pt(&game, bear), (5, 4));
    assert_eq!(pt(&game, enemy), (2, 2));
    let later = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(pt(&game, later), (3, 3));
    game.move_object(anthem, Zone::Hand);
    assert_eq!(pt(&game, bear), (4, 3));
    game.move_object(bear, Zone::Hand);
    game.move_object(bear, Zone::Battlefield);
    assert_eq!(pt(&game, bear), (2, 2));
}

#[test]
fn asymmetric_modifiers_counters_and_cleanup_combine_in_same_layer() {
    let mut game = scenario();
    let bear = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    named(
        &mut game,
        Player::P0,
        "Anthem of Champions",
        Zone::Battlefield,
    );
    game.state.objects[bear as usize].counters.plus_one = 1;
    let spell = named(&mut game, Player::P0, "Sugar Rush", Zone::Hand);
    game.state.players[0].mana = [2, 0, 1, 0, 0, 0];
    cast(&mut game, spell, Some(Target::Card(bear)));
    pass_pair(&mut game);
    assert_eq!(pt(&game, bear), (7, 4));
    let saved = game.snapshot();
    let mut branch = game.clone();
    branch.finish_cleanup();
    assert_eq!(pt(&branch, bear), (4, 4));
    assert_eq!(pt(&game, bear), (7, 4));
    game.finish_cleanup();
    assert_eq!(game.state, branch.state);
    game.restore(&saved).unwrap();
    assert_eq!(pt(&game, bear), (7, 4));
}

#[test]
fn self_pump_abilities_keep_independent_stats_and_can_kill_their_source() {
    let mut game = scenario();
    let nautilus = named(
        &mut game,
        Player::P0,
        "Hermitic Nautilus",
        Zone::Battlefield,
    );
    let before = pt(&game, nautilus);
    game.state.players[0].mana[1] = 4;
    activate(&mut game, nautilus);
    assert_eq!(pt(&game, nautilus), (before.0 + 3, before.1 - 3));
    activate(&mut game, nautilus);
    assert_eq!(game.state.objects[nautilus as usize].zone, Zone::Graveyard);
    assert_eq!(pt(&game, nautilus), before);
}

#[test]
fn damage_assignment_uses_anthem_power_and_toughness() {
    let mut game = scenario();
    let attacker = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let blocker = named(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    named(
        &mut game,
        Player::P0,
        "Anthem of Champions",
        Zone::Battlefield,
    );
    game.state.attackers.push(Attacker {
        card: attacker,
        blocked: true,
    });
    game.state.blocking.push(blocker);
    game.state.blocks.push((blocker, attacker));
    game.state.step = Step::CombatDamage;
    game.prepare_damage();
    game.state_based_actions();
    assert_eq!(game.state.objects[blocker as usize].zone, Zone::Graveyard);
    assert_eq!(
        game.state.objects[attacker as usize].zone,
        Zone::Battlefield
    );
    assert_eq!(game.state.objects[attacker as usize].damage, 2);
}

#[test]
fn source_index_tracks_real_entry_departure_reentry_and_restore() {
    let mut game = scenario();
    let spell = named(&mut game, Player::P0, "Anthem of Champions", Zone::Hand);
    game.state.players[0].mana = [1, 0, 0, 0, 1, 0];
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert_eq!(game.state.static_sources, vec![spell]);
    let saved = game.snapshot();
    game.move_object(spell, Zone::Hand);
    assert!(game.state.static_sources.is_empty());
    game.move_object(spell, Zone::Battlefield);
    assert_eq!(game.state.static_sources, vec![spell]);
    game.move_object(spell, Zone::Graveyard);
    assert!(game.state.static_sources.is_empty());
    game.restore(&saved).unwrap();
    assert_eq!(game.state.static_sources, vec![spell]);
    let mut branch = game.clone();
    branch.move_object(spell, Zone::Hand);
    assert!(branch.state.static_sources.is_empty());
    assert_eq!(game.state.static_sources, vec![spell]);
}

#[test]
fn creature_filters_use_exact_earlier_layer_characteristics() {
    let mut game = scenario();
    let source = named(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let artifact = named(&mut game, Player::P0, "Sterling Hound", Zone::Battlefield);
    let enemy = named(&mut game, Player::P1, "Llanowar Elves", Zone::Battlefield);
    let elf = named(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let flyer = named(&mut game, Player::P0, "Healer's Hawk", Zone::Battlefield);
    let token_card = Card::catalogue()
        .find(|c| {
            c.token_info()
                .is_some_and(|info| info.subtypes == "Elf Warrior")
        })
        .unwrap();
    game.create_tokens(Player::P0, token_card, 1, false);
    let token = (game.state.objects.len() - 1) as CardId;
    game.state.attackers.push(Attacker {
        card: source,
        blocked: false,
    });
    let all = CreatureFilter {
        relation: -1,
        other: false,
        subtype: None,
        keyword: Card::GrizzlyBears.keywords(),
        attacking: false,
        tokens: None,
        artifact: false,
    };
    for (filter, expected) in [
        (
            CreatureFilter {
                relation: 0,
                other: true,
                ..all
            },
            vec![artifact, elf, flyer, token],
        ),
        (CreatureFilter { relation: 1, ..all }, vec![enemy]),
        (
            CreatureFilter {
                subtype: Some("Elf"),
                relation: 0,
                ..all
            },
            vec![elf, token],
        ),
        (
            CreatureFilter {
                tokens: Some(true),
                ..all
            },
            vec![token],
        ),
        (
            CreatureFilter {
                artifact: true,
                ..all
            },
            vec![artifact],
        ),
        (
            CreatureFilter {
                attacking: true,
                ..all
            },
            vec![source],
        ),
        (
            CreatureFilter {
                keyword: Keywords::FLYING,
                ..all
            },
            vec![flyer],
        ),
    ] {
        let actual: Vec<_> = (0..game.state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| game.creature_matches(filter, Player::P0, game.object_ref(source), id))
            .collect();
        assert_eq!(actual, expected);
    }
}

#[test]
fn self_modifier_does_not_follow_a_source_into_a_new_incarnation() {
    let mut game = scenario();
    let source = named(&mut game, Player::P0, "Shivan Dragon", Zone::Battlefield);
    game.state.players[0].mana[3] = 1;
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,..} if *card==source))
        .unwrap();
    act(&mut game, action);
    game.move_object(source, Zone::Hand);
    game.move_object(source, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(pt(&game, source), (5, 5));
    assert_eq!(game.state.objects[source as usize].growth, Stats::default());
}
