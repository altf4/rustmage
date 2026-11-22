use super::{activation, begin_attack, enter_step, life, pass_pair, target, tokens};
use crate::support::*;
use rustmage_core::{cards::*, engine::*};

fn cast_down(game: &mut Game, card: CardId) -> ObjectRef {
    let source = announce_hand_disguise(game, card);
    pay(game);
    settle(game);
    let exact = game.object_ref(source.card);
    assert!(game.is_face_down(exact));
    assert_eq!(stats(game, exact.card), (2, 2));
    exact
}

fn black(generic: u32, symbols: u32) -> ManaCost {
    ManaCost {
        colored: [0, 0, symbols, 0, 0, 0],
        generic,
    }
}

fn red(generic: u32, symbols: u32) -> ManaCost {
    ManaCost {
        colored: [0, 0, 0, symbols, 0, 0],
        generic,
    }
}

fn turn_up(game: &mut Game, object: ObjectRef, cost: ManaCost) {
    act(game, Action::BeginFaceUp { object, cost });
    choose(game, |action| matches!(action, Action::PayFaceUp { .. }));
    assert_eq!(game.object_ref(object.card), object);
    assert!(!game.is_face_down(object));
}

fn choose_player(game: &mut Game, player: Player) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
}

fn pick_pile(card: ObjectRef) -> Action {
    Action::ChooseEffectObject { object: card }
}

#[test]
fn alley_assailant_enters_tapped_only_face_up_and_drains_a_target_opponent_when_revealed() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let card = put(&mut game, player, "Alley Assailant", Zone::Hand);
    let mut normal = game.clone();
    cast(&mut normal, card, None);
    settle(&mut normal);
    assert!(normal.object_view(card).tapped);
    assert_eq!(stats(&normal, card), (3, 3));
    assert_eq!(life(&normal, player), 20);

    let exact = cast_down(&mut game, card);
    let card = exact.card;
    assert!(!game.object_view(card).tapped);
    reject(
        &mut game,
        Action::BeginFaceUp {
            object: exact,
            cost: black(2, 1),
        },
    );
    turn_up(&mut game, exact, black(4, 2));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    choose_player(&mut game, opponent);
    settle(&mut game);
    assert!(!game.object_view(card).tapped);
    assert_eq!(stats(&game, card), (3, 3));
    assert_eq!((life(&game, player), life(&game, opponent)), (23, 17));
}

#[test]
fn basilica_stalker_only_triggers_after_combat_damage_while_face_up() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let card = put(&mut game, player, "Basilica Stalker", Zone::Hand);
    let top = put(&mut game, player, "Shock", Zone::Library);
    let exact = cast_down(&mut game, card);
    let card = exact.card;
    assert!(!game.keywords(card).contains(Keywords::FLYING));
    game.state.objects[card as usize].summoning_sick = false;
    let mut hidden = game.clone();
    begin_attack(&mut hidden, card);
    pass_pair(&mut hidden);
    act(&mut hidden, Action::FinishBlockers);
    pass_pair(&mut hidden);
    assert_eq!(life(&hidden, opponent), 18);
    assert_eq!(life(&hidden, player), 20);
    assert!(hidden.state.stack.is_empty());

    turn_up(&mut game, exact, black(4, 1));
    assert!(game.keywords(card).contains(Keywords::FLYING));
    begin_attack(&mut game, card);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(life(&game, opponent), 17);
    assert_eq!(life(&game, player), 21);
    assert!(game.observe(player).library_choice.unwrap().surveil);
    assert!(game.observe(opponent).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn hunted_bonebrute_gives_the_opponent_dogs_and_only_drains_after_a_face_up_death() {
    const DOG: Card = Card::from_catalogue_id(5256);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let card = put(&mut game, player, "Hunted Bonebrute", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let mut normal = game.clone();
    cast(&mut normal, card, None);
    pass_pair(&mut normal);
    choose_player(&mut normal, opponent);
    settle(&mut normal);
    assert!(normal.keywords(card).contains(Keywords::MENACE));
    let dogs = tokens(&normal, opponent, DOG);
    assert_eq!(dogs.len(), 2);
    for dog in dogs {
        assert_eq!(stats(&normal, dog), (1, 1));
        assert_eq!(normal.object_view(dog).owner, opponent);
        assert_eq!(normal.object_view(dog).colors, 1);
    }

    let exact = cast_down(&mut game, card);
    let card = exact.card;
    assert!(tokens(&game, opponent, DOG).is_empty());
    let mut hidden = game.clone();
    cast(&mut hidden, shock, Some(Target::Card(card)));
    settle(&mut hidden);
    assert_eq!(hidden.object_view(card).zone, Zone::Graveyard);
    assert_eq!(life(&hidden, opponent), 20);

    turn_up(&mut game, exact, black(1, 1));
    assert!(tokens(&game, opponent, DOG).is_empty());
    assert_eq!(stats(&game, card), (6, 2));
    cast(&mut game, shock, Some(Target::Card(card)));
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    assert_eq!(life(&game, opponent), 17);
}

#[test]
fn nightdrinker_moroii_loses_life_for_normal_entry_only() {
    let player = Player::P1;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Nightdrinker Moroii", Zone::Hand);
    let mut normal = game.clone();
    cast(&mut normal, card, None);
    settle(&mut normal);
    assert_eq!(life(&normal, player), 17);

    let exact = cast_down(&mut game, card);
    let card = exact.card;
    assert!(!game.keywords(card).contains(Keywords::FLYING));
    assert_eq!(life(&game, player), 20);
    turn_up(&mut game, exact, black(0, 2));
    settle(&mut game);
    assert_eq!(life(&game, player), 20);
    assert_eq!(stats(&game, card), (4, 2));
    assert!(game.keywords(card).contains(Keywords::FLYING));
}

#[test]
fn bolrac_clan_basher_tramples_in_each_of_its_two_combat_damage_steps() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let card = put(&mut game, player, "Bolrac-Clan Basher", Zone::Hand);
    let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let exact = cast_down(&mut game, card);
    let card = exact.card;
    let keywords = Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE);
    assert!(!game.keywords(card).contains(keywords));
    turn_up(&mut game, exact, red(3, 2));
    assert!(game.keywords(card).contains(keywords));
    game.state.objects[card as usize].summoning_sick = false;
    begin_attack(&mut game, card);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::Block {
            attacker: card,
            blocker,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    for _ in 0..2 {
        let Some(range) = game.damage_assignment_range() else {
            break;
        };
        let amount = if range.target == Target::Card(blocker) {
            range.minimum
        } else {
            range.maximum
        };
        act(
            &mut game,
            Action::AssignDamage {
                target: range.target,
                amount,
            },
        );
    }
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(life(&game, opponent), 19);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(life(&game, opponent), 16);
}

#[test]
fn concealed_weapon_equips_normally_or_attaches_from_its_turn_up_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Concealed Weapon", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let mut normal = game.clone();
    cast(&mut normal, card, None);
    settle(&mut normal);
    assert_eq!(normal.object_view(card).attached_to, None);
    act(&mut normal, activation(card));
    target(&mut normal, ally);
    act(
        &mut normal,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 1],
        },
    );
    settle(&mut normal);
    assert_eq!(
        normal.object_view(card).attached_to,
        Some(normal.object_ref(ally))
    );
    assert_eq!(stats(&normal, ally), (5, 2));

    let exact = cast_down(&mut game, card);
    let card = exact.card;
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
    turn_up(&mut game, exact, red(2, 1));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, ally);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(
        game.object_view(card).attached_to,
        Some(game.object_ref(ally))
    );
    assert_eq!(stats(&game, ally), (5, 2));
    assert_eq!(stats(&game, enemy), (2, 2));
}

#[test]
fn expose_the_culprit_resolves_both_modes_in_order_and_rekeys_the_cloaked_pile() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Expose the Culprit", Zone::Hand);
    let hidden = put(&mut game, player, "Nightdrinker Moroii", Zone::Hand);
    let revealed = put(&mut game, player, "Bolrac-Clan Basher", Zone::Battlefield);
    let foreign = put(
        &mut game,
        opponent,
        "Nightdrinker Moroii",
        Zone::Battlefield,
    );
    let ordinary = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let hidden_ref = cast_down(&mut game, hidden);
    let hidden = hidden_ref.card;
    let revealed_ref = game.object_ref(revealed);

    act(&mut game, Action::AnnounceCast { card: spell });
    let mut first_only = game.clone();
    act(&mut first_only, Action::ChooseMode { mode: 0 });
    target(&mut first_only, hidden);
    pay(&mut first_only);
    settle(&mut first_only);
    assert!(!first_only.is_face_down(hidden_ref));
    assert!(first_only.on_battlefield(revealed));

    act(&mut game, Action::ChooseMode { mode: 2 });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(revealed),
        },
    );
    target(&mut game, hidden);
    pay(&mut game);
    pass_pair(&mut game);
    assert!(!game.is_face_down(hidden_ref));
    let wrong = game.object_ref(foreign);
    reject(&mut game, pick_pile(wrong));
    let wrong = game.object_ref(ordinary);
    reject(&mut game, pick_pile(wrong));
    let mut empty = game.clone();
    act(&mut empty, Action::FinishEffectChoice);
    assert_eq!(empty.object_ref(hidden), hidden_ref);

    act(&mut game, pick_pile(hidden_ref));
    act(&mut game, pick_pile(revealed_ref));
    let finish = Action::FinishEffectChoice;
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut raw = game.clone();
    let mut indexed = game.clone();
    drop(game);
    act(&mut raw, finish);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    let own = raw.observe(player).face_down_permanents;
    let other = raw.observe(opponent).face_down_permanents;
    assert_eq!(own.len(), 2);
    assert_eq!(other.len(), 2);
    for row in other {
        assert_eq!(row.cause, FaceDownCause::Cloak);
        assert!(row.underlying.is_none());
        assert!(![hidden, revealed].contains(&row.object.card));
        assert_eq!(stats(&raw, row.object.card), (2, 2));
    }
    for name in ["Nightdrinker Moroii", "Bolrac-Clan Basher"] {
        let card = Card::from_name(name).unwrap();
        assert!(own.iter().any(|row| row.underlying == Some(card)));
    }
}

#[test]
fn frantic_scapegoat_transfers_suspect_to_one_member_of_the_actual_entry_batch() {
    let player = Player::P1;
    let mut game = scenario(player);
    let goat = put(&mut game, player, "Frantic Scapegoat", Zone::Hand);
    let old = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let batch = put(&mut game, player, "Hop to It", Zone::Hand);
    cast(&mut game, goat, None);
    settle(&mut game);
    let source = game.object_ref(goat);
    assert_eq!(game.observe(player.other()).suspected, [source]);
    assert!(
        game.keywords(goat)
            .contains(Keywords::HASTE.union(Keywords::MENACE))
    );
    cast(&mut game, batch, None);
    pass_pair(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    let old = game.object_ref(old);
    reject(&mut game, pick_pile(old));
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.observe(player).suspected, [source]);

    let entrant = game.controlled_token_ids(player).next().unwrap();
    let exact = game.object_ref(entrant);
    act(&mut game, pick_pile(exact));
    settle(&mut game);
    assert_eq!(game.observe(player).suspected, [exact]);
    assert!(!game.keywords(goat).contains(Keywords::MENACE));
    let next = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, next, None);
    settle(&mut game);
    assert_eq!(game.observe(player).suspected, [exact]);
}

#[test]
fn fugitive_codebreaker_reduces_the_turn_up_cost_only_and_draws_after_discarding() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Fugitive Codebreaker", Zone::Hand);
    let first = put(&mut game, player, "Swamp", Zone::Hand);
    let second = put(&mut game, player, "Hill Giant", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Library);
    put(&mut game, player, "Opt", Zone::Graveyard);
    put(&mut game, player, "Divination", Zone::Graveyard);
    put(&mut game, player, "Hill Giant", Zone::Graveyard);
    put(&mut game, player.other(), "Shock", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut game, Action::AnnounceDisguise { card });
    game.state.players[player.index()].mana[5] = 3;
    let exact = cast_down(&mut game, card);
    let card = exact.card;
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
    turn_up(&mut game, exact, red(3, 1));
    assert!(game.keywords(card).contains(Keywords::HASTE));
    pass_pair(&mut game);
    for discard in [first, second] {
        act(&mut game, Action::Discard { card: discard });
    }
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    game.state.players[player.index()].mana[3] = 1;
    cast(&mut game, shock, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(stats(&game, card), (3, 2));
}

#[test]
fn goblin_maskmakers_stack_attack_discounts_on_face_down_spells_this_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Goblin Maskmaker", Zone::Battlefield);
    let second = put(&mut game, player, "Goblin Maskmaker", Zone::Battlefield);
    let disguised = put(&mut game, player, "Fugitive Codebreaker", Zone::Hand);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: first });
    act(&mut game, Action::Attack { card: second });
    act(&mut game, Action::FinishAttackers);
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    settle(&mut game);
    enter_step(&mut game, Step::Main2);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: disguised });
    let mut expired = game.clone();
    enter_step(&mut expired, Step::End);
    pass_pair(&mut expired);
    expired.state.active = player;
    expired.state.priority = player;
    enter_step(&mut expired, Step::Main1);
    expired.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut expired, Action::AnnounceDisguise { card: disguised });

    let exact = cast_down(&mut game, disguised);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
    reject(
        &mut game,
        Action::BeginFaceUp {
            object: exact,
            cost: red(3, 1),
        },
    );
}

#[test]
fn offender_at_large_can_boost_a_creature_on_normal_entry_or_when_turned_face_up() {
    let player = Player::P1;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Offender at Large", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let mut normal = game.clone();
    cast(&mut normal, card, None);
    pass_pair(&mut normal);
    target(&mut normal, ally);
    settle(&mut normal);
    assert_eq!(stats(&normal, ally), (4, 2));
    let exact = cast_down(&mut game, card);
    assert_eq!(stats(&game, ally), (2, 2));
    turn_up(&mut game, exact, red(4, 1));
    let mut no_target = game.clone();
    act(&mut no_target, Action::FinishTargets);
    settle(&mut no_target);
    assert_eq!(stats(&no_target, ally), (2, 2));
    target(&mut game, ally);
    settle(&mut game);
    assert_eq!(stats(&game, ally), (4, 2));
    enter_step(&mut game, Step::End);
    pass_pair(&mut game);
    assert_eq!(stats(&game, ally), (2, 2));
}

#[test]
fn pyrotechnic_performer_uses_the_turned_creature_as_damage_source() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let performer = put(&mut game, player, "Pyrotechnic Performer", Zone::Hand);
    let basher = put(&mut game, player, "Bolrac-Clan Basher", Zone::Hand);
    let expose = put(&mut game, opponent, "Expose the Culprit", Zone::Hand);
    let power = put(&mut game, player, "Ancestral Anger", Zone::Hand);
    let source = cast_down(&mut game, performer);
    turn_up(&mut game, source, red(0, 1));
    settle(&mut game);
    assert_eq!(life(&game, opponent), 17);
    let exact = cast_down(&mut game, basher);
    let basher = exact.card;
    cast(&mut game, power, Some(Target::Card(basher)));
    settle(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: expose });
    act(&mut game, Action::ChooseMode { mode: 0 });
    target(&mut game, basher);
    pay(&mut game);
    pass_pair(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    pass_pair(&mut game);
    assert!(!game.is_face_down(exact));
    assert_eq!(stats(&game, basher), (4, 2));
    settle(&mut game);
    assert_eq!(life(&game, opponent), 13);
    assert!(game.object_view(basher).dealt_damage_this_turn);
}
