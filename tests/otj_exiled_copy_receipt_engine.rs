//! Non-target graveyard choice, exact exile receipt, and optional in-place copy.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXILE_COPY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileCardFromAnyGraveyardRemember,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(0),
            },
            Instruction::MayIfRememberedExiledCreature { skip: 1 },
            Instruction::CopySourceFromRememberedExiledCreature,
        ],
    }],
    ..CardRules::EMPTY
};

fn pass_to(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..12 {
        if ready(game) {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("graveyard copy scenario exceeded its priority budget");
}

fn exile(game: &Game, card: CardId) -> Action {
    Action::SelectResolutionExile {
        object: game.object_ref(card),
    }
}

#[test]
fn exile_and_copy_are_distinct_choices_using_the_exact_graveyard_receipt() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXILE_COPY)]);
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let own_noncreature = put(&mut game, player, "Forest", Zone::Graveyard);
    let foreign_creature = put(&mut game, other, "Hill Giant", Zone::Graveyard);
    let not_in_graveyard = put(&mut game, other, "Hill Giant", Zone::Exile);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, source, None);
    pass_to(&mut game, |game| {
        game.state
            .stack
            .last()
            .is_some_and(|packet| matches!(packet.kind, StackKind::TriggeredAbility { .. }))
    });
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    let mut departed = game.clone();
    departed.move_object(source, Zone::Graveyard);

    pass_to(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    let source_ref = game.object_ref(source);
    assert_eq!(game.object_view(source).counters.plus_one, 1);
    let invalid = exile(&game, not_in_graveyard);
    reject(&mut game, invalid);
    let mut skipped = game.clone();
    act(&mut skipped, Action::ChooseOptional { accept: false });
    settle(&mut skipped);
    assert_eq!(skipped.object_view(source).card, Card::GrizzlyBears);
    assert_eq!(skipped.object_view(foreign_creature).zone, Zone::Graveyard);

    let mut noncreature = game.clone();
    let choice = exile(&noncreature, own_noncreature);
    act(&mut noncreature, choice);
    settle(&mut noncreature);
    assert_eq!(noncreature.object_view(own_noncreature).zone, Zone::Exile);
    assert_eq!(noncreature.object_view(source).card, Card::GrizzlyBears);

    pass_to(&mut departed, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    let choice = exile(&departed, foreign_creature);
    act(&mut departed, choice);
    settle(&mut departed);
    assert_eq!(departed.object_view(foreign_creature).zone, Zone::Exile);
    assert_eq!(departed.object_view(source).zone, Zone::Graveyard);

    let choice = exile(&game, foreign_creature);
    act(&mut game, choice);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
    assert_eq!(game.object_view(foreign_creature).zone, Zone::Exile);
    let mut declined_copy = game.clone();
    act(&mut declined_copy, Action::ChooseOptional { accept: false });
    settle(&mut declined_copy);
    assert_eq!(declined_copy.object_view(source).card, Card::GrizzlyBears);
    assert_eq!(
        declined_copy.object_view(foreign_creature).zone,
        Zone::Exile
    );

    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.object_ref(source), source_ref);
    assert_eq!(
        game.object_view(source).card,
        game.object_view(foreign_creature).card
    );
    assert_eq!(game.object_view(source).counters.plus_one, 1);
    let turn = game.state.turn;
    game.begin_step(Step::End);
    pass_to(&mut game, |game| game.state.turn > turn);
    assert_eq!(game.object_view(source).card, Card::GrizzlyBears);
    assert_eq!(game.object_view(source).counters.plus_one, 1);
}
