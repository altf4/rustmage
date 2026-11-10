//! Live attacking counts are independent of who controls a copied ability.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ATTACKER: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowScaled {
                subject: Subject::Source,
                amount: AmountValue::Stat(StatValue::AllAttackingCreatures),
                power: 1,
                toughness: 0,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::AllAttackingSubtype("Rat")),
                minimum: 1,
                skip: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::TRAMPLE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const FORK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn an_opposing_copy_checks_the_current_attackers_and_rats() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Hill Giant").unwrap(), &ATTACKER),
        (Card::from_name("Opt").unwrap(), &FORK),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let rat = put(&mut game, player, "Burglar Rat", Zone::Battlefield);
    let copy_spell = put(&mut game, player.other(), "Opt", Zone::Hand);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::Attack { card: rat });
    act(&mut game, Action::FinishAttackers);

    let original = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player.other()));
    cast(&mut game, copy_spell, Some(Target::Stack(original)));
    resolve_top(&mut game);
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    let copy = game.state.stack.last().unwrap();
    assert!(copy.is_copy);
    assert_eq!(copy.controller, player.other());
    assert_eq!(copy.source.unwrap().object, game.object_ref(attacker));
    let copy_id = copy.id;

    let mut removed = game.clone();
    let responder = removed.actor().unwrap();
    let shock = put(&mut removed, responder, "Shock", Zone::Hand);
    cast(&mut removed, shock, Some(Target::Card(rat)));
    resolve_top(&mut removed);
    assert_eq!(removed.object_view(rat).zone, Zone::Graveyard);
    assert_eq!(removed.state.stack.last().unwrap().id, copy_id);
    resolve_top(&mut removed);
    assert_eq!(removed.state.stack.last().unwrap().id, original);
    assert_eq!(
        removed.stats(attacker),
        Stats {
            power: 4,
            toughness: 3
        }
    );
    assert!(!removed.keywords(attacker).contains(Keywords::TRAMPLE));

    resolve_top(&mut game);
    assert_eq!(game.state.stack.last().unwrap().id, original);
    assert_eq!(
        game.stats(attacker),
        Stats {
            power: 5,
            toughness: 3
        }
    );
    assert!(game.keywords(attacker).contains(Keywords::TRAMPLE));
}
