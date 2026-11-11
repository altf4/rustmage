//! Printed Discover and explore consumers from The Lost Caverns of Ixalan.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn discover(game: &Game, player: Player, found: CardId) {
    assert_eq!(game.state.pending, Pending::ResolutionCasts { player });
    let result = game.observe(player.other()).cascade.unwrap();
    assert_eq!(result.discovered, Some(game.object_ref(found)));
}

fn take_discovery(game: &mut Game, player: Player, found: CardId) {
    discover(game, player, found);
    act(game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(found).zone, Zone::Hand);
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn select(game: &Game, card: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(card),
    }
}

fn candidate(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

#[test]
fn daring_discovery_marks_surviving_targets_and_curator_repeats_the_value_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let curator = candidate(
        &mut game,
        player,
        "Curator of Sun's Creation",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, curator), (3, 3));
    let targets = ["Grizzly Bears", "Hill Giant", "Savannah Lions"]
        .map(|name| put(&mut game, player.other(), name, Zone::Battlefield));
    let untouched = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let second_spell = candidate(&mut game, player, "Daring Discovery", Zone::Hand);
    let spell = candidate(&mut game, player, "Daring Discovery", Zone::Hand);
    let third_hit = put(&mut game, player, "Opt", Zone::Library);
    let repeated_hit = put(&mut game, player, "Hill Giant", Zone::Library);
    let first_hit = put(&mut game, player, "Shock", Zone::Library);
    announce(&mut game, spell, &targets.map(Target::Card));

    let mut all_illegal = game.clone();
    for id in targets {
        all_illegal.move_object(id, Zone::Graveyard);
    }
    settle(&mut all_illegal);
    assert_eq!(all_illegal.object_view(first_hit).zone, Zone::Library);

    game.move_object(targets[0], Zone::Graveyard);
    pass_pair(&mut game);
    for id in [targets[1], targets[2]] {
        assert!(game.object_view(id).keywords.contains(Keywords::CANT_BLOCK));
    }
    assert!(
        !game
            .object_view(untouched)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    );
    take_discovery(&mut game, player, first_hit);
    pass_pair(&mut game);
    take_discovery(&mut game, player, repeated_hit);
    settle(&mut game);

    announce(&mut game, second_spell, &[]);
    pass_pair(&mut game);
    take_discovery(&mut game, player, third_hit);
    settle(&mut game);
}

#[test]
fn buried_treasure_pays_mana_and_its_graveyard_ability_exiles_it_before_discovering() {
    let player = Player::P1;
    let mut game = scenario(player);
    let buried = candidate(&mut game, player, "Buried Treasure", Zone::Battlefield);
    for color in 0..5 {
        let mut branch = game.clone();
        branch.state.players[player.index()].mana = [0; 6];
        let mut production = [0; 6];
        production[color] = 1;
        choose(&mut branch, |a| match a {
            Action::ActivateMana {
                card,
                ability: 0,
                production: found,
                ..
            }
            | Action::AnnounceMana {
                card,
                ability: 0,
                production: found,
                ..
            } => *card == buried && *found == production,
            _ => false,
        });
        if matches!(branch.state.pending, Pending::PayCosts { .. }) {
            pay(&mut branch);
        }
        assert_eq!(branch.state.players[player.index()].mana, production);
        assert_eq!(branch.object_view(buried).zone, Zone::Graveyard);
    }
    game.move_object(buried, Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    reject(&mut game, activate(buried, 1));
    game.state.players[player.index()].mana[5] = 5;
    game.state.step = Step::End;
    reject(&mut game, activate(buried, 1));
    game.state.step = Step::Main2;
    let hit = put(&mut game, player, "Serra Angel", Zone::Library);
    act(&mut game, activate(buried, 1));
    pay(&mut game);
    assert_eq!(game.object_view(buried).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    take_discovery(&mut game, player, hit);
}

#[test]
fn caparocti_taps_two_untapped_artifacts_or_creatures_to_discover() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = candidate(&mut game, player, "Caparocti Sunborn", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
    let hit = put(&mut game, player, "Short Bow", Zone::Library);
    game.state.objects[creature as usize].summoning_sick = true;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);

    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.object_view(hit).zone, Zone::Library);
    assert!(!declined.object_view(creature).tapped);
    let tap_choice = |game: &Game, card| Action::Sacrifice {
        object: game.object_ref(card),
    };
    let invalid = tap_choice(&game, enemy);
    reject(&mut game, invalid);
    let first = tap_choice(&game, creature);
    act(&mut game, first.clone());
    reject(&mut game, first);
    let second = tap_choice(&game, artifact);
    act(&mut game, second);
    for id in [creature, artifact] {
        assert!(game.object_view(id).tapped);
    }
    take_discovery(&mut game, player, hit);
}

#[test]
fn chimil_protects_spells_while_present_and_discovers_five_on_its_own_end_step() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = candidate(
        &mut game,
        player,
        "Chimil, the Inner Sun",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    let counter = put(&mut game, player.other(), "Negate", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(player.other())));
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(spell)));
    let mut left = game.clone();
    left.move_object(source, Zone::Graveyard);
    settle(&mut left);
    settle(&mut game);
    assert_eq!(left.state.players[player.other().index()].life, 20);
    assert_eq!(game.state.players[player.other().index()].life, 18);

    let hit = put(&mut game, player, "Serra Angel", Zone::Library);
    let mut opposing = game.clone();
    opposing.state.active = player.other();
    opposing.begin_step(Step::End);
    opposing.prepare_triggers();
    assert!(opposing.state.stack.is_empty());
    game.begin_step(Step::End);
    game.prepare_triggers();
    pass_pair(&mut game);
    take_discovery(&mut game, player, hit);
}

#[test]
fn defossilize_explores_twice_on_the_exact_returned_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let original = game.object_ref(creature);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let nonland = put(&mut game, player, "Opt", Zone::Library);
    let spell = candidate(&mut game, player, "Defossilize", Zone::Hand);
    reject(
        &mut game,
        Action::Cast {
            card: spell,
            target: Some(Target::Card(enemy)),
            payment: [0, 0, 0, 0, 0, 5],
        },
    );
    cast(&mut game, spell, Some(Target::Card(creature)));
    let mut stale = game.clone();
    stale.move_object(creature, Zone::Exile);
    stale.move_object(creature, Zone::Graveyard);
    settle(&mut stale);
    assert!(stale.observe(player).last_explore.is_none());

    pass_pair(&mut game);
    assert_ne!(original, game.object_ref(creature));
    let choice = game.observe(player).explore_choice.unwrap();
    assert_eq!(choice.explorer.object, game.object_ref(creature));
    assert_eq!(
        game.observe(player.other()).explore_choice,
        Some(choice.clone())
    );
    let keep = Action::ChooseExplore {
        object: choice.revealed.object,
        graveyard: false,
    };
    let mill = Action::ChooseExplore {
        object: choice.revealed.object,
        graveyard: true,
    };
    let saved = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == mill)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, mill);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    indexed.restore(&saved).unwrap();
    act(&mut indexed, keep.clone());
    act(&mut indexed, keep);
    assert_eq!(indexed.object_view(nonland).zone, Zone::Library);
    assert_eq!(indexed.counter_count(creature, CounterKind::PlusOne), 2);
}

#[test]
fn deepfathom_echo_explores_before_the_optional_nontargeted_copy() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = candidate(&mut game, player, "Deepfathom Echo", Zone::Battlefield);
    let donor = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let top = put(&mut game, player, "Shock", Zone::Library);
    game.grant_keyword_effects(&[donor], game.object_ref(donor), Keywords::HEXPROOF);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    pass_pair(&mut game);
    let choice = game.observe(player).explore_choice.unwrap();
    assert_eq!(choice.revealed.object, game.object_ref(top));
    act(
        &mut game,
        Action::ChooseExplore {
            object: choice.revealed.object,
            graveyard: false,
        },
    );
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(stats(&declined, source), (5, 5));
    let invalid = select(&game, enemy);
    reject(&mut game, invalid);
    let original = game.object_ref(source);
    let selected = select(&game, donor);
    act(&mut game, selected);
    assert_eq!(game.object_ref(source), original);
    assert_eq!(game.object_view(source).name, "Grizzly Bears");
    assert_eq!(stats(&game, source), (3, 3));
    assert!(
        !game
            .object_view(source)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    game.expire_temporary_copies(true);
    assert_eq!(game.object_view(source).name, "Deepfathom Echo");
    assert_eq!(stats(&game, source), (5, 5));
}
