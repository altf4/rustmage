//! Mixed-zone deployment is an optional private nontargeting entry transaction.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DEPLOY: Instruction = Instruction::MayDeployFromHandOrGraveyard {
    filter: SearchFilter::Subtype("Bird"),
    counter: CounterKind::Finality,
};
const COMBAT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::ControlledSubtypes(&["Bird"]),
            combat_only: true,
            opponent_only: false,
            one_or_more: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Deploy a Bird",
                targets: TargetPlan::NONE,
                program: &[DEPLOY],
            },
            SpellMode {
                text: "Draw a card",
                targets: TargetPlan::NONE,
                program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
const CHOICE_BIRD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureTypes(&["Bird", "Bear"])),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn choose_card(object: ObjectRef) -> Action {
    Action::ChooseEffectObject { object }
}

fn put_combat_trigger(game: &mut Game, player: Player, birds: &[CardId]) {
    let damage = birds
        .iter()
        .map(|&id| DamageEvent {
            preventable: true,
            source: game.damage_source(id),
            target: Target::Player(player.other()),
            amount: 1,
            combat: true,
        })
        .collect::<Vec<_>>();
    game.apply_damage_batch(&damage);
    game.advance_replacements(false);
    game.prepare_triggers();
    assert_eq!(game.actor(), Some(player));
    assert!(matches!(game.state.pending, Pending::Modes { .. }));
    assert!(game.state.ordering_triggers.is_empty());
    act(game, Action::ChooseMode { mode: 0 });
    assert_eq!(game.state.stack.len(), 1);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.pending, Pending::EffectChoice { player });
}

#[test]
fn combat_mode_can_choose_either_zone_and_never_reveals_hand_candidates() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &COMBAT_OBSERVER),
        (named("Aven Interrupter"), &CHOICE_BIRD),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let provider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let bird_one = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let bird_two = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let hand = put(&mut game, player, "Aven Interrupter", Zone::Hand);
        let grave = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
        let wrong_type = put(&mut game, player, "Bear Cub", Zone::Hand);
        let wrong_owner = put(&mut game, opponent, "Healer's Hawk", Zone::Graveyard);
        put_combat_trigger(&mut game, player, &[bird_one, bird_two]);
        let hidden = game.object_ref(hand);
        let public = game.object_ref(grave);
        let options = game.observe(player).effect_choice.unwrap();
        assert_eq!(options.candidates, [hidden, public]);
        assert!(options.optional);
        let opposed = game.observe(opponent).effect_choice.unwrap();
        assert_eq!(opposed.candidates, [public]);
        assert_eq!(opposed.required, options.required);
        assert_eq!(
            game.apply(opponent, choose_card(hidden)),
            Err(Error::WrongActor)
        );
        let invalid_type = game.object_ref(wrong_type);
        let invalid_owner = game.object_ref(wrong_owner);
        reject(&mut game, choose_card(invalid_type));
        reject(&mut game, choose_card(invalid_owner));

        let decline = Action::ChooseOptional { accept: false };
        let before = game.snapshot();
        let mut from_graveyard = game.clone();
        act(&mut from_graveyard, choose_card(public));
        assert_eq!(from_graveyard.object_view(grave).zone, Zone::Battlefield);
        assert_eq!(from_graveyard.object_view(hand).zone, Zone::Hand);
        assert_eq!(
            from_graveyard.counter_count(grave, CounterKind::Finality),
            1
        );
        from_graveyard.move_object(grave, Zone::Graveyard);
        assert_eq!(from_graveyard.object_view(grave).zone, Zone::Exile);

        let mut stale = game.clone();
        stale.move_object(grave, Zone::Exile);
        stale.move_object(grave, Zone::Graveyard);
        reject(&mut stale, choose_card(public));
        act(&mut stale, decline.clone());
        assert_eq!(stale.object_view(grave).zone, Zone::Graveyard);

        game.move_object(provider, Zone::Exile);
        let mut indexed = game.clone();
        let choose_hand = choose_card(hidden);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|action| *action == choose_hand)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, choose_hand);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
        let choose_type = Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Bird").unwrap()),
        };
        act(&mut game, choose_type.clone());
        act(&mut indexed, choose_type);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(game.object_view(hand).zone, Zone::Battlefield);
        assert_eq!(game.counter_count(hand, CounterKind::Finality), 1);
        assert_eq!(game.object_view(grave).zone, Zone::Graveyard);

        indexed.restore(&before).unwrap();
        act(&mut indexed, decline);
        assert_eq!(indexed.object_view(hand).zone, Zone::Hand);
        assert_eq!(indexed.object_view(grave).zone, Zone::Graveyard);
    }
}

#[test]
fn an_empty_private_hand_route_presents_the_same_public_choice_shape() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &COMBAT_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let bird = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let hidden = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    let mut empty = game.clone();
    empty.state.objects[hidden as usize].card = Card::GrizzlyBears;
    put_combat_trigger(&mut game, player, &[bird]);
    put_combat_trigger(&mut empty, player, &[bird]);

    let other = game.observe(player.other()).effect_choice.unwrap();
    let other_empty = empty.observe(player.other()).effect_choice.unwrap();
    assert_eq!(other.required, other_empty.required);
    assert_eq!(other.amount, other_empty.amount);
    assert!(other.candidates.is_empty() && other_empty.candidates.is_empty());
    assert_eq!(
        game.observe(player).effect_choice.unwrap().candidates,
        [game.object_ref(hidden)]
    );
    assert!(
        empty
            .observe(player)
            .effect_choice
            .unwrap()
            .candidates
            .is_empty()
    );
    let legal = empty.legal_actions();
    assert!(legal.contains(&Action::ChooseOptional { accept: false }));
    assert!(
        !legal
            .iter()
            .any(|action| matches!(action, Action::ChooseEffectObject { .. }))
    );
    act(&mut empty, Action::ChooseOptional { accept: false });
    assert_eq!(empty.state.pending, Pending::Priority);
}
