//! Copy creation preserves copiable values and schedules exact delayed exile.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SPLITTER: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::HASTE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::InstantSorcery,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyTokenUntilNextEndStep {
                subject: Subject::Source,
                amount: 1,
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileDelayedBattlefieldObjects],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 5)],
        },
    ],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn tokens(g: &Game) -> Vec<ObjectRef> {
    g.observe(Player::P0)
        .objects
        .into_iter()
        .filter(|view| view.zone == Zone::Battlefield && view.is_token())
        .map(|view| view.reference)
        .collect()
}
fn cast_shock(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    let spell = put(g, p, "Shock", Zone::Hand);
    cast(g, spell, Some(Target::Player(p.other())));
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("copy and delayed exile resolution exceeded action limit");
}

#[test]
fn copies_use_the_current_or_departed_copiable_values() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &SPLITTER)]);
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let source_ref = g.object_ref(source);
            cast_shock(&mut g, p);
            g.apply_temporary_copy(
                source_ref,
                CopyRecipe::new(named("Hill Giant")),
                CopyDeadline::EndOfTurn,
            );
            g.add_counter(source, CounterKind::PlusOne, 1);
            g.state.objects[source as usize].tapped = true;
            if departed {
                g.move_object(source, Zone::Hand);
            }
            drain(&mut g);
            let [copy] = tokens(&g).try_into().unwrap();
            let view = g.object_view(copy.card);
            assert_eq!(view.card, named("Hill Giant"));
            assert_eq!((view.power, view.toughness), (3, 3));
            assert!(!view.tapped);
            assert!(!view.counters.has_any());
            assert_eq!(g.observe(p).delayed_steps.unwrap().len(), 1);
            g.begin_step(Step::End);
            g.prepare_triggers();
            drain(&mut g);
            assert_ne!(g.object_view(copy.card).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn replacement_created_copy_waves_exile_only_their_own_exact_tokens() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &SPLITTER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        cast_shock(&mut g, p);
        drain(&mut g);
        assert_eq!(tokens(&g).len(), 2);
        g.move_object(source, Zone::Hand);
        cast_shock(&mut g, p);
        drain(&mut g);
        let wave = tokens(&g);
        assert_eq!(wave.len(), 6);
        assert_eq!(g.observe(p).delayed_steps.unwrap().len(), 3);

        if g.actor() != Some(p.other()) {
            act(&mut g, Action::Pass);
        }
        let spell = put(&mut g, p.other(), "Murder", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(wave[0].card)));
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].life, 25);
        let unrelated = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.active = p.other();
        g.begin_step(Step::End);
        g.prepare_triggers();
        drain(&mut g);
        assert!(tokens(&g).is_empty());
        assert_eq!(g.object_view(unrelated).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
