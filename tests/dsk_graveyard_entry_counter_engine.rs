//! Conditional self-casts keep their entry counters on the exact spell.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const SPRINTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SelfGraveyardCastIfWithCounter {
            condition: StatValue::NonZombieCreatureDiedThisTurn,
            counter: CounterKind::PlusOne,
        }],
    }],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Spell),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};
const FIVE_COLORS: ManaCost = ManaCost {
    colored: [1, 1, 1, 1, 1, 0],
    generic: 0,
};
const ALTERNATIVE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SpellAlternativeManaCost(FIVE_COLORS)],
    }],
    ..CardRules::EMPTY
};

fn own_route(object: ObjectRef) -> Action {
    Action::AnnounceSelfGraveyardWithCounter {
        object,
        method: CastMethod::Normal,
        flash_surcharge: false,
    }
}

fn first_death(game: &mut Game, player: Player) {
    let victim = put(game, player, "Bear Cub", Zone::Battlefield);
    assert_eq!(
        game.move_object(victim, Zone::Graveyard).destination,
        Zone::Graveyard
    );
}

#[test]
fn the_turn_fact_uses_actual_deaths_and_effective_zombie_subtypes() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPRINTER)]);
    let mut game = scenario(PLAYER);
    let card = spawn(&mut game, PLAYER, Card::GrizzlyBears, Zone::Graveyard);
    let object = game.object_ref(card);
    let opponent = PLAYER.other();
    reject(&mut game, own_route(object));

    let creature_card = put(&mut game, opponent, "Bear Cub", Zone::Hand);
    game.move_object(creature_card, Zone::Graveyard);
    reject(&mut game, own_route(object));

    let changeling = put(&mut game, opponent, "Three Tree Mascot", Zone::Battlefield);
    assert!(game.has_subtype(changeling, "Zombie"));
    game.move_object(changeling, Zone::Graveyard);
    assert_eq!(game.observe(PLAYER).creatures_died_this_turn, 1);
    reject(&mut game, own_route(object));

    let replacement = put(&mut game, opponent, "Rest in Peace", Zone::Battlefield);
    let diverted = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    assert_eq!(
        game.move_object(diverted, Zone::Graveyard).destination,
        Zone::Exile
    );
    reject(&mut game, own_route(object));
    game.move_object(replacement, Zone::Hand);

    first_death(&mut game, opponent);
    assert!(game.observe(PLAYER).non_zombie_creature_died_this_turn);
    assert!(game.observe(opponent).non_zombie_creature_died_this_turn);
    assert!(game.legal_actions().contains(&own_route(object)));
    reject(&mut game, Action::AnnounceCast { card });
    act(&mut game, Action::Pass);
    reject(&mut game, own_route(object));

    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn > turn && game.state.step == Step::Main1 {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    assert!(game.state.turn > turn);
    assert!(!game.observe(PLAYER).non_zombie_creature_died_this_turn);
    put(&mut game, PLAYER, "High Fae Trickster", Zone::Battlefield);
    game.state.players[PLAYER.index()].mana = [8; 6];
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(PLAYER));
    reject(&mut game, own_route(object));
}

#[test]
fn an_independent_permission_and_spell_copies_do_not_inherit_the_entry_counter() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPRINTER), (opt, &COPY)]);
    let mut game = scenario(PLAYER);
    let card = spawn(&mut game, PLAYER, Card::GrizzlyBears, Zone::Graveyard);
    let stale = game.object_ref(card);
    game.move_object(card, Zone::Hand);
    game.move_object(card, Zone::Graveyard);
    let object = game.object_ref(card);
    first_death(&mut game, PLAYER);
    game.grant_graveyard_permission(PLAYER, object, GraveyardCastKind::Cast);
    reject(&mut game, own_route(stale));

    let saved = game.snapshot();
    let revision = game.revision();
    let route = own_route(object);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == route)
        .unwrap();
    let mut raw = game.clone();
    act(&mut raw, route.clone());
    game.apply_index(PLAYER, revision, index).unwrap();
    assert_eq!(game.snapshot(), raw.snapshot());
    let stack_ref = raw.object_ref(card);
    assert!(raw.has_cast_entry_counters(stack_ref));

    game.restore(&saved).unwrap();
    assert_eq!(
        game.apply_index(PLAYER, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut game, Action::AnnounceCast { card });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);

    pay(&mut raw);
    let copier = put(&mut raw, PLAYER, "Opt", Zone::Hand);
    cast(&mut raw, copier, Some(Target::Spell(card)));
    act(&mut raw, Action::Pass);
    act(&mut raw, Action::Pass);
    let spell_copy = raw.state.stack.last().unwrap();
    assert!(spell_copy.is_copy);
    let copy_id = spell_copy.source.unwrap().id;
    settle(&mut raw);
    assert_eq!(raw.object_view(copy_id).zone, Zone::Battlefield);
    assert_eq!(raw.counter_count(copy_id, CounterKind::PlusOne), 0);
    assert_eq!(raw.counter_count(card, CounterKind::PlusOne), 1);
}

#[test]
fn independent_mana_alternatives_keep_the_conditional_permission_and_counter() {
    let altar = Card::from_name("Fountainport Bell").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &SPRINTER), (altar, &ALTERNATIVE)]);
    let mut game = scenario(PLAYER);
    let card = spawn(&mut game, PLAYER, Card::GrizzlyBears, Zone::Graveyard);
    let object = game.object_ref(card);
    let provider = spawn(&mut game, PLAYER, altar, Zone::Battlefield);
    let action = Action::AnnounceCastWithManaAlternative {
        object,
        provider: game.object_ref(provider),
        face: CastFace::Normal,
        cost: FIVE_COLORS,
        flash_surcharge: false,
        origin: PaidCastOrigin::SelfGraveyardWithCounter { object },
    };
    reject(&mut game, action.clone());
    first_death(&mut game, PLAYER);
    act(&mut game, action);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
}
