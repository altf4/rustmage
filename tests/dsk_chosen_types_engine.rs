//! A chosen layer-four subtype follows each recipient's rules zone and relationship.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TYPE_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ChosenCreatureSubtypeAcrossZones],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        opening_hand_battlefield: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ENTRY_COUNTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledSubtypeEntryCounters {
            subtype: "Dragon",
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Subtype("Dragon"),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};
const HAND_SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchHandAndOrLibrary(SearchFilter::Subtype(
            "Dragon",
        ))],
    ),
    ..CardRules::EMPTY
};
const FUEL: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 32,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 1,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn provider_definition() -> Card {
    named("Leyline of Hope")
}

fn entry_type(name: &str) -> EntryChoiceValue {
    EntryChoiceValue::CreatureType(CreatureType::from_name(name).unwrap())
}

fn choose_type(game: &mut Game, name: &str) {
    act(
        game,
        Action::ChooseEntryValue {
            value: entry_type(name),
        },
    );
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn placed_provider(game: &mut Game, player: Player, name: &str) -> CardId {
    let source = spawn(game, player, provider_definition(), Zone::Battlefield);
    game.set_entry_value(game.object_ref(source), entry_type(name));
    source
}

#[test]
fn opening_choice_applies_by_control_on_the_battlefield_and_ownership_elsewhere() {
    let definition = provider_definition();
    let _rules = fixture_rules::activate(&[(definition, &TYPE_PROVIDER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = Game::new(Config {
        decks: [
            [vec![definition], vec![Card::Forest; 6]].concat(),
            vec![Card::Forest; 7],
        ],
        seed: 20261023,
        starting_player: player,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    act(&mut game, Action::Keep);
    act(&mut game, Action::Keep);
    let opening = game
        .legal_actions()
        .into_iter()
        .find(|choice| matches!(choice, Action::BeginOnBattlefield { .. }))
        .unwrap();
    let Action::BeginOnBattlefield { object: source } = opening else {
        unreachable!()
    };
    act(&mut game, opening);
    reject(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Blue),
        },
    );
    let checkpoint = game.snapshot();
    choose_type(&mut game, "Dragon");
    act(&mut game, Action::FinishOpeningHand);
    act(&mut game, Action::FinishOpeningHand);
    assert_eq!(game.state.step, Step::Upkeep);
    assert_eq!(
        game.object_view(source.card).entry_choice,
        Some(entry_type("Dragon"))
    );
    let mut alternate = game.clone();
    alternate.restore(&checkpoint).unwrap();
    choose_type(&mut alternate, "Soldier");
    assert_eq!(
        alternate.object_view(source.card).entry_choice,
        Some(entry_type("Soldier"))
    );

    for zone in [
        Zone::Hand,
        Zone::Library,
        Zone::Graveyard,
        Zone::Exile,
        Zone::Command,
    ] {
        let ours = spawn(&mut game, player, Card::GrizzlyBears, zone);
        let theirs = spawn(&mut game, opponent, Card::GrizzlyBears, zone);
        assert!(game.has_subtype(ours, "Bear") && game.has_subtype(ours, "Dragon"));
        assert!(game.has_subtype(theirs, "Bear") && !game.has_subtype(theirs, "Dragon"));
    }
    let hidden = spawn(&mut game, player, Card::GrizzlyBears, Zone::Exile);
    let hidden_ref = game.object_ref(hidden);
    game.remember_face_down_exile(hidden_ref, 1 << player.index(), None);
    assert!(!game.has_subtype(hidden, "Dragon") && !game.has_subtype(hidden, "Bear"));
    let outside = spawn(&mut game, player, Card::GrizzlyBears, Zone::Outside);
    assert!(!game.has_subtype(outside, "Dragon"));
    let stolen = spawn(&mut game, opponent, Card::GrizzlyBears, Zone::Battlefield);
    let surrendered = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player, None);
    game.record_control(game.object_ref(surrendered), opponent, None);
    assert!(game.has_subtype(stolen, "Dragon"));
    assert!(!game.has_subtype(surrendered, "Dragon"));
}

#[test]
fn layer_four_grants_survive_ability_loss_and_respect_subtype_setter_order() {
    let _rules = fixture_rules::activate(&[(provider_definition(), &TYPE_PROVIDER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = placed_provider(&mut game, player, "Dragon");
    let bear = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let hand = spawn(&mut game, player, Card::GrizzlyBears, Zone::Hand);
    let original = game.object_ref(source);
    game.remove_abilities_exact(&[original], original, player, AnimationDuration::Permanent);
    assert!(!game.printed_abilities_available(source));
    assert!(game.has_subtype(bear, "Dragon"));
    assert!(game.has_subtype(hand, "Dragon"));
    assert!(game.has_subtype(changeling, "Angel") && game.has_subtype(changeling, "Dragon"));

    let artifact = put(&mut game, player, "Fountainport Bell", Zone::Battlefield);
    let animation = put(&mut game, player, "Zoetic Glyph", Zone::Hand);
    assert!(!game.has_subtype(artifact, "Dragon"));
    cast(&mut game, animation, Some(Target::Card(artifact)));
    settle(&mut game);
    assert!(game.has_subtype(artifact, "Golem") && game.has_subtype(artifact, "Dragon"));

    let setter = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, setter, Some(Target::Card(bear)));
    settle(&mut game);
    assert!(game.has_subtype(bear, "Citizen"));
    assert!(!game.has_subtype(bear, "Bear") && !game.has_subtype(bear, "Dragon"));
    game.move_object(source, Zone::Hand);
    assert!(!game.has_subtype(hand, "Dragon"));
    cast(&mut game, source, None);
    pass_pair(&mut game);
    choose_type(&mut game, "Soldier");
    assert_ne!(game.object_ref(source), original);
    assert!(game.has_subtype(bear, "Citizen") && game.has_subtype(bear, "Soldier"));
    assert!(game.has_subtype(hand, "Bear") && game.has_subtype(hand, "Soldier"));
    assert!(!game.has_subtype(hand, "Dragon"));
}

#[test]
fn a_chosen_type_supports_paid_casting_and_prospective_entry_replacements() {
    let _rules = fixture_rules::activate(&[
        (provider_definition(), &TYPE_PROVIDER),
        (Card::GrizzlyBears, &ENTRY_COUNTER),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = placed_provider(&mut game, player, "Dragon");
    spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    put(&mut game, player, "Firespitter Whelp", Zone::Battlefield);
    let reducer = put(&mut game, player, "Gathering Stone", Zone::Battlefield);
    let cavern = put(&mut game, player, "Cavern of Souls", Zone::Battlefield);
    for permanent in [reducer, cavern] {
        game.set_entry_value(game.object_ref(permanent), entry_type("Dragon"));
    }
    let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    let announce = Action::AnnounceCast { card: spell };
    let mut absent = game.clone();
    absent.move_object(source, Zone::Exile);
    reject(&mut absent, announce.clone());
    act(&mut game, announce);
    assert_eq!(game.mana_payment_domain().unwrap().generic, 0);
    choose(&mut game, |choice| {
        matches!(choice, Action::ActivateMana { card, ability: 1, production, .. }
            if *card == cavern && *production == [0, 0, 0, 0, 1, 0])
    });
    pay(&mut game);
    let spell_view = game
        .observe(opponent)
        .objects
        .into_iter()
        .find(|object| object.id == spell)
        .unwrap();
    assert_eq!(spell_view.zone, Zone::Stack);
    assert!(spell_view.has_subtype("Dragon") && spell_view.has_subtype("Bear"));
    let spell_id = game
        .state
        .stack
        .iter()
        .find(|item| item.kind == StackKind::Spell)
        .unwrap()
        .id;
    assert!(
        game.observe(player)
            .uncounterable_spell_ids
            .contains(&spell_id)
    );

    let mut departed = game.clone();
    departed.move_object(source, Zone::Exile);
    assert!(!departed.has_subtype(spell, "Dragon"));
    for (branch, counters) in [(&mut game, 1), (&mut departed, 0)] {
        settle(branch);
        assert_eq!(branch.counter_count(spell, CounterKind::PlusOne), counters);
        assert_eq!(branch.object_view(spell).zone, Zone::Battlefield);
        // The Dragon spell's cast trigger was already queued when the grant applied.
        assert_eq!(branch.state.players[opponent.index()].life, 19);
    }
}

#[test]
fn projected_payment_requires_the_subtype_provider_to_survive_mana_costs() {
    let fuel = named("Evolving Wilds");
    let _rules = fixture_rules::activate(&[(provider_definition(), &TYPE_PROVIDER), (fuel, &FUEL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = placed_provider(&mut game, player, "Dragon");
    let cavern = put(&mut game, player, "Cavern of Souls", Zone::Battlefield);
    game.set_entry_value(game.object_ref(cavern), entry_type("Dragon"));
    let fuel = spawn(&mut game, player, fuel, Zone::Battlefield);
    let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    let announce = Action::AnnounceCast { card: spell };
    reject(&mut game, announce.clone());
    let other = put(&mut game, player, "Roiling Dragonstorm", Zone::Battlefield);
    act(&mut game, announce);
    act(
        &mut game,
        Action::AnnounceMana {
            card: fuel,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        },
    );
    let provider = game.object_ref(provider);
    let other = game.object_ref(other);
    reject(&mut game, Action::ChooseAdditionalCost { object: provider });
    act(&mut game, Action::ChooseAdditionalCost { object: other });
    pay(&mut game);
    choose(&mut game, |choice| {
        matches!(choice, Action::ActivateMana { card, ability: 1, production, .. }
            if *card == cavern && *production == [0, 0, 0, 0, 1, 0])
    });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(other.card).zone, Zone::Graveyard);
    assert_eq!(game.object_view(provider.card).zone, Zone::Battlefield);
    assert!(game.has_subtype(spell, "Dragon"));
}

#[test]
fn an_exile_cast_uses_the_spells_controller_and_selected_face() {
    let _rules = fixture_rules::activate(&[(provider_definition(), &TYPE_PROVIDER)]);
    for caster in [Player::P0, Player::P1] {
        let controller = Player::P0;
        let mut game = scenario(caster);
        placed_provider(&mut game, controller, "Soldier");
        let card = put(&mut game, caster.other(), "Bear Cub", Zone::Exile);
        assert_eq!(
            game.has_subtype(card, "Soldier"),
            caster.other() == controller
        );
        game.grant_exile_play_permission(caster, game.object_ref(card), false);
        act(&mut game, Action::AnnounceExile { card });
        pay(&mut game);
        assert_eq!(game.object_view(card).controller, caster);
        assert_eq!(game.has_subtype(card, "Soldier"), caster == controller);
        settle(&mut game);
        assert_eq!(game.has_subtype(card, "Soldier"), caster == controller);
    }

    let player = Player::P0;
    let mut game = scenario(player);
    placed_provider(&mut game, player, "Soldier");
    let omen = put(&mut game, player, "Runescale Stormbrood", Zone::Hand);
    assert!(game.has_subtype(omen, "Soldier"));
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceOmen { card: omen });
    assert!(!game.has_subtype(omen, "Soldier"));
    choose(&mut game, |choice| {
        matches!(choice, Action::ChooseTarget { .. })
    });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(game.object_view(omen).zone, Zone::Library);
}

#[test]
fn private_searches_and_looks_find_only_the_correct_in_zone_characteristics() {
    let search = named("Opt");
    let hand_search = named("Divination");
    let _rules = fixture_rules::activate(&[
        (provider_definition(), &TYPE_PROVIDER),
        (search, &SEARCH),
        (hand_search, &HAND_SEARCH),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    placed_provider(&mut game, player, "Dragon");
    let sought = put(&mut game, player, "Bear Cub", Zone::Library);
    let noncreature = put(&mut game, player, "Island", Zone::Library);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Library);
    let spell = spawn(&mut game, player, search, Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    let view = game.observe(player).search_choice.unwrap();
    assert!(
        view.cards
            .iter()
            .any(|object| object.id == sought && object.has_subtype("Dragon"))
    );
    assert!(game.observe(opponent).search_choice.is_none());
    assert!(
        game.observe(opponent)
            .objects
            .iter()
            .all(|object| object.id != sought)
    );
    reject(&mut game, Action::ChooseSearch { card: foreign });
    reject(&mut game, Action::ChooseSearch { card: noncreature });
    act(&mut game, Action::ChooseSearch { card: sought });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(sought).zone, Zone::Hand);

    let spell = spawn(&mut game, player, hand_search, Zone::Hand);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseHandLibrarySearchZones {
            hand: true,
            library: false,
        },
    );
    assert!(
        game.observe(opponent)
            .hand_library_search
            .unwrap()
            .cards
            .is_none()
    );
    act(&mut game, Action::ChooseSearch { card: sought });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(sought).zone, Zone::Battlefield);

    let top = put(&mut game, player, "Hill Giant", Zone::Library);
    let dragonologist = put(&mut game, player, "Dragonologist", Zone::Hand);
    cast(&mut game, dragonologist, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player).library_selection.is_some());
    assert!(game.observe(opponent).library_selection.is_none());
    act(&mut game, Action::SelectLibrary { card: top });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
}
