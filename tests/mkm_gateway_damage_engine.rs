//! Fixed damage from the resolving ability's current creature cohort.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GATEWAY: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::PermanentsDealFixedDamage {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            exclude_source: false,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const LIFELINK: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    keywords: Keywords::LIFELINK,
    ..CardRules::EMPTY
};
const DEATHTOUCH: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::DEATHTOUCH,
    ..CardRules::EMPTY
};

#[test]
fn fixed_damage_uses_live_creature_sources_and_their_keywords() {
    let enchantment = Card::from_name("Authority of the Consuls").unwrap();
    let hawk = Card::from_name("Healer's Hawk").unwrap();
    let _rules = fixture_rules::activate(&[
        (enchantment, &GATEWAY),
        (hawk, &LIFELINK),
        (Card::GrizzlyBears, &DEATHTOUCH),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let gateway = put(&mut game, player, "Authority of the Consuls", Zone::Hand);
    let lifelink = spawn(&mut game, player, hawk, Zone::Battlefield);
    let deathtouch = spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut game, opponent, Card::GrizzlyBears, Zone::Battlefield);
    let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
    assert_eq!(stats(&game, lifelink).0, 0);
    cast(&mut game, gateway, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(lifelink),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );

    let mut shielded = game.clone();
    shielded.add_counter(target, CounterKind::Shield, 1);
    settle(&mut shielded);
    assert_eq!(shielded.counter_count(target, CounterKind::Shield), 0);
    assert_eq!(shielded.object_view(target).zone, Zone::Battlefield);
    assert_eq!(shielded.object_view(target).damage, 0);
    assert_eq!(shielded.state.players[player.index()].life, 20);

    let mut protected = game.clone();
    let reference = protected.object_ref(target);
    protected.grant_keyword_effects(&[target], reference, Keywords::PROTECTION_GREEN);
    settle(&mut protected);
    assert_eq!(protected.object_view(target).zone, Zone::Battlefield);
    assert_eq!(protected.object_view(target).damage, 1);
    assert!(protected.object_view(lifelink).dealt_damage_this_turn);
    assert!(!protected.object_view(deathtouch).dealt_damage_this_turn);
    assert_eq!(protected.state.players[player.index()].life, 21);

    let mut departed = game.clone();
    act(&mut departed, Action::Pass);
    cast(&mut departed, bounce, Some(Target::Card(deathtouch)));
    settle(&mut departed);
    assert_eq!(departed.object_view(deathtouch).zone, Zone::Hand);
    assert_eq!(departed.object_view(target).damage, 1);
    assert_eq!(departed.state.players[player.index()].life, 21);

    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert!(game.object_view(lifelink).dealt_damage_this_turn);
    assert!(game.object_view(deathtouch).dealt_damage_this_turn);
    assert!(!game.object_view(enemy).dealt_damage_this_turn);
    assert!(!game.object_view(gateway).dealt_damage_this_turn);
    assert_eq!(game.state.players[player.index()].life, 21);
}
