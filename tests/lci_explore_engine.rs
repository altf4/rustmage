//! Exact explore ordering and owned scry/entry continuations.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
const EXPLORE: CardRules = CardRules {
    effect: Effect::Spell(CREATURE, &[Instruction::Explore(Subject::Targets)]),
    ..CardRules::EMPTY
};
const SCRY_FIRST: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ScryBeforeControlledCreatureExplores],
    }],
    ..CardRules::EMPTY
};
const REPEAT_X: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExploreRepeated {
            subject: Subject::Source,
            amount: AmountValue::ChosenX,
        }],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const RETURN_AND_EXPLORE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ReturnToBattlefieldThenExplore(
            AmountValue::Fixed(2),
        )],
    ),
    ..CardRules::EMPTY
};

fn resolve_until_choice(g: &mut Game) {
    for _ in 0..12 {
        if g.state.pending != Pending::Priority || g.state.stack.is_empty() {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("explore scenario exceeded its priority budget");
}

fn arrange(card: CardId, destination: LibraryDestination) -> Action {
    Action::ArrangeLibrary { card, destination }
}

fn revealed_choice(g: &Game, player: Player, card: CardId, graveyard: bool) -> Action {
    let choice = g.observe(player).explore_choice.expect("public reveal");
    assert_eq!(choice.revealed.object, g.object_ref(card));
    assert_eq!(
        g.observe(player.other()).explore_choice.as_ref(),
        Some(&choice)
    );
    Action::ChooseExplore {
        object: choice.revealed.object,
        graveyard,
    }
}

fn assert_private_scry(g: &Game, player: Player, card: CardId) {
    let observation = g.observe(player);
    let choice = observation.library_choice.expect("private scry");
    assert!(!choice.surveil);
    assert_eq!(choice.remaining.len(), 1);
    assert_eq!(choice.remaining[0].reference, g.object_ref(card));
    let opponent = g.observe(player.other());
    assert!(opponent.library_choice.is_none());
    assert!(opponent.known_library_top[player.index()].is_none());
    assert!(opponent.explore_choice.is_none());
    assert!(opponent.last_explore.is_none());
}

#[test]
fn separate_private_scries_precede_one_public_explore_across_owned_branches() {
    let provider = Card::from_name("Ghost Vacuum").unwrap();
    let _rules = fixture_rules::activate(&[(Card::Shock, &EXPLORE), (provider, &SCRY_FIRST)]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let creature = put(&mut root, player, "Grizzly Bears", Zone::Battlefield);
        put(&mut root, player, "Ghost Vacuum", Zone::Battlefield);
        put(&mut root, player, "Ghost Vacuum", Zone::Battlefield);
        put(&mut root, player.other(), "Ghost Vacuum", Zone::Battlefield);
        let second = put(&mut root, player, "Opt", Zone::Library);
        let top = put(&mut root, player, "Opt", Zone::Library);
        let spell = put(&mut root, player, "Shock", Zone::Hand);
        cast(&mut root, spell, Some(Target::Card(creature)));
        resolve_until_choice(&mut root);
        assert_private_scry(&root, player, top);
        assert_eq!(root.counter_count(creature, CounterKind::PlusOne), 0);
        reject(&mut root, arrange(second, LibraryDestination::Bottom));

        let saved = root.snapshot();
        let first = arrange(top, LibraryDestination::Bottom);
        let index = root
            .legal_actions()
            .iter()
            .position(|action| *action == first)
            .unwrap();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        let mut keep_top = root.clone();
        drop(root);
        act(&mut raw, first.clone());
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_private_scry(&raw, player, second);
        assert_eq!(raw.counter_count(creature, CounterKind::PlusOne), 0);

        act(&mut keep_top, arrange(top, LibraryDestination::Top));
        assert_private_scry(&keep_top, player, top);
        act(&mut keep_top, arrange(top, LibraryDestination::Top));
        let keep = revealed_choice(&keep_top, player, top, false);
        act(&mut keep_top, keep);

        for game in [&mut raw, &mut indexed] {
            act(game, arrange(second, LibraryDestination::Top));
            let mill = revealed_choice(game, player, second, true);
            assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
            act(game, mill);
            assert_eq!(game.object_view(second).zone, Zone::Graveyard);
            assert_eq!(game.observe(player).last_explore.unwrap().sequence, 1);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(keep_top.object_view(second).zone, Zone::Library);

        let completed = raw.snapshot();
        raw.restore(&saved).unwrap();
        assert_private_scry(&raw, player, top);
        act(&mut raw, first);
        act(&mut raw, arrange(second, LibraryDestination::Top));
        let mill = revealed_choice(&raw, player, second, true);
        act(&mut raw, mill);
        assert_eq!(raw.snapshot(), completed);
    }
}

fn cast_x(g: &mut Game, creature: CardId, value: u32) {
    act(g, Action::AnnounceCast { card: creature });
    act(g, Action::ChooseX { value });
    pay(g);
    resolve_until_choice(g);
}

#[test]
fn repeat_uses_paid_x_and_finishes_each_reveal_before_the_next() {
    let definition = Card::from_name("Wildwood Scourge").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &REPEAT_X)]);
    let player = Player::P0;
    for x in [0, 3] {
        let mut game = scenario(player);
        let nonland = put(&mut game, player, "Opt", Zone::Library);
        let land = put(&mut game, player, "Forest", Zone::Library);
        let creature = put(&mut game, player, "Wildwood Scourge", Zone::Hand);
        cast_x(&mut game, creature, x);
        if x == 0 {
            assert!(game.observe(player).last_explore.is_none());
            assert_eq!(game.object_view(land).zone, Zone::Library);
            assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);
            continue;
        }
        assert_eq!(game.object_view(land).zone, Zone::Hand);
        assert_eq!(game.observe(player).last_explore.unwrap().sequence, 1);
        let keep = revealed_choice(&game, player, nonland, false);
        act(&mut game, keep);
        assert_eq!(game.observe(player).last_explore.unwrap().sequence, 2);
        let mill = revealed_choice(&game, player, nonland, true);
        act(&mut game, mill);
        assert_eq!(game.observe(player).last_explore.unwrap().sequence, 3);
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
        assert_eq!(game.object_view(nonland).zone, Zone::Graveyard);
    }
}

#[test]
fn empty_library_scry_and_explore_complete_after_counter_replacement() {
    let definition = Card::from_name("Wildwood Scourge").unwrap();
    let provider = Card::from_name("Ghost Vacuum").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &REPEAT_X), (provider, &SCRY_FIRST)]);
    let player = Player::P1;
    let mut game = scenario(player);
    for card in game.state.players[player.index()].library.clone() {
        game.move_object(card, Zone::Exile);
    }
    put(&mut game, player, "Ghost Vacuum", Zone::Battlefield);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let creature = put(&mut game, player, "Wildwood Scourge", Zone::Hand);
    cast_x(&mut game, creature, 2);
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.state.players[player.index()].scried_or_surveilled_this_turn);
    let event = game.observe(player).last_explore.unwrap();
    assert_eq!(event.sequence, 2);
    assert_eq!(event.explorer.object, game.object_ref(creature));
    assert!(event.revealed.is_none());
    assert!(event.counter_added);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 4);
}

#[test]
fn reanimation_explores_only_the_successfully_returned_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &RETURN_AND_EXPLORE)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let origin = game.object_ref(creature);
        let nonland = put(&mut game, player, "Opt", Zone::Library);
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));

        let mut stale = game.clone();
        stale.move_object(creature, Zone::Exile);
        stale.move_object(creature, Zone::Graveyard);
        settle(&mut stale);
        assert!(stale.observe(player).last_explore.is_none());
        assert_eq!(stale.object_view(creature).zone, Zone::Graveyard);

        resolve_until_choice(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        let current = game.object_ref(creature);
        assert_ne!(origin, current);
        for _ in 0..2 {
            let action = revealed_choice(&game, player, nonland, false);
            assert_eq!(
                game.observe(player).explore_choice.unwrap().explorer.object,
                current
            );
            act(&mut game, action);
        }
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
        assert_eq!(game.observe(player).last_explore.unwrap().sequence, 2);
    }
}

#[test]
fn departed_explorer_uses_latest_controller_and_never_counters_a_new_incarnation() {
    let definition = Card::from_name("Wildwood Scourge").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &REPEAT_X)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let nonland = put(&mut game, other, "Opt", Zone::Library);
    let own_top = put(&mut game, player, "Forest", Zone::Library);
    let creature = put(&mut game, player, "Wildwood Scourge", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: creature });
    act(&mut game, Action::ChooseX { value: 1 });
    pay(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let origin = game.object_ref(creature);
    // Arrange a control change and departure with the ETB still on the stack.
    game.state.objects[creature as usize].controller = other;
    game.move_object(creature, Zone::Graveyard);
    game.move_object(creature, Zone::Battlefield);
    assert_ne!(game.object_ref(creature), origin);
    resolve_until_choice(&mut game);
    let choice = game.observe(other).explore_choice.unwrap();
    assert_eq!(choice.explorer.object, origin);
    assert_eq!(choice.explorer.controller, other);
    assert!(!choice.counter_added);
    assert_eq!(game.actor(), Some(other));
    let keep = revealed_choice(&game, other, nonland, false);
    act(&mut game, keep);
    assert_eq!(game.object_view(own_top).zone, Zone::Library);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);
}
