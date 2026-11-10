//! Real actions through shared EOE resolution, land-play, and free-cast adapters.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("turn did not end");
}

const SURVEIL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::SurveilScaled(AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            }))),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn scaled_surveil_keeps_the_look_private_and_resumes_the_draw() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &SURVEIL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    put(&mut game, player, "Chrome Companion", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Nutrient Block",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let kept = put(&mut game, player, "Island", Zone::Library);
    let milled = put(&mut game, player, "Plains", Zone::Library);
    cast(&mut game, spell, None);
    resolve_top(&mut game);
    assert!(game.observe(player).library_choice.is_some());
    assert!(game.observe(player.other()).library_choice.is_none());
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| matches!(action, Action::ArrangeLibrary { .. }))
            .count(),
        4,
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: milled,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert_eq!(game.object_view(kept).zone, Zone::Hand);
}

const PARITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ChooseManaValueParityDestroyCreatures,
            Instruction::TakeCreaturesUntilEndTurn,
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn parity_destruction_then_untap_and_haste_uses_the_surviving_group() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &PARITY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let surviving = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let destroyed = put(
        &mut game,
        player.other(),
        "Savannah Lions",
        Zone::Battlefield,
    );
    let artifact = put(
        &mut game,
        player.other(),
        "Nutrient Block",
        Zone::Battlefield,
    );
    for id in [own, surviving, destroyed, artifact] {
        game.state.objects[id as usize].tapped = true;
    }
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    resolve_top(&mut game);
    act(
        &mut game,
        Action::ChooseManaValueParity {
            parity: ManaValueParity::Odd,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(destroyed).zone, Zone::Graveyard);
    assert_eq!(game.object_view(artifact).controller, player.other());
    assert!(game.object_view(artifact).tapped);
    for id in [own, surviving] {
        let object = game.object_view(id);
        assert_eq!(object.controller, player);
        assert!(!object.tapped);
        assert!(object.keywords.contains(Keywords::HASTE));
    }
    finish_turn(&mut game);
    assert_eq!(game.object_view(surviving).controller, player.other());
    assert!(
        !game
            .object_view(surviving)
            .keywords
            .contains(Keywords::HASTE)
    );
}

const LAND_OR_SPELL: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCastOrLandPlayed,
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SetControllerLifeToHalfStarting],
        },
    ],
    ..CardRules::EMPTY
};

const RETURN_LAND: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};

#[test]
fn special_action_land_play_is_distinct_from_putting_a_land_onto_the_battlefield() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Nutrient Block").unwrap(), &LAND_OR_SPELL),
        (Card::Divination, &RETURN_LAND),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    let played = put(&mut game, player, "Breeding Pool", Zone::Hand);
    let returned = put(&mut game, player, "Forest", Zone::Graveyard);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let initial = game.state.players[player.index()].library.len();
    act(&mut game, Action::PlayLand { card: played });
    assert_eq!(game.state.players[player.index()].library.len(), initial);
    act(&mut game, Action::ChooseEntryLife { pay: false });
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        initial - 1
    );
    assert!(game.object_view(played).tapped);
    cast(&mut game, spell, Some(Target::Card(returned)));
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        initial - 2
    );
    assert_eq!(game.object_view(returned).zone, Zone::Battlefield);

    let mut low = game.clone();
    low.state.players[player.index()].life = 4;
    finish_turn(&mut game);
    finish_turn(&mut low);
    for game in [&game, &low] {
        assert_eq!(game.state.players[player.index()].life, 10);
    }
}

const FREE_FIRST: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::ShuffleHandAndGraveyardThenDraw(7)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FreeFirstSpellOnOwnTurn],
    }],
    ..CardRules::EMPTY
};

#[test]
fn first_spell_alternative_is_symmetric_and_consumed_by_a_committed_cast() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Phyrexian Arena").unwrap(), &FREE_FIRST)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player.other(),
        "Phyrexian Arena",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let second = put(&mut game, player, "Hill Giant", Zone::Hand);
    let counter = put(&mut game, player.other(), "Cancel", Zone::Hand);
    let free = |card| Action::AnnounceFree {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    };

    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, free(first));
    pay(&mut game);
    act(&mut game, Action::Pass);
    reject(&mut game, free(counter));
    cast(&mut game, counter, Some(Target::Spell(first)));
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    reject(&mut game, free(second));
}

#[test]
fn cast_entry_shuffles_only_its_controllers_hand_and_graveyard() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Phyrexian Arena").unwrap(), &FREE_FIRST)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Phyrexian Arena", Zone::Hand);
    let hand = put(&mut game, player, "Island", Zone::Hand);
    let grave = put(&mut game, player, "Plains", Zone::Graveyard);
    let opposing_hand = put(&mut game, player.other(), "Island", Zone::Hand);
    let opposing_grave = put(&mut game, player.other(), "Plains", Zone::Graveyard);
    let hand_ref = game.object_ref(hand);
    let grave_ref = game.object_ref(grave);
    let opposing_grave_ref = game.object_ref(opposing_grave);

    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 7);
    assert_ne!(game.object_ref(hand), hand_ref);
    assert_ne!(game.object_ref(grave), grave_ref);
    assert_eq!(game.hand_count(player.other()), 1);
    assert_eq!(game.object_view(opposing_hand).zone, Zone::Hand);
    assert_eq!(game.object_ref(opposing_grave), opposing_grave_ref);
}

#[test]
fn free_casting_from_adventure_exile_preserves_the_ordinary_face_permission() {
    let provider = Card::from_name("Phyrexian Arena").unwrap();
    let _rules = fixture_rules::activate(&[(provider, &FREE_FIRST)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player.other(),
        "Phyrexian Arena",
        Zone::Battlefield,
    );
    let adventure = put(&mut game, player, "Hearth Elemental", Zone::Hand);
    act(&mut game, Action::AnnounceAdventure { card: adventure });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(adventure).zone, Zone::Exile);
    finish_turn(&mut game);
    finish_turn(&mut game);
    while game.state.step != Step::Main1 {
        act(&mut game, Action::Pass);
    }
    assert!(game.legal_actions().contains(&Action::AnnounceFree {
        card: adventure,
        face: CastFace::Normal,
        flash_surcharge: false,
    }));
    reject(
        &mut game,
        Action::AnnounceFree {
            card: adventure,
            face: CastFace::Adventure,
            flash_surcharge: false,
        },
    );
}
