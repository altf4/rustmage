//! Repeated choices own a settled quantity and a private fresh cost choice.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const VIPER: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Blight,
                amount: 1,
            },
            Instruction::OpponentSacrificeDiscardOrLoseLife {
                repetitions: AmountValue::SourceCounterQuantity(CounterKind::Blight),
                life: 4,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const ONCE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        &[Instruction::OpponentSacrificeDiscardOrLoseLife {
            repetitions: AmountValue::Fixed(1),
            life: 3,
        }],
    ),
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn pick(g: &Game, card: CardId) -> Action {
    if g.object_view(card).zone == Zone::Hand {
        Action::Discard { card }
    } else {
        Action::Sacrifice {
            object: g.object_ref(card),
        }
    }
}
fn pass_pair(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}

#[test]
fn choices_follow_settled_blight_count_and_see_each_previous_payment() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &VIPER)]);
    for p in [Player::P0, Player::P1] {
        let opponent = p.other();
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Hill Giant", Zone::Hand);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let discarding_land = put(&mut g, opponent, "Forest", Zone::Hand);
        let unused = put(&mut g, opponent, "Shock", Zone::Hand);
        let wrong_hand = put(&mut g, p, "Forest", Zone::Hand);
        let sacrifice = put(&mut g, opponent, "Candy Trail", Zone::Battlefield);
        let protected_land = put(&mut g, opponent, "Forest", Zone::Battlefield);
        cast(&mut g, spell, None);
        pass_pair(&mut g);
        g.add_counter(spell, CounterKind::Blight, 1);
        pass_pair(&mut g);
        assert_eq!(g.counter_count(spell, CounterKind::Blight), 3);
        assert_eq!(g.actor(), Some(opponent));
        assert!(g.observe(p).effect_choice.is_none());
        let candidates = g.observe(opponent).effect_choice.unwrap().candidates;
        assert!(candidates.contains(&g.object_ref(discarding_land)));
        assert!(candidates.contains(&g.object_ref(sacrifice)));
        assert!(!candidates.contains(&g.object_ref(protected_land)));
        let bad = pick(&g, wrong_hand);
        reject(&mut g, bad);

        let action = pick(&g, discarding_land);
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let revision = g.revision();
        let snapshot = g.snapshot();
        let mut indexed = g.clone();
        assert_eq!(g.apply(p, action.clone()), Err(Error::WrongActor));
        assert_eq!(g.snapshot(), snapshot);
        act(&mut g, action.clone());
        indexed.apply_index(opponent, revision, index).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        indexed.restore(&snapshot).unwrap();
        assert_eq!(
            indexed.apply_index(opponent, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);

        for branch in [&mut g, &mut indexed] {
            assert_eq!(branch.object_view(discarding_land).zone, Zone::Graveyard);
            reject(
                branch,
                Action::Discard {
                    card: discarding_land,
                },
            );
            let next = pick(branch, sacrifice);
            act(branch, next);
            assert_eq!(branch.object_view(sacrifice).zone, Zone::Graveyard);
            assert_eq!(branch.state.players[opponent.index()].life, 20);
            act(branch, Action::DeclineEffectPayment);
            assert_eq!(branch.state.pending, Pending::Priority);
            assert_eq!(branch.state.players[opponent.index()].life, 16);
            assert_eq!(branch.object_view(unused).zone, Zone::Hand);
            assert_eq!(branch.object_view(protected_land).zone, Zone::Battlefield);
        }
        assert_eq!(g.snapshot(), indexed.snapshot());
    }
}

#[test]
fn departed_source_uses_its_last_blight_count_and_one_choice_can_be_targeted() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &VIPER), (named("Shock"), &ONCE)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let viper = put(&mut g, p, "Hill Giant", Zone::Hand);
        cast(&mut g, viper, None);
        pass_pair(&mut g);
        g.add_counter(viper, CounterKind::Blight, 2);
        g.move_object(viper, Zone::Graveyard);
        pass_pair(&mut g);
        for _ in 0..2 {
            assert_eq!(g.actor(), Some(p.other()));
            assert!(
                g.observe(p.other())
                    .effect_choice
                    .unwrap()
                    .candidates
                    .is_empty()
            );
            act(&mut g, Action::DeclineEffectPayment);
        }
        assert_eq!(g.counter_count(viper, CounterKind::Blight), 0);
        assert_eq!(g.state.players[p.other().index()].life, 12);
        assert_eq!(g.state.pending, Pending::Priority);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        pass_pair(&mut g);
        act(&mut g, Action::DeclineEffectPayment);
        assert_eq!(g.state.players[p.other().index()].life, 9);
        assert_eq!(g.state.pending, Pending::Priority);
    }
}
