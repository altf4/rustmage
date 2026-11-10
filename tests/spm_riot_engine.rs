//! Riot is an entry replacement, including for abilities granted in layer six.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const RIOT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::RIOT,
    ..CardRules::EMPTY
};
const PROVIDER: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::GrantRiotToOtherControlledSubtype("Spider"),
            StaticAbilityFragment::GrantRiotToOtherControlledSubtype("Drake"),
        ],
    }],
    ..RIOT
};
const COPIER: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const COUNTER_PROHIBITION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};

const fn tap_mana(sacrifice: bool) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            sacrifice,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed(GREEN))],
        sorcery: false,
        once_per_turn: false,
    }
}
const MANA_SOURCE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[tap_mana(true)],
    ..CardRules::EMPTY
};
const RIOT_MANA: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[tap_mana(false)],
    ..RIOT
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn riot(game: &Game, player: Player, entrant: ObjectRef, remaining: usize) {
    assert_eq!(
        game.observe(player).entry_riot,
        Some(EntryRiotChoiceView { entrant, remaining })
    );
}

fn has_haste(game: &Game, id: CardId) -> bool {
    game.object_view(id).keywords.contains(Keywords::HASTE)
}

#[test]
fn printed_and_two_granted_instances_are_independent_and_owned() {
    let _rules = fixture_rules::activate(&[
        (card("Treetop Snarespinner"), &PROVIDER),
        (card("Spinner of Souls"), &RIOT),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
        put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        let id = put(&mut game, player, "Spinner of Souls", Zone::Hand);
        cast(&mut game, id, None);
        resolve_top(&mut game);
        let entrant = game.object_ref(id);
        riot(&game, player, entrant, 3);
        assert_eq!(game.object_view(id).zone, Zone::Stack);
        let counter = Action::ChooseEntryRiot { counter: true };
        let haste = Action::ChooseEntryRiot { counter: false };
        act(&mut game, counter.clone());
        riot(&game, player, entrant, 2);

        let checkpoint = game.snapshot();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == haste)
            .unwrap();
        let mut indexed = game.clone();
        let mut raw = game;
        assert_eq!(
            raw.apply(player.other(), haste.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(raw.snapshot(), checkpoint);
        act(&mut raw, haste.clone());
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        riot(&raw, player, entrant, 1);

        act(&mut raw, counter.clone());
        assert_ne!(raw.object_ref(id), entrant);
        assert!(has_haste(&raw, id));
        assert_eq!(raw.counter_count(id, CounterKind::PlusOne), 4);
        assert_eq!(
            raw.observe(player)
                .keyword_effects
                .iter()
                .filter(|row| row.recipient == raw.object_ref(id)
                    && row.origin == KeywordEffectOrigin::Riot)
                .count(),
            1
        );
        reject(&mut raw, haste);

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, counter.clone());
        act(&mut indexed, counter);
        assert!(!has_haste(&indexed, id));
        assert_eq!(indexed.counter_count(id, CounterKind::PlusOne), 6);
    }
}

#[test]
fn grant_respects_ability_removal_and_riot_haste_survives_cleanup_and_control_change() {
    let _rules = fixture_rules::activate(&[
        (card("Treetop Snarespinner"), &PROVIDER),
        (card("Spinner of Souls"), &RIOT),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
    let blank_provider = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, blank_provider, Some(Target::Card(provider)));
    settle(&mut game);
    assert!(!game.object_view(provider).keywords.contains(Keywords::RIOT));

    let id = put(&mut game, player, "Spinner of Souls", Zone::Hand);
    cast(&mut game, id, None);
    resolve_top(&mut game);
    riot(&game, player, game.object_ref(id), 1);
    act(&mut game, Action::ChooseEntryRiot { counter: false });
    let permanent = game.object_ref(id);
    assert!(has_haste(&game, id));

    let blank = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(id)));
    settle(&mut game);
    assert!(!has_haste(&game, id));
    let destroy = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, destroy, Some(Target::Card(blank)));
    settle(&mut game);
    assert!(has_haste(&game, id));

    game.change_controller(permanent, player.other());
    game.finish_cleanup();
    assert_eq!(game.object_ref(id), permanent);
    assert!(has_haste(&game, id));
    let row = game
        .observe(player)
        .keyword_effects
        .into_iter()
        .find(|row| row.recipient == permanent && row.origin == KeywordEffectOrigin::Riot)
        .unwrap();
    assert!(row.timestamp > 0);
    assert_eq!(row.expires_turn, None);

    game.move_object(id, Zone::Hand);
    assert_ne!(game.object_ref(id), permanent);
    assert!(!has_haste(&game, id));
    assert!(
        !game
            .observe(player)
            .keyword_effects
            .iter()
            .any(|row| row.recipient == permanent)
    );
}

#[test]
fn coentrants_do_not_grant_riot_until_after_entry_but_tokens_are_eligible() {
    let _rules = fixture_rules::activate(&[(card("Treetop Snarespinner"), &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Treetop Snarespinner", Zone::Graveyard);
    let sibling = put(&mut game, player, "Spinner of Souls", Zone::Graveyard);
    let opponent = put(
        &mut game,
        player.other(),
        "Spinner of Souls",
        Zone::Battlefield,
    );
    let moves = [sibling, provider]
        .into_iter()
        .map(|id| game.entry_move(id, Zone::Battlefield, false))
        .collect();
    game.stage_entries(moves, None);
    assert!(game.advance_entries());
    riot(&game, player, game.object_ref(provider), 1);
    act(&mut game, Action::ChooseEntryRiot { counter: true });
    assert!(game.observe(player).entry_riot.is_none());
    assert_eq!(game.counter_count(provider, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(sibling, CounterKind::PlusOne), 0);
    assert!(!has_haste(&game, sibling));
    assert!(game.object_view(sibling).keywords.contains(Keywords::RIOT));
    assert!(!game.object_view(opponent).keywords.contains(Keywords::RIOT));

    let tokens = game.create_token_batch_capture(player, Card::DrakeToken, 2, false, None, true);
    assert!(game.advance_entries());
    riot(&game, player, game.object_ref(tokens[0].card), 1);
    act(&mut game, Action::ChooseEntryRiot { counter: false });
    assert_eq!(game.object_view(tokens[0].card).zone, Zone::Outside);
    riot(&game, player, game.object_ref(tokens[1].card), 1);
    act(&mut game, Action::ChooseEntryRiot { counter: true });
    assert_eq!(game.object_ref(tokens[0].card), tokens[0]);
    assert!(has_haste(&game, tokens[0].card));
    assert_eq!(game.counter_count(tokens[1].card, CounterKind::PlusOne), 1);
}

#[test]
fn copy_replacement_precedes_riot_and_counter_prohibition_forces_haste() {
    let _rules = fixture_rules::activate(&[
        (card("Treetop Snarespinner"), &PROVIDER),
        (card("Spinner of Souls"), &RIOT),
        (card("Hill Giant"), &COPIER),
        (card("Short Bow"), &COUNTER_PROHIBITION),
    ]);
    let player = Player::P0;
    for prohibited in [false, true] {
        let mut game = scenario(player);
        put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
        let original = put(&mut game, player, "Spinner of Souls", Zone::Battlefield);
        if prohibited {
            put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
        }
        let copier = put(&mut game, player, "Hill Giant", Zone::Hand);
        cast(&mut game, copier, None);
        resolve_top(&mut game);
        assert!(game.observe(player).entry_riot.is_none());
        let original = game.object_ref(original);
        act(
            &mut game,
            Action::ChooseEntryCopy {
                object: Some(original),
            },
        );
        if !prohibited {
            riot(&game, player, game.object_ref(copier), 2);
            act(&mut game, Action::ChooseEntryRiot { counter: true });
            act(&mut game, Action::ChooseEntryRiot { counter: false });
        }
        assert!(game.observe(player).entry_riot.is_none());
        assert_eq!(game.object_view(copier).zone, Zone::Battlefield);
        assert_eq!(game.object_view(copier).card, card("Spinner of Souls"));
        assert!(has_haste(&game, copier));
        assert_eq!(
            game.counter_count(copier, CounterKind::PlusOne),
            u64::from(!prohibited)
        );
    }
}

#[test]
fn private_riot_entry_preserves_hidden_origin_and_funding_uses_only_haste() {
    let _rules = fixture_rules::activate(&[
        (card("Llanowar Elves"), &RIOT_MANA),
        (card("Short Bow"), &MANA_SOURCE),
    ]);
    let player = Player::P0;
    let mut hidden = scenario(player);
    let elf = put(&mut hidden, player, "Llanowar Elves", Zone::Hand);
    let movement = hidden.entry_move(elf, Zone::Battlefield, false);
    hidden.stage_entries(vec![movement], None);
    assert!(hidden.advance_entries());
    riot(&hidden, player, hidden.object_ref(elf), 1);
    let opponent = hidden.observe(player.other());
    assert!(opponent.entry_riot.is_none());
    assert_eq!(opponent.decision, "entry_choice");

    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let elf = put(&mut game, player, "Llanowar Elves", Zone::Exile);
    let mana = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    reject(&mut game, cast.clone());
    let exiled = game.object_ref(elf);
    game.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: game.object_ref(mana),
        exiled,
        return_zone: Zone::Battlefield,
    });
    act(&mut game, cast);
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == mana),
    );
    riot(&game, player, exiled, 1);
    assert_eq!(game.object_view(mana).zone, Zone::Graveyard);
    assert_eq!(game.object_view(elf).zone, Zone::Exile);
    reject(&mut game, Action::ChooseEntryRiot { counter: true });
    act(&mut game, Action::ChooseEntryRiot { counter: false });
    assert_eq!(game.object_view(elf).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(elf), exiled);
    assert!(has_haste(&game, elf));
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, .. } if *card == elf),
    );
    pay(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Stack);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
