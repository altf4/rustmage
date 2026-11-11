//! Layer-six activation grants use the recipient's actual rules and tap cost.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FRESH_TOKEN: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Filtered {
        base: &TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        predicate: &TargetPredicate::Token,
    },
    predicate: &TargetPredicate::EnteredBattlefieldThisTurn,
};
const GRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(FRESH_TOKEN)),
        program: &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledNontokenCreatureActivation(
            0,
        )],
    }],
    ..CardRules::EMPTY
};
const TREASURE: Card = Card::from_catalogue_id(5302);
const SPIRIT: Card = Card::from_catalogue_id(5326);

fn token(game: &mut Game, player: Player, definition: Card, tapped: bool) -> ObjectRef {
    game.create_token_batch_capture(player, definition, 1, tapped, None, true)[0]
}

fn grant(game: &Game, provider: CardId, creature: CardId) -> Action {
    Action::AnnounceCreatureGrant {
        source: game.object_ref(creature),
        grantor: game.object_ref(provider),
        ability: 0,
    }
}

fn choose_token(object: ObjectRef) -> Action {
    Action::ChooseTarget {
        target: Target::Card(object.card),
    }
}

fn treasure_count(game: &Game, player: Player) -> usize {
    game.observe(player)
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.card == TREASURE
                && object.controller == player
        })
        .count()
}

#[test]
fn grants_respect_token_entry_time_ownership_and_the_actual_tapped_creature() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GRANT)]);
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(other);
    let stale = token(&mut game, player, TREASURE, false);
    let turn = game.state.turn;
    game.begin_step(Step::End);
    for _ in 0..12 {
        if game.state.turn > turn {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.active, player);
    assert!(!game.object_view(stale.card).entered_this_turn);
    game.begin_step(Step::Main1);
    game.give_priority();

    let provider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[provider as usize].summoning_sick = true;
    let ready = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let sick = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.objects[sick as usize].summoning_sick = true;
    let enemy = put(&mut game, other, "Hill Giant", Zone::Battlefield);
    let spirit = token(&mut game, player, SPIRIT, false);
    let fresh = token(&mut game, player, TREASURE, true);
    let foreign = token(&mut game, other, TREASURE, false);
    game.set_counters(fresh.card, Counters::of(CounterKind::PlusOne, 2));
    for recipient in [enemy, sick, spirit.card] {
        let action = grant(&game, provider, recipient);
        reject(&mut game, action);
    }
    assert!(
        game.legal_actions()
            .contains(&grant(&game, provider, provider))
    );

    let mut blank = game.clone();
    blank.state.players[other.index()].mana[1] = 2;
    let piranhas = put(&mut blank, other, "Eaten by Piranhas", Zone::Hand);
    act(&mut blank, Action::Pass);
    cast(&mut blank, piranhas, Some(Target::Card(provider)));
    settle(&mut blank);
    let removed_grant = grant(&blank, provider, ready);
    reject(&mut blank, removed_grant);

    let action = grant(&game, provider, ready);
    act(&mut game, action);
    for forbidden in [stale, foreign] {
        reject(&mut game, choose_token(forbidden));
    }
    act(&mut game, choose_token(fresh));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    if game.state.announcing.is_some() {
        pay(&mut game);
    }
    assert!(game.object_view(ready).tapped);
    assert!(!game.object_view(provider).tapped);
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.source.unwrap().object, game.object_ref(ready));
    assert_eq!(
        packet.attachment_trigger.as_ref().unwrap().grantor,
        game.object_ref(provider)
    );

    let mut invalidated = game.clone();
    invalidated.record_control(fresh, other, None);
    settle(&mut invalidated);
    assert_eq!(treasure_count(&invalidated, player), 1);

    game.move_object(provider, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(treasure_count(&game, player), 3);
    let created = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.id != stale.card
                && object.id != fresh.card
                && object.controller == player
                && object.zone == Zone::Battlefield
                && object.card == TREASURE
        })
        .unwrap();
    assert!(created.is_token());
    assert!(!created.tapped);
    assert_eq!(created.counters.get(CounterKind::PlusOne), 0);
}
