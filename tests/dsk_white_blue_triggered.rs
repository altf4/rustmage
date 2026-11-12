//! Card-level scenarios for Duskmourn's white and blue triggered effects.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OPPONENT: Player = Player::P1;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn declare_attacks(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn end_turn(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(game);
    pass_pair(game);
}

fn next_own_turn(game: &mut Game) {
    end_turn(game);
    assert_eq!(game.state.active, OPPONENT);
    end_turn(game);
    assert_eq!(game.state.active, PLAYER);
}

fn begin_additional_cost(game: &mut Game, spell: CardId) {
    act(game, Action::AnnounceCast { card: spell });
    if game.legal_actions().contains(&Action::BeginOrderedCosts) {
        act(game, Action::BeginOrderedCosts);
    }
    if matches!(game.state.pending, Pending::CastCosts { .. }) {
        let additional = game
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional)
            .unwrap();
        act(
            game,
            Action::ChooseCastCost {
                clause: additional as u16,
            },
        );
    }
    assert!(matches!(game.state.pending, Pending::AdditionalCost { .. }));
}

fn cost_object(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn finish_cast(game: &mut Game) {
    for _ in 0..8 {
        match game.state.pending {
            Pending::Priority => return,
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(game),
            ref pending => panic!("unexpected cost choice {pending:?}"),
        }
    }
    panic!("cast did not finish");
}

#[test]
fn splitskin_doll_draws_first_and_checks_other_controlled_creatures_at_resolution() {
    let mut game = scenario(PLAYER);
    let doll = put(&mut game, PLAYER, "Splitskin Doll", Zone::Hand);
    let small = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    put(&mut game, PLAYER, "Hill Giant", Zone::Battlefield);
    put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let growth = put(&mut game, PLAYER, "Giant Growth", Zone::Hand);
    let top = put(&mut game, PLAYER, "Shock", Zone::Library);
    cast(&mut game, doll, None);
    pass_pair(&mut game);
    assert_eq!(stats(&game, doll), (2, 1));
    assert_eq!(game.object_view(top).zone, Zone::Library);

    let mut with_small_creature = game.clone();
    settle(&mut with_small_creature);
    assert_eq!(with_small_creature.object_view(top).zone, Zone::Hand);
    assert_eq!(with_small_creature.object_view(growth).zone, Zone::Hand);

    cast(&mut game, growth, Some(Target::Card(small)));
    pass_pair(&mut game);
    assert_eq!(stats(&game, small), (5, 5));
    pass_pair(&mut game);
    assert!(matches!(
        game.state.pending,
        Pending::EffectDiscard { player: PLAYER, .. }
    ));
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert!(
        game.legal_actions()
            .contains(&Action::Discard { card: top })
    );
    assert!(
        game.observe(OPPONENT)
            .objects
            .iter()
            .all(|object| object.id != top)
    );
    act(&mut game, Action::Discard { card: top });
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn prowler_can_only_help_another_attacker_and_the_block_restriction_expires() {
    let mut game = scenario(PLAYER);
    let prowler = put(&mut game, PLAYER, "Clammy Prowler", Zone::Battlefield);
    let attacker = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let nonattacker = put(&mut game, PLAYER, "Hill Giant", Zone::Battlefield);
    let blocker = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    declare_attacks(&mut game, &[prowler, attacker]);
    for illegal in [prowler, nonattacker, blocker] {
        reject(&mut game, target(illegal));
    }
    act(&mut game, target(attacker));
    settle(&mut game);
    assert!(
        game.object_view(attacker)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    pass_pair(&mut game);
    reject(&mut game, Action::Block { blocker, attacker });
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: prowler,
        },
    );
    act(&mut game, Action::FinishBlockers);
    end_turn(&mut game);
    assert!(
        !game
            .object_view(attacker)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
}

#[test]
fn abduction_links_both_exiles_and_returns_each_exact_card_to_its_owner() {
    let mut game = scenario(PLAYER);
    let fear = put(&mut game, PLAYER, "Fear of Abduction", Zone::Hand);
    let land = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    let payment = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut game, PLAYER, "Hill Giant", Zone::Battlefield);
    let bounce = put(&mut game, PLAYER, "Unsummon", Zone::Hand);
    game.change_controller(game.object_ref(victim), OPPONENT);
    reject(&mut game, Action::AnnounceCast { card: fear });
    game.change_controller(game.object_ref(payment), PLAYER);
    let own = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);

    begin_additional_cost(&mut game, fear);
    let land_cost = cost_object(&game, land);
    let opposing_cost = cost_object(&game, victim);
    reject(&mut game, land_cost);
    reject(&mut game, opposing_cost);
    let paid = cost_object(&game, payment);
    act(&mut game, paid);
    finish_cast(&mut game);
    assert_eq!(game.object_view(payment).zone, Zone::Exile);
    pass_pair(&mut game);
    assert_eq!(stats(&game, fear), (5, 5));
    assert!(game.object_view(fear).keywords.contains(Keywords::FLYING));
    reject(&mut game, target(own));
    act(&mut game, target(victim));
    let source = game.object_ref(fear);

    let mut early_departure = game.clone();
    cast(&mut early_departure, bounce, Some(Target::Card(fear)));
    settle(&mut early_departure);
    assert_eq!(early_departure.object_view(payment).zone, Zone::Hand);
    assert_eq!(early_departure.object_view(victim).zone, Zone::Exile);

    settle(&mut game);
    let linked: Vec<_> = game
        .observe(PLAYER)
        .linked_exile
        .iter()
        .filter_map(|row| (row.source == source).then_some(row.exiled))
        .collect();
    assert_eq!(linked, [game.object_ref(payment), game.object_ref(victim)]);
    cast(&mut game, bounce, Some(Target::Card(fear)));
    settle(&mut game);
    for (card, owner) in [(payment, OPPONENT), (victim, PLAYER)] {
        let view = game.object_view(card);
        assert_eq!((view.zone, view.owner), (Zone::Hand, owner));
    }
}

#[test]
fn isolation_returns_a_controlled_permanent_even_if_the_spell_is_countered() {
    let mut game = scenario(PLAYER);
    let fear = put(&mut game, PLAYER, "Fear of Isolation", Zone::Hand);
    let stolen = put(&mut game, OPPONENT, "Prophetic Prism", Zone::Battlefield);
    let enemy = put(&mut game, OPPONENT, "Hill Giant", Zone::Battlefield);
    let counter = put(&mut game, OPPONENT, "Essence Scatter", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: fear });
    game.change_controller(game.object_ref(stolen), PLAYER);
    let land = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    game.create_tokens(PLAYER, Card::from_catalogue_id(5669), 1, false);
    let token = game.controlled_token_ids(PLAYER).next().unwrap();
    begin_additional_cost(&mut game, fear);
    let opponent_cost = cost_object(&game, enemy);
    reject(&mut game, opponent_cost);

    for material in [land, token] {
        let mut branch = game.clone();
        let chosen = cost_object(&branch, material);
        act(&mut branch, chosen);
        finish_cast(&mut branch);
        settle(&mut branch);
        assert_ne!(branch.object_view(material).zone, Zone::Battlefield);
        assert_eq!(stats(&branch, fear), (2, 3));
        assert!(branch.object_view(fear).keywords.contains(Keywords::FLYING));
    }

    let chosen = cost_object(&game, stolen);
    act(&mut game, chosen);
    finish_cast(&mut game);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert_eq!(game.object_view(stolen).owner, OPPONENT);
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(fear)));
    settle(&mut game);
    assert_eq!(game.object_view(fear).zone, Zone::Graveyard);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
}

const GRIMOIRE: &str = "Marina Vendrell's Grimoire";

#[test]
fn grimoire_draws_on_cast_and_actual_life_gain_and_keeps_large_hands() {
    let mut game = scenario(PLAYER);
    let grimoire = put(&mut game, PLAYER, GRIMOIRE, Zone::Hand);
    let abuelo = put(
        &mut game,
        PLAYER,
        "Abuelo, Ancestral Echo",
        Zone::Battlefield,
    );
    let prince = put(&mut game, PLAYER, "Charming Prince", Zone::Hand);
    put(&mut game, PLAYER, "Forest", Zone::Hand);
    for _ in 0..4 {
        put(&mut game, PLAYER, "Island", Zone::Library);
    }
    cast(&mut game, grimoire, None);
    settle(&mut game);
    assert_eq!(game.hand_count(PLAYER), 7);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 5);

    let mut blink = game.clone();
    let original = blink.object_ref(grimoire);
    act(
        &mut blink,
        Action::AnnounceActivate {
            card: abuelo,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut blink, target(grimoire));
    pay(&mut blink);
    settle(&mut blink);
    assert_eq!(blink.object_view(grimoire).zone, Zone::Exile);
    blink.begin_step(Step::End);
    blink.prepare_triggers();
    settle(&mut blink);
    assert_eq!(blink.object_view(grimoire).zone, Zone::Battlefield);
    assert_ne!(blink.object_ref(grimoire), original);
    assert_eq!(blink.hand_count(PLAYER), 7);

    cast(&mut game, prince, None);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseMode { mode: 1 });
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, 23);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 8);
    assert_eq!(game.hand_count(PLAYER), 9);
    assert!(game.observe(PLAYER).players[PLAYER.index()].no_maximum_hand_size);
    end_turn(&mut game);
    assert_eq!(game.state.active, OPPONENT);
    assert_eq!(game.hand_count(PLAYER), 9);
}

#[test]
fn grimoire_discards_the_life_lost_and_preserves_only_the_life_total_exception() {
    let mut game = scenario(PLAYER);
    let grimoire = put(&mut game, PLAYER, GRIMOIRE, Zone::Battlefield);
    let removal = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    let shock = put(&mut game, OPPONENT, "Shock", Zone::Hand);
    let mut empty_after_loss = game.clone();
    act(&mut empty_after_loss, Action::Pass);
    cast(&mut empty_after_loss, shock, Some(Target::Player(PLAYER)));
    pass_pair(&mut empty_after_loss);
    pass_pair(&mut empty_after_loss);
    if matches!(
        empty_after_loss.state.pending,
        Pending::EffectDiscard { .. }
    ) {
        act(&mut empty_after_loss, Action::Discard { card: removal });
    }
    settle(&mut empty_after_loss);
    assert_eq!(empty_after_loss.hand_count(PLAYER), 0);
    assert_eq!(empty_after_loss.state.players[PLAYER.index()].life, 18);
    assert_eq!(empty_after_loss.outcome(), Some(Outcome::Winner(OPPONENT)));

    let first = put(&mut game, PLAYER, "Forest", Zone::Hand);
    let second = put(&mut game, PLAYER, "Island", Zone::Hand);
    game.state.players[PLAYER.index()].life = 1;
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Player(PLAYER)));
    pass_pair(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, -1);
    assert_eq!(game.outcome(), None);
    assert_eq!(
        game.state.stack.last().unwrap().life_event.unwrap().amount,
        2
    );
    pass_pair(&mut game);
    assert!(matches!(
        game.state.pending,
        Pending::EffectDiscard { player: PLAYER, .. }
    ));
    assert!(
        game.observe(OPPONENT)
            .objects
            .iter()
            .all(|object| object.id != first && object.id != second)
    );
    for card in [first, second] {
        act(&mut game, Action::Discard { card });
    }
    settle(&mut game);
    assert_eq!(game.hand_count(PLAYER), 1);
    assert_eq!(game.outcome(), None);

    cast(&mut game, removal, Some(Target::Card(grimoire)));
    settle(&mut game);
    assert_eq!(game.object_view(grimoire).zone, Zone::Graveyard);
    assert_eq!(game.outcome(), Some(Outcome::Winner(OPPONENT)));
}

fn hallcreeper_hits(game: &mut Game, attacker: CardId, blocker: CardId) {
    declare_attacks(game, &[attacker]);
    pass_pair(game);
    reject(game, Action::Block { blocker, attacker });
    act(game, Action::FinishBlockers);
    pass_pair(game);
}

#[test]
fn hallcreeper_spends_modes_at_announcement_and_copies_in_place_indefinitely() {
    let mut game = scenario(PLAYER);
    let hallcreeper = put(&mut game, PLAYER, "Silent Hallcreeper", Zone::Hand);
    let donor = put(&mut game, PLAYER, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    game.add_counter(donor, CounterKind::PlusOne, 2);
    cast(&mut game, hallcreeper, None);
    settle(&mut game);
    let source = game.object_ref(hallcreeper);
    next_own_turn(&mut game);

    hallcreeper_hits(&mut game, hallcreeper, enemy);
    act(&mut game, Action::ChooseMode { mode: 0 });
    settle(&mut game);
    assert_eq!(stats(&game, hallcreeper), (3, 3));
    assert_eq!(game.counter_count(hallcreeper, CounterKind::PlusOne), 2);

    next_own_turn(&mut game);
    hallcreeper_hits(&mut game, hallcreeper, enemy);
    reject(&mut game, Action::ChooseMode { mode: 0 });
    let hand = game.hand_count(PLAYER);
    act(&mut game, Action::ChooseMode { mode: 1 });
    settle(&mut game);
    assert_eq!(game.hand_count(PLAYER), hand + 1);

    next_own_turn(&mut game);
    hallcreeper_hits(&mut game, hallcreeper, enemy);
    reject(&mut game, Action::ChooseMode { mode: 1 });
    act(&mut game, Action::ChooseMode { mode: 2 });
    reject(&mut game, target(hallcreeper));
    reject(&mut game, target(enemy));
    act(&mut game, target(donor));

    let mut fizzled = game.clone();
    fizzled.move_object(donor, Zone::Graveyard);
    settle(&mut fizzled);
    assert_eq!(fizzled.object_view(hallcreeper).name, "Silent Hallcreeper");
    next_own_turn(&mut fizzled);
    hallcreeper_hits(&mut fizzled, hallcreeper, enemy);
    assert_eq!(fizzled.state.pending, Pending::Priority);
    assert!(fizzled.state.stack.is_empty());
    fizzled.move_object(hallcreeper, Zone::Hand);
    fizzled.move_object(hallcreeper, Zone::Battlefield);
    assert_ne!(fizzled.object_ref(hallcreeper), source);
    next_own_turn(&mut fizzled);
    hallcreeper_hits(&mut fizzled, hallcreeper, enemy);
    assert!(
        fizzled
            .legal_actions()
            .contains(&Action::ChooseMode { mode: 0 })
    );
    assert!(
        fizzled
            .legal_actions()
            .contains(&Action::ChooseMode { mode: 1 })
    );

    settle(&mut game);
    assert_eq!(game.object_ref(hallcreeper), source);
    assert_eq!(game.object_view(hallcreeper).name, "Hill Giant");
    assert_eq!(stats(&game, hallcreeper), (5, 5));
    assert!(
        !game
            .object_view(hallcreeper)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    game.move_object(donor, Zone::Graveyard);
    next_own_turn(&mut game);
    assert_eq!(game.object_ref(hallcreeper), source);
    assert_eq!(game.object_view(hallcreeper).name, "Hill Giant");
    assert_eq!(stats(&game, hallcreeper), (5, 5));
}

fn shock(game: &mut Game, player: Player, target: Target) {
    let spell = put(game, player, "Shock", Zone::Hand);
    cast(game, spell, Some(target));
    settle(game);
}

#[test]
fn mindskinner_prevents_only_its_controllers_damage_to_opponents_and_mills() {
    let mut game = scenario(PLAYER);
    let skin = put(&mut game, PLAYER, "The Mindskinner", Zone::Hand);
    let giant = put(&mut game, OPPONENT, "Hill Giant", Zone::Battlefield);
    cast(&mut game, skin, None);
    settle(&mut game);
    assert_eq!(stats(&game, skin), (10, 1));
    assert!(
        game.object_view(skin)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );

    let mut unpreventable = game.clone();
    put(
        &mut unpreventable,
        OPPONENT,
        "Sunspine Lynx",
        Zone::Battlefield,
    );
    shock(&mut unpreventable, PLAYER, Target::Player(OPPONENT));
    assert_eq!(unpreventable.state.players[OPPONENT.index()].life, 18);
    assert_eq!(unpreventable.library_card_count(OPPONENT), 6);

    shock(&mut game, PLAYER, Target::Player(OPPONENT));
    assert_eq!(game.state.players[OPPONENT.index()].life, 20);
    assert_eq!(game.library_card_count(OPPONENT), 6);
    assert_eq!(game.library_card_count(PLAYER), 8);

    shock(&mut game, PLAYER, Target::Card(giant));
    assert_eq!(game.object_view(giant).damage, 2);
    assert_eq!(game.library_card_count(OPPONENT), 6);
    act(&mut game, Action::Pass);
    shock(&mut game, OPPONENT, Target::Player(PLAYER));
    assert_eq!(game.state.players[PLAYER.index()].life, 18);
    assert_eq!(game.library_card_count(PLAYER), 8);
}

const TALE: &str = "The Tale of Tamiyo";

#[test]
fn tales_first_three_chapters_repeat_shared_card_types_and_stop_at_the_mismatch() {
    for chapter in 1..=3 {
        let mut game = scenario(PLAYER);
        let mismatch_two = put(&mut game, PLAYER, "Bear Cub", Zone::Library);
        let mismatch_one = put(&mut game, PLAYER, "Forest", Zone::Library);
        let drawn = put(&mut game, PLAYER, "Shock", Zone::Library);
        let kindred_two = put(&mut game, PLAYER, "Firdoch Core", Zone::Library);
        let kindred_one = put(&mut game, PLAYER, "Ashling's Command", Zone::Library);
        let tale = if chapter == 1 {
            let tale = put(&mut game, PLAYER, TALE, Zone::Hand);
            cast(&mut game, tale, None);
            tale
        } else {
            let tale = put(&mut game, PLAYER, TALE, Zone::Battlefield);
            game.state.objects[tale as usize].counters.lore = chapter - 1;
            game.begin_step(Step::Main1);
            game.prepare_triggers();
            tale
        };
        settle(&mut game);
        assert_eq!(game.object_view(tale).zone, Zone::Battlefield);
        assert_eq!(game.counter_count(tale, CounterKind::Lore), chapter);
        for milled in [kindred_one, kindred_two, mismatch_one, mismatch_two] {
            assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
        }
        assert_eq!(game.object_view(drawn).zone, Zone::Hand);
        assert_eq!(game.library_card_count(PLAYER), 8);
        assert_eq!(game.hand_count(PLAYER), 1);
    }
}

#[test]
fn tales_last_chapter_copies_exact_graveyard_targets_and_pays_for_each_cast() {
    let mut game = scenario(PLAYER);
    let tale = put(&mut game, PLAYER, TALE, Zone::Battlefield);
    let instant = put(&mut game, PLAYER, "Shock", Zone::Graveyard);
    let sorcery = put(&mut game, PLAYER, "Exsanguinate", Zone::Graveyard);
    let skipped = put(&mut game, PLAYER, "Opt", Zone::Graveyard);
    let foreign = put(&mut game, OPPONENT, "Opt", Zone::Graveyard);
    let creature = put(&mut game, PLAYER, "Bear Cub", Zone::Graveyard);
    let unrelated = put(
        &mut game,
        PLAYER,
        "Liliana, Dreadhorde General",
        Zone::Graveyard,
    );
    game.state.objects[tale as usize].counters.lore = 3;
    game.begin_step(Step::Main1);
    game.prepare_triggers();

    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    settle(&mut zero);
    assert_eq!(zero.object_view(tale).zone, Zone::Graveyard);
    assert_eq!(zero.object_view(instant).zone, Zone::Graveyard);

    for illegal in [foreign, creature, unrelated] {
        reject(&mut game, target(illegal));
    }
    for legal in [instant, sorcery, skipped] {
        act(&mut game, target(legal));
    }
    act(&mut game, Action::FinishTargets);
    pass_pair(&mut game);
    let copies = game
        .observe(PLAYER)
        .resolution_free_casts
        .unwrap()
        .remaining;
    assert_eq!(copies.len(), 3);
    let copy_of = |original| {
        let definition = game.object_view(original).card;
        *copies
            .iter()
            .find(|copy| game.object_view(copy.card).card == definition)
            .unwrap()
    };
    let instant_copy = copy_of(instant);
    let sorcery_copy = copy_of(sorcery);
    let skipped_copy = copy_of(skipped);
    let cast_copy = |object| Action::CastExiledDuringResolution {
        object,
        face: CastFace::Normal,
    };
    for original in [instant, sorcery, skipped] {
        assert_eq!(game.object_view(original).zone, Zone::Exile);
        let original_action = cast_copy(game.object_ref(original));
        reject(&mut game, original_action);
    }
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);

    let action = cast_copy(instant_copy);
    let saved = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(PLAYER, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed
            .observe(PLAYER)
            .resolution_free_casts
            .unwrap()
            .remaining,
        copies
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(OPPONENT),
        },
    );
    pay(&mut game);

    act(&mut game, cast_copy(sorcery_copy));
    reject(&mut game, Action::ChooseX { value: 100 });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().mana_value(), 4);
    assert_eq!(game.state.stack.len(), 2);
    assert_eq!(game.object_view(tale).zone, Zone::Battlefield);
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(tale).zone, Zone::Graveyard);
    assert_eq!(game.object_view(skipped_copy.card).zone, Zone::Outside);
    settle(&mut game);
    assert_eq!(game.state.players[OPPONENT.index()].life, 16);
}
