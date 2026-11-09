mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const REDIRECT: ReplacementSpec = ReplacementSpec::RedirectControlledDamageToSource;
const TURTLE: CardRules = CardRules {
    effect: Effect::Creature(8, 20),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::RetainDamageDuringCleanup],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, REDIRECT)],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[REDIRECT],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const TWO_HITS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::DamageWithController {
            damage: 2,
            controller_damage: 3,
        }],
    ),
    keywords: Keywords::LIFELINK,
    ..CardRules::EMPTY
};
const UNPREVENTABLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::DamageCannotBePreventedThisTurn,
            Instruction::Damage(2),
        ],
    ),
    ..CardRules::EMPTY
};
const BLANK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};
const TOUCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::Damage(2)],
    ),
    keywords: Keywords::DEATHTOUCH,
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn cleanup(game: &mut Game) {
    game.state.step = Step::End;
    pass_priority(game);
}

fn redirect(game: &Game, source: CardId) -> Action {
    Action::ChooseReplacement {
        source: game.object_ref(source),
        ability: 0,
    }
}

#[test]
fn simultaneous_packets_follow_current_control_and_use_one_shield_after_converging() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &TURTLE),
        (Card::Shock, &TWO_HITS),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for shield in [false, true] {
        let mut game = scenario(player);
        let turtle = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.change_controller(game.object_ref(turtle), opponent);
        let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        if shield {
            game.add_counter(turtle, CounterKind::Shield, 2);
        }
        let bolt = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, bolt, Some(Target::Card(victim)));
        settle(&mut game);
        assert_eq!(game.object_view(turtle).controller, opponent);
        assert_eq!(game.object_view(turtle).damage, if shield { 0 } else { 5 });
        assert_eq!(
            game.counter_count(turtle, CounterKind::Shield),
            u64::from(shield)
        );
        assert_eq!(game.object_view(victim).damage, 0);
        assert_eq!(game.state.players[opponent.index()].life, 20);
        assert_eq!(
            game.state.players[player.index()].life,
            if shield { 20 } else { 25 }
        );
    }
}

#[test]
fn each_provider_can_redirect_the_packet_once_and_replay_keeps_the_selected_order() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &TURTLE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut raw = scenario(player);
    let first = put(&mut raw, opponent, "Hill Giant", Zone::Battlefield);
    let second = put(&mut raw, opponent, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut raw, player, "Shock", Zone::Hand);
    cast(&mut raw, spell, Some(Target::Player(opponent)));
    pass_priority(&mut raw);
    assert_eq!(raw.actor(), Some(opponent));
    let first_redirect = redirect(&raw, first);
    let second_redirect = redirect(&raw, second);
    let saved = raw.snapshot();
    let revision = raw.revision();
    let index = raw
        .legal_actions()
        .iter()
        .position(|action| *action == first_redirect)
        .unwrap();
    let mut indexed = raw.clone();
    assert_eq!(
        raw.apply(player, first_redirect.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);
    act(&mut raw, first_redirect);
    indexed.apply_index(opponent, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(first).damage, 0);
    assert_eq!(raw.object_view(second).damage, 2);
    assert_eq!(raw.state.players[opponent.index()].life, 20);

    indexed.restore(&saved).unwrap();
    act(&mut indexed, second_redirect);
    settle(&mut indexed);
    assert_eq!(indexed.object_view(first).damage, 2);
    assert_eq!(indexed.object_view(second).damage, 0);
    assert_eq!(indexed.state.players[opponent.index()].life, 20);
}

#[test]
fn redirection_competes_with_original_prevention_and_rechecks_the_new_recipient() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &TURTLE),
        (Card::from_name("Opt").unwrap(), &UNPREVENTABLE),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for (unpreventable, redirect_first) in [(false, false), (false, true), (true, true)] {
        let mut game = scenario(player);
        let turtle = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let shielded = if unpreventable { turtle } else { victim };
        game.add_counter(shielded, CounterKind::Shield, 1);
        let spell = put(
            &mut game,
            player,
            if unpreventable { "Opt" } else { "Shock" },
            Zone::Hand,
        );
        cast(&mut game, spell, Some(Target::Card(victim)));
        if !unpreventable {
            pass_priority(&mut game);
            assert_eq!(game.actor(), Some(opponent));
            let action = if redirect_first {
                redirect(&game, turtle)
            } else {
                Action::ChooseDamagePrevention {
                    source: game.object_ref(victim),
                    prevention: DamagePreventionKind::ShieldCounter,
                }
            };
            act(&mut game, action);
        }
        settle(&mut game);
        assert_eq!(game.object_view(victim).damage, 0);
        assert_eq!(
            game.object_view(turtle).damage,
            if redirect_first { 2 } else { 0 }
        );
        assert_eq!(
            game.counter_count(shielded, CounterKind::Shield),
            u64::from(!unpreventable && redirect_first),
        );
    }
}

#[test]
fn redirected_combat_damage_keeps_its_source_and_lifelink() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &TURTLE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let turtle = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    pass_priority(&mut game);
    pass_priority(&mut game);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    pass_priority(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_priority(&mut game);
    assert_eq!(game.object_view(turtle).damage, 1);
    assert_eq!(game.state.players[opponent.index()].life, 20);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert!(game.state.objects[attacker as usize].dealt_combat_damage_this_incarnation);
}

#[test]
fn cleanup_samples_active_damage_retention_before_temporary_effects_expire() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &TURTLE),
        (Card::from_name("Opt").unwrap(), &BLANK),
        (Card::Shock, &TOUCH),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for blank in [false, true] {
        let mut game = scenario(player);
        let turtle = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.state.objects[other as usize].damage = 1;
        let source = game.object_ref(turtle);
        game.grant_keyword_effects(&[turtle], source, Keywords::INDESTRUCTIBLE);
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(opponent)));
        settle(&mut game);
        assert_eq!(game.object_view(turtle).damage, 2);
        assert!(!game.state.objects[turtle as usize].deathtouch_since_sba);
        if blank {
            let blanker = put(&mut game, player, "Opt", Zone::Hand);
            cast(&mut game, blanker, Some(Target::Card(turtle)));
            settle(&mut game);
        }
        cleanup(&mut game);
        assert_eq!(game.state.active, opponent);
        assert_eq!(game.object_view(turtle).zone, Zone::Battlefield);
        assert_eq!(game.object_view(turtle).damage, if blank { 0 } else { 2 });
        assert_eq!(game.object_view(other).damage, 0);
        assert!(!game.keywords(turtle).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn retained_damage_can_be_lethal_as_temporary_toughness_ends() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Hill Giant").unwrap(), &TURTLE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let turtle = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.objects[turtle as usize].growth.toughness = 2;
    game.state.objects[turtle as usize].damage = 21;
    cleanup(&mut game);
    assert_eq!(game.state.step, Step::Cleanup);
    assert_eq!(game.state.active, player);
    assert_eq!(game.object_view(turtle).zone, Zone::Graveyard);
    pass_priority(&mut game);
    assert_eq!(game.state.active, player.other());
}
