//! The trigger controller chooses one untargeted survivor per player before a single destruction batch.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GLYPHBRIDGE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::DestroyCreaturesExceptChosenByPower(2)],
    }],
    ..CardRules::EMPTY
};
const DEATHS: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesBatch {
            filter: CreatureFilter {
                relation: -1,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn resolve_glyphbridge(game: &mut Game, source: CardId) {
    cast(game, source, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    let trigger = game.state.stack.last().unwrap();
    assert!(trigger.targets.is_empty());
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn one_caster_selects_both_groups_and_unselected_creatures_die_simultaneously() {
    let definition = Card::from_name("Braided Net").unwrap();
    let observer = Card::from_name("Phyrexian Arena").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &GLYPHBRIDGE), (observer, &DEATHS)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, observer, Zone::Battlefield);
    let source = spawn(&mut game, player, definition, Zone::Hand);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other_ally = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let foe = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let other_foe = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    let pumped = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(pumped, CounterKind::PlusOne, 1);
    game.grant_keyword_effects(&[foe], game.object_ref(foe), Keywords::HEXPROOF);
    resolve_glyphbridge(&mut game, source);
    assert_eq!(game.actor(), Some(player));
    assert!(game.object_view(foe).keywords.contains(Keywords::HEXPROOF));
    reject(&mut game, Action::DeclineEffectPayment);
    let select = |game: &Game, id| Action::ChooseEffectObject {
        object: game.object_ref(id),
    };
    let invalid = select(&game, pumped);
    reject(&mut game, invalid);

    let saved = game.snapshot();
    let first = select(&game, foe);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == first)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, first.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, first);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.actor(), Some(player));
    let invalid = select(&game, other_foe);
    reject(&mut game, invalid);
    for id in [ally, other_ally, foe, other_foe, pumped] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    let second = select(&game, ally);
    act(&mut game, second);
    assert_eq!(game.state.stack.len(), 1);
    for id in [other_ally, other_foe, pumped] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    for id in [ally, foe] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    let life = game.state.players[player.index()].life;
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, life + 1);
}

#[test]
fn players_without_qualifying_creatures_have_no_selection_and_uncast_entry_does_not_sweep() {
    let definition = Card::from_name("Braided Net").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &GLYPHBRIDGE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, definition, Zone::Hand);
    let big = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let small = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let mut uncast = game.clone();
    uncast.move_object(source, Zone::Battlefield);
    assert!(uncast.state.pending_triggers.is_empty());
    assert_eq!(uncast.object_view(big).zone, Zone::Battlefield);

    let mut none = game.clone();
    none.move_object(small, Zone::Graveyard);
    resolve_glyphbridge(&mut none, source);
    assert_eq!(none.state.pending, Pending::Priority);
    assert_eq!(none.object_view(big).zone, Zone::Graveyard);

    resolve_glyphbridge(&mut game, source);
    let object = game.object_ref(small);
    assert_eq!(
        game.legal_actions()
            .into_iter()
            .filter(|action| *action != Action::Concede)
            .collect::<Vec<_>>(),
        vec![Action::ChooseEffectObject { object }]
    );
    act(&mut game, Action::ChooseEffectObject { object });
    assert_eq!(game.object_view(big).zone, Zone::Graveyard);
    assert_eq!(game.object_view(small).zone, Zone::Battlefield);
}
