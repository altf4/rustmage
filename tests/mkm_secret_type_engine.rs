//! A secret choice is private until the linked activation's reveal cost is paid.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ATTACKING_TOKEN: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Filtered {
        base: &TargetRule::Creature,
        predicate: &TargetPredicate::Token,
    },
    predicate: &TargetPredicate::Attacking,
};
const KILLER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseSecretCreatureType(&["Spirit", "Goblin"])],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::RevealChosenSecretCreatureType,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(ATTACKING_TOKEN)),
        program: &[
            Instruction::IfTargetHasRevealedCreatureType { skip: 2 },
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DEATHTOUCH,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn choose_type(name: &str) -> Action {
    Action::ChooseCreatureType {
        creature_type: CreatureType::from_name(name).unwrap(),
    }
}

#[test]
fn the_choice_stays_private_and_only_its_chooser_can_pay_the_reveal_cost() {
    let source_card = Card::from_name("Doubling Season").unwrap();
    let _rules = fixture_rules::activate(&[(source_card, &KILLER)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = spawn(&mut game, player, source_card, Zone::Hand);
    cast(&mut game, source, None);
    for _ in 0..4 {
        act(&mut game, Action::Pass);
    }
    assert!(matches!(
        game.state.pending,
        Pending::CreatureTypeChoice { .. }
    ));
    reject(&mut game, choose_type("Human"));

    let mut alternate = game.clone();
    act(&mut game, choose_type("Spirit"));
    act(&mut alternate, choose_type("Goblin"));
    assert_eq!(
        game.observe(player).secret_creature_types[0]
            .creature_type
            .name(),
        "Spirit"
    );
    assert!(game.observe(opponent).secret_creature_types.is_empty());
    assert_eq!(
        serde_json::to_value(game.observe(opponent)).unwrap(),
        serde_json::to_value(alternate.observe(opponent)).unwrap(),
    );

    let announcement = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    for branch in [&mut game, &mut alternate] {
        let token = branch.state.objects.len() as CardId;
        branch.create_tokens(player, Card::from_catalogue_id(5342), 1, false);
        assert!(!branch.advance_entries());
        branch.state.objects[token as usize].summoning_sick = false;
        branch.state.step = Step::Attackers;
        branch.state.pending = Pending::Attackers;
        act(branch, Action::Attack { card: token });
        act(branch, Action::FinishAttackers);

        let mut stolen = branch.clone();
        stolen.record_control(stolen.object_ref(source), opponent, None);
        stolen.state.priority = opponent;
        reject(&mut stolen, announcement.clone());

        act(branch, announcement.clone());
        act(
            branch,
            Action::ChooseTarget {
                target: Target::Card(token),
            },
        );
        assert!(branch.observe(opponent).stack.is_empty());
        pay(branch);
        assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
        let view = branch.observe(opponent);
        assert!(view.secret_creature_types.is_empty());
        assert!(matches!(
            view.stack
                .last()
                .unwrap()
                .event_object
                .as_ref()
                .unwrap()
                .facts,
            EventObjectFacts::RevealedCreatureType(_)
        ));
        settle(branch);
    }
    let token = game.state.objects.len() as CardId - 1;
    assert_eq!(game.counter_count(token, CounterKind::PlusOne), 3);
    assert_eq!(alternate.counter_count(token, CounterKind::PlusOne), 0);
    assert!(game.keywords(token).contains(Keywords::DEATHTOUCH));
    assert!(!alternate.keywords(token).contains(Keywords::DEATHTOUCH));
}
