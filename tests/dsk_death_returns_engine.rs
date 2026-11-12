//! Exact death receipts for immediate returns and delayed sacrifices.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const CREATURE: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn a_counterless_death_returns_the_exact_card_tapped_to_its_owner() {
    const RETURNER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::SourceWithoutCounters),
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnDeathCardWithCounters {
                under_controller: false,
                tapped: true,
                counter: CounterKind::Stun,
                amount: 2,
            }],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::GrizzlyBears, &RETURNER)]);
    let caster = Player::P0;
    let owner = caster.other();
    let mut game = scenario(caster);
    let creature = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    game.change_controller(game.object_ref(creature), caster);
    let original = game.object_ref(creature);
    let removal = put(&mut game, caster, "Murder", Zone::Hand);
    let mut counted = game.clone();
    counted.add_counter(creature, CounterKind::Nest, 1);
    for branch in [&mut game, &mut counted] {
        cast(branch, removal, Some(Target::Card(creature)));
        pass_pair(branch);
        assert_eq!(branch.object_view(creature).zone, Zone::Graveyard);
    }
    assert!(counted.state.stack.is_empty());
    let death = game.object_ref(creature);
    assert_ne!(death, original);
    let mut stale = game.clone();
    stale.move_object(creature, Zone::Exile);
    stale.move_object(creature, Zone::Graveyard);
    assert_ne!(stale.object_ref(creature), death);
    settle(&mut stale);
    assert_eq!(stale.object_view(creature).zone, Zone::Graveyard);

    let saved = game.snapshot();
    let mut indexed = game.clone();
    let actor = indexed.actor().unwrap();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    indexed
        .apply_index(actor, indexed.revision(), index)
        .unwrap();
    act(&mut game, Action::Pass);
    settle(&mut indexed);
    settle(&mut game);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.object_ref(creature), death);
    assert_eq!(game.object_view(creature).controller, owner);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert!(game.object_view(creature).tapped);
    assert_eq!(game.counter_count(creature, CounterKind::Stun), 2);
}

#[test]
fn destroy_and_return_uses_only_actual_creature_cards_and_sacrifices_exact_entrants() {
    const DESTROY_RETURN: CardRules = CardRules {
        effect: Effect::Spell(CREATURE, &[Instruction::DestroyThenReturnForOwnEndStep]),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Murder"), &DESTROY_RETURN)]);
    let caster = Player::P1;
    let owner = caster.other();
    let mut game = scenario(caster);
    let creature = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(owner, Card::from_catalogue_id(5258), 1, false);
    let removal = put(&mut game, caster, "Murder", Zone::Hand);
    let original = game.object_ref(creature);
    let mut finality = game.clone();
    finality.add_counter(creature, CounterKind::Finality, 1);
    let mut indestructible = game.clone();
    indestructible.state.objects[creature as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let mut token_only = game.clone();
    for (branch, victim, destination) in [
        (&mut finality, creature, Zone::Exile),
        (&mut indestructible, creature, Zone::Battlefield),
        (&mut token_only, token, Zone::Outside),
    ] {
        cast(branch, removal, Some(Target::Card(victim)));
        settle(branch);
        assert_eq!(branch.state.objects[victim as usize].zone, destination);
        assert!(branch.observe(caster).delayed_sacrifices.is_empty());
    }

    cast(&mut game, removal, Some(Target::Card(creature)));
    settle(&mut game);
    let returned = game.object_ref(creature);
    assert_ne!(returned, original);
    assert_eq!(game.object_view(creature).controller, caster);
    assert_eq!(game.object_view(creature).owner, owner);
    let delays = game.observe(caster).delayed_sacrifices;
    assert_eq!(delays.len(), 1);
    assert!(delays[0].own_end_step);
    assert_eq!(delays[0].tokens, [returned]);

    let mut blink = game.clone();
    blink.move_object(creature, Zone::Exile);
    blink.move_object(creature, Zone::Battlefield);
    assert_ne!(blink.object_ref(creature), returned);
    for (branch, destination) in [
        (&mut game, Zone::Graveyard),
        (&mut blink, Zone::Battlefield),
    ] {
        branch.begin_step(Step::End);
        branch.prepare_triggers();
        settle(branch);
        assert_eq!(branch.object_view(creature).zone, destination);
    }
}

#[test]
fn a_death_observer_can_use_the_pre_death_controller_even_for_a_token() {
    const WATCHER: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::ReturnDeath(ReturnDeathFilter::Creature(
                CastRelation::Opponent,
            )),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayOpponentLife { amount: 3, skip: 2 },
                Instruction::LoseLife(PlayerSet::EventPlayer, 3),
                Instruction::Skip(1),
                Instruction::ReturnDeathCardWithCounters {
                    under_controller: true,
                    tapped: false,
                    counter: CounterKind::Finality,
                    amount: 1,
                },
            ],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &WATCHER)]);
    let caster = Player::P0;
    let opponent = caster.other();
    let mut game = scenario(caster);
    put(&mut game, caster, "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, caster, "Grizzly Bears", Zone::Battlefield);
    game.change_controller(game.object_ref(creature), opponent);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(opponent, Card::from_catalogue_id(5258), 1, false);
    let before = game.state.players[opponent.index()].life;
    for (victim, pay) in [(creature, false), (token, true)] {
        let removal = put(&mut game, caster, "Murder", Zone::Hand);
        cast(&mut game, removal, Some(Target::Card(victim)));
        pass_pair(&mut game);
        pass_pair(&mut game);
        assert_eq!(game.actor(), Some(opponent));
        act(&mut game, Action::ChooseOptional { accept: pay });
        settle(&mut game);
    }
    assert_eq!(game.state.players[opponent.index()].life, before - 3);
    assert_eq!(game.object_view(creature).controller, caster);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(creature, CounterKind::Finality), 1);
    assert_eq!(game.state.objects[token as usize].zone, Zone::Outside);
}

#[test]
fn a_cast_entry_owns_x_and_collects_both_players_sacrifices_before_moving_them() {
    const SWEEP: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeScaled {
                players: PlayerSet::Each,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: AmountValue::ChosenX,
            }],
        }],
        extras: &CardExtras {
            x_symbols: 2,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(card("Short Bow"), &SWEEP)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let yours = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Short Bow", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    let actions = [yours, theirs, second].map(|id| Action::Sacrifice {
        object: game.object_ref(id),
    });
    act(&mut game, actions[0].clone());
    assert_eq!(game.actor(), Some(opponent));
    act(&mut game, actions[1].clone());
    for id in [yours, theirs, second] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    act(&mut game, actions[2].clone());
    for id in [yours, theirs, second] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
}
