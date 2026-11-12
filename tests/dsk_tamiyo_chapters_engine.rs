//! Exact repeated mill receipts and paid card copies within a Saga chapter.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const MILL_CHAPTER: &[Instruction] = &[
    Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
    Instruction::DrawAndRepeatMillingIfSharedType,
];

const fn own_graveyard(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}

const CHAPTERS: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: MILL_CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: MILL_CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: MILL_CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Union(&[
                    own_graveyard(4 | 8),
                    TargetRule::Filtered {
                        base: &own_graveyard(64),
                        predicate: &TargetPredicate::Subtype("Jace"),
                    },
                ]),
                min: 0,
                max: usize::MAX,
            }),
            program: &[Instruction::ExileTargetsCopyCastPaid],
        },
    ],
    ..CardRules::EMPTY
};

const ZERO_ALTERNATIVE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ExileCastForZeroOncePerTurn],
    }],
    ..CardRules::EMPTY
};

const SAGA: &str = "Maximum Carnage";
const ZERO_PROVIDER: &str = "Fountainport Bell";

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn chapter(game: &mut Game, player: Player, number: u64) -> CardId {
    let saga = put(game, player, SAGA, Zone::Battlefield);
    game.state.objects[saga as usize].counters.lore = number - 1;
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    assert_eq!(game.counter_count(saga, CounterKind::Lore), number);
    saga
}

fn library(game: &mut Game, player: Player, top_first: &[&str]) -> Vec<CardId> {
    while let Some(id) = game.state.players[player.index()].library.last().copied() {
        game.move_object(id, Zone::Exile);
    }
    let mut ids: Vec<_> = top_first
        .iter()
        .rev()
        .map(|name| put(game, player, name, Zone::Library))
        .collect();
    ids.reverse();
    ids
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn cast_copy(object: ObjectRef) -> Action {
    Action::CastExiledDuringResolution {
        object,
        face: CastFace::Normal,
    }
}

fn choose_targets(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        act(game, target(card));
    }
    act(game, Action::FinishTargets);
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn copies(game: &Game, player: Player) -> Vec<ObjectRef> {
    game.observe(player)
        .resolution_free_casts
        .unwrap()
        .remaining
}

fn copy_of(game: &Game, copies: &[ObjectRef], original: CardId) -> ObjectRef {
    let definition = game.object_view(original).card;
    *copies
        .iter()
        .find(|object| game.object_view(object.card).card == definition)
        .unwrap()
}

#[test]
fn all_three_chapters_repeat_for_kindred_and_stop_on_the_next_mismatch() {
    let _rules = fixture_rules::activate(&[(card(SAGA), &CHAPTERS)]);
    let player = Player::P0;
    for number in 1..=3 {
        let mut game = scenario(player);
        let ids = library(
            &mut game,
            player,
            &[
                "Ashling's Command",
                "Firdoch Core",
                "Shock",
                "Forest",
                "Grizzly Bears",
            ],
        );
        let saga = chapter(&mut game, player, number);
        settle(&mut game);
        for index in [0, 1, 3, 4] {
            assert_eq!(game.object_view(ids[index]).zone, Zone::Graveyard);
        }
        assert_eq!(game.object_view(ids[2]).zone, Zone::Hand);
        assert_eq!(game.hand_count(player), 1);
        assert_eq!(game.object_view(saga).zone, Zone::Battlefield);
        assert!(game.outcome().is_none());
    }
}

#[test]
fn actual_public_mills_count_across_replacements_and_hidden_moves_do_not() {
    let _rules = fixture_rules::activate(&[(card(SAGA), &CHAPTERS)]);
    let player = Player::P1;

    let mut public = scenario(player);
    put(
        &mut public,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let ids = library(
        &mut public,
        player,
        &["Island", "Forest", "Shock", "Mountain"],
    );
    chapter(&mut public, player, 1);
    settle(&mut public);
    assert_eq!(public.object_view(ids[2]).zone, Zone::Hand);
    for index in [0, 1, 3] {
        assert_eq!(public.object_view(ids[index]).zone, Zone::Exile);
    }

    let mut hidden = scenario(player);
    let ids = library(
        &mut hidden,
        player,
        &["Darksteel Colossus", "Fountainport Bell"],
    );
    chapter(&mut hidden, player, 1);
    settle(&mut hidden);
    assert_eq!(hidden.object_view(ids[0]).zone, Zone::Library);
    assert_eq!(hidden.object_view(ids[1]).zone, Zone::Graveyard);
    assert_eq!(hidden.hand_count(player), 0);
    assert!(hidden.outcome().is_none());

    let mut larger = scenario(player);
    put(
        &mut larger,
        player.other(),
        "The Water Crystal",
        Zone::Battlefield,
    );
    let ids = library(
        &mut larger,
        player,
        &[
            "Forest",
            "Shock",
            "Grizzly Bears",
            "Fountainport Bell",
            "Rest in Peace",
            "Island",
            "Opt",
            "Divination",
        ],
    );
    chapter(&mut larger, player, 1);
    settle(&mut larger);
    assert_eq!(larger.object_view(ids[6]).zone, Zone::Hand);
    assert_eq!(larger.object_view(ids[7]).zone, Zone::Graveyard);
    assert_eq!(larger.hand_count(player), 1);

    let mut empty_draw = scenario(player);
    library(&mut empty_draw, player, &["Forest", "Island"]);
    chapter(&mut empty_draw, player, 1);
    settle(&mut empty_draw);
    assert_eq!(empty_draw.outcome(), Some(Outcome::Winner(player.other())));
}

#[test]
fn the_repeated_mill_waits_for_a_private_draw_choice_and_resumes_exactly_once() {
    let _rules = fixture_rules::activate(&[(card(SAGA), &CHAPTERS)]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Lorehold, the Historian",
        Zone::Battlefield,
    );
    let ids = library(
        &mut game,
        player,
        &["Island", "Mountain", "Shock", "Forest", "Grizzly Bears"],
    );
    chapter(&mut game, player, 2);
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::MiracleReveal { .. }));
    assert!(game.observe(player).miracle.is_some());
    assert!(game.observe(player.other()).miracle.is_none());
    assert!(
        !game
            .observe(player.other())
            .objects
            .iter()
            .any(|object| object.id == ids[2])
    );
    assert_eq!(game.object_view(ids[3]).zone, Zone::Library);
    reject(&mut game, Action::FinishResolutionCasts);

    let snapshot = game.snapshot();
    let action = Action::DeclineMiracle;
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(indexed.snapshot(), game.snapshot());
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.object_view(ids[3]).zone, Zone::Graveyard);
    assert_eq!(game.object_view(ids[4]).zone, Zone::Graveyard);

    indexed.restore(&snapshot).unwrap();
    assert_eq!(indexed.object_view(ids[3]).zone, Zone::Library);
    act(&mut indexed, Action::DeclineMiracle);
    settle(&mut indexed);
    assert_eq!(indexed.snapshot(), game.snapshot());
}

#[test]
fn final_chapter_casts_any_number_in_any_order_and_pays_real_costs() {
    let _rules = fixture_rules::activate(&[
        (card(SAGA), &CHAPTERS),
        (card(ZERO_PROVIDER), &ZERO_ALTERNATIVE),
    ]);
    let player = Player::P0;
    let enemy = player.other();
    let mut game = scenario(player);
    let x_spell = put(&mut game, player, "Exsanguinate", Zone::Graveyard);
    let instant = put(&mut game, player, "Candy Grapple", Zone::Graveyard);
    let walker = put(&mut game, player, "Jace, Reality Sculptor", Zone::Graveyard);
    let uncast = put(&mut game, player, "Opt", Zone::Graveyard);
    let foreign = put(&mut game, enemy, "Opt", Zone::Graveyard);
    let unrelated_walker = put(
        &mut game,
        player,
        "Liliana, Dreadhorde General",
        Zone::Graveyard,
    );
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let hidden = put(&mut game, player, "Island", Zone::Hand);
    let provider = put(&mut game, player, ZERO_PROVIDER, Zone::Battlefield);
    let bargain = put(&mut game, player, "Prophetic Prism", Zone::Battlefield);
    let victim = put(&mut game, enemy, "Hill Giant", Zone::Battlefield);
    game.add_counter(victim, CounterKind::PlusOne, 2);
    let saga = chapter(&mut game, player, 4);
    game.state.players[player.index()].mana = [0, 2, 2, 0, 0, 6];
    for illegal in [foreign, unrelated_walker, creature] {
        reject(&mut game, target(illegal));
    }
    choose_targets(&mut game, &[x_spell, instant, uncast, walker]);
    pass_pair(&mut game);
    let cohort = copies(&game, player);
    assert_eq!(cohort.len(), 4);
    for exiled in [x_spell, instant, walker, uncast] {
        assert_eq!(game.object_view(exiled).zone, Zone::Exile);
        assert!(cohort.iter().all(|copy| copy.card != exiled));
        let exact = game.object_ref(exiled);
        reject(&mut game, cast_copy(exact));
    }
    reject(&mut game, Action::Pass);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
    assert!(game.observe(enemy).resolving.is_some());
    assert!(
        !game
            .observe(enemy)
            .objects
            .iter()
            .any(|object| object.id == hidden)
    );
    assert_eq!(game.object_view(saga).zone, Zone::Battlefield);

    let declined_from = game.snapshot();
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(saga).zone, Zone::Graveyard);
    assert!(
        cohort
            .iter()
            .all(|object| declined.object_view(object.card).zone == Zone::Outside)
    );
    assert!(declined.observe(player).stack.is_empty());
    declined.restore(&declined_from).unwrap();
    assert_eq!(copies(&declined, player), cohort);

    let permanent_copy = copy_of(&game, &cohort, walker);
    let action = cast_copy(permanent_copy);
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    declined
        .apply_index(player, declined.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(declined.snapshot(), game.snapshot());
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 2, 0, 0, 0, 3],
        },
    );

    let x_copy = copy_of(&game, &cohort, x_spell);
    act(&mut game, cast_copy(x_copy));
    reject(&mut game, Action::ChooseX { value: 5 });
    act(&mut game, Action::ChooseX { value: 2 });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 2, 0, 0, 2],
        },
    );
    let x_item = game.observe(player).stack.last().unwrap().clone();
    assert_eq!(x_item.x, Some(2));
    assert_eq!(x_item.cast.unwrap().method, CastMethod::Normal);
    assert_eq!(x_item.mana_value(), 4);

    let instant_copy = copy_of(&game, &cohort, instant);
    let provider = game.object_ref(provider);
    act(
        &mut game,
        Action::AnnounceExileForZero {
            object: instant_copy,
            provider,
            face: CastFace::Normal,
            flash_surcharge: false,
            origin: ExileZeroOrigin::PaidResolution,
        },
    );
    let sacrifice = Some(game.object_ref(bargain));
    act(&mut game, Action::ChooseBargain { sacrifice });
    act(&mut game, target(victim));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        act(&mut game, Action::PayCosts { payment: [0; 6] });
    }
    assert_eq!(game.object_view(bargain).zone, Zone::Graveyard);
    assert_eq!(game.observe(player).stack.len(), 3);
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 3);
    assert_eq!(game.object_view(saga).zone, Zone::Battlefield);
    let unused = copy_of(&game, &cohort, uncast);
    assert_eq!(copies(&game, player), [unused]);

    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
    assert_eq!(game.object_view(unused.card).zone, Zone::Outside);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[enemy.index()].life, 18);
    assert!(game.object_view(permanent_copy.card).is_token());
    assert_eq!(
        game.counter_count(permanent_copy.card, CounterKind::Loyalty),
        5
    );
    for original in [x_spell, instant, walker, uncast] {
        assert_eq!(game.object_view(original).zone, Zone::Exile);
    }
}

#[test]
fn final_chapter_copies_only_actual_exiles_and_may_choose_no_targets() {
    struct ClearPrevention;
    impl Drop for ClearPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
        }
    }
    let _clear = ClearPrevention;
    let _rules = fixture_rules::activate(&[(card(SAGA), &CHAPTERS)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let stale = put(&mut game, player, "Divination", Zone::Graveyard);
    let prevented = put(&mut game, player, "Shock", Zone::Graveyard);
    let actual = put(&mut game, player, "Opt", Zone::Graveyard);
    let saga = chapter(&mut game, player, 4);

    let mut none = game.clone();
    choose_targets(&mut none, &[]);
    settle(&mut none);
    assert_eq!(none.object_view(saga).zone, Zone::Graveyard);
    assert!(none.observe(player).resolution_free_casts.is_none());
    assert_eq!(none.object_view(actual).zone, Zone::Graveyard);

    choose_targets(&mut game, &[stale, prevented, actual]);
    let before = game.object_ref(stale);
    game.move_object(stale, Zone::Exile);
    game.move_object(stale, Zone::Graveyard);
    assert_ne!(game.object_ref(stale), before);
    zone_replacements::TEST_PREVENT_ZONE_MOVE
        .with(|slot| slot.set(Some(game.object_ref(prevented))));
    pass_pair(&mut game);
    assert_eq!(game.object_view(stale).zone, Zone::Graveyard);
    assert_eq!(game.object_view(prevented).zone, Zone::Graveyard);
    assert_eq!(game.object_view(actual).zone, Zone::Exile);
    let cohort = copies(&game, player);
    assert_eq!(cohort.len(), 1);
    assert_eq!(game.object_view(cohort[0].card).card, card("Opt"));
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(cohort[0].card).zone, Zone::Outside);
}
