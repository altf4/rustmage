mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const NONLAND: SacrificeFilter = SacrificeFilter {
    artifact_enchantment_or_token: false,
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: true,
    other: false,
    tokens: None,
    subtype: None,
};
const USURPER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileControlledLinked,
                players: PlayerSet::Targeted,
                filter: SacrificeFilter {
                    kinds: 1,
                    nonland: false,
                    tokens: Some(false),
                    ..NONLAND
                },
                amount: 1,
                optional_skip: None,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileGraveyardLinked,
                players: PlayerSet::Targeted,
                filter: NONLAND,
                amount: 1,
                optional_skip: None,
            },
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::LinkedExilePaidCastOnOwnTurnWithFlash],
    }],
    ..CardRules::EMPTY
};

fn announce(game: &Game, source: CardId, card: CardId) -> Action {
    Action::AnnounceLinkedPaid {
        object: game.object_ref(card),
        source: game.object_ref(source),
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}

fn finish_choice(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishEffectChoice) {
        act(game, Action::FinishEffectChoice);
    }
}

#[test]
fn linked_casts_follow_current_control_and_can_use_any_type_of_mana_with_flash() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &USURPER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, opponent, "Divination", Zone::Graveyard);
    let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let unlinked = put(&mut game, opponent, "Divination", Zone::Exile);
    cast(&mut game, source, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));

    let first = game.object_ref(creature);
    let land = game.object_ref(land);
    reject(&mut game, Action::SelectResolutionExile { object: land });
    act(&mut game, Action::SelectResolutionExile { object: first });
    finish_choice(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.actor(), Some(opponent));

    let object = game.object_ref(spell);
    reject(&mut game, Action::SelectResolutionExile { object: land });
    let choice = Action::SelectResolutionExile { object };
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(opponent, revision, index).unwrap();
    act(&mut game, choice.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, choice);
    assert_eq!(game.snapshot(), indexed.snapshot());
    finish_choice(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);

    let action = announce(&game, source, spell);
    let unrelated = announce(&game, source, unlinked);
    reject(&mut game, unrelated);
    let mut off_turn = game.clone();
    off_turn.state.active = opponent;
    reject(&mut off_turn, action.clone());
    let mut removed = game.clone();
    removed.move_object(source, Zone::Exile);
    removed.move_object(source, Zone::Battlefield);
    let renewed = announce(&removed, source, spell);
    reject(&mut removed, renewed);

    let mut stolen = game.clone();
    stolen.record_control(stolen.object_ref(source), opponent, None);
    reject(&mut stolen, action.clone());
    stolen.state.active = opponent;
    stolen.state.priority = opponent;
    assert!(stolen.legal_actions().contains(&action));

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    assert_eq!(game.state.stack.len(), 1);
    game.state.players[player.index()].mana = [0, 0, 0, 3, 0, 0];
    act(&mut game, action);
    pay(&mut game);
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert!(cast.spend_any_mana);
    assert_eq!(cast.origin, Zone::Exile);
    assert_eq!(cast.payment, [0, 0, 0, 3, 0, 0]);
    settle(&mut game);
    assert_eq!(game.object_view(spell).owner, opponent);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn linked_flash_preserves_disguise_and_keeps_behold_optional() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &USURPER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let disguise = put(&mut game, opponent, "Bubble Smuggler", Zone::Exile);
    let spell = put(&mut game, opponent, "Molten Exhale", Zone::Exile);
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let source = game.object_ref(provider);
    for card in [disguise, spell] {
        game.link_exile_to_ability(source, None, game.object_ref(card));
    }
    game.state.step = Step::BeginCombat;
    let action = Action::AnnounceLinkedAlternative {
        object: game.object_ref(disguise),
        source,
        method: CastMethod::Disguise,
    };
    let mut face_down = game.clone();
    face_down.state.players[player.index()].mana = [0, 0, 0, 0, 3, 0];
    act(&mut face_down, action);
    pay(&mut face_down);
    let disguised = face_down.state.stack.last().unwrap();
    let cast = disguised.cast.unwrap();
    assert_eq!(cast.method, CastMethod::Disguise);
    assert!(cast.spend_any_mana);
    assert_eq!(cast.payment, [0, 0, 0, 0, 3, 0]);
    assert_eq!(
        disguised.source.unwrap().card,
        Card::from_name("@face-down/disguise").unwrap()
    );

    game.state.players[player.index()].mana = [8, 0, 0, 0, 0, 0];
    let action = announce(&game, provider, spell);
    act(&mut game, action);
    if game
        .legal_actions()
        .contains(&Action::ChooseCastBehold { pay: false })
    {
        act(&mut game, Action::ChooseCastBehold { pay: false });
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().beheld,
        Some(false)
    );
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}
