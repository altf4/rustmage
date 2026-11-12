//! A mana ability can commit a Forage cost without using the stack.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EMPTY: Mana = [0; 6];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const SPLIT: Mana = [0, 0, 0, 1, 1, 0];
const BLACKS: Mana = [0, 0, 2, 0, 0, 0];
const TAP_MANA: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(GREEN))],
    sorcery: false,
    once_per_turn: false,
};
const FORAGER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[
        TAP_MANA,
        ActivatedAbility {
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Forage,
                    filter: SacrificeFilter {
                        kinds: 0,
                        nonland: false,
                        other: false,
                        source_only: false,
                        tokens: None,
                        subtype: None,
                        keywords: Keywords::NONE,
                        artifact_enchantment_or_token: false,
                    },
                }),
                ..TAP_MANA.cost
            },
            program: &[Instruction::AddMana(ManaProduction::Combination {
                mask: 31,
                amount: 2,
            })],
            ..TAP_MANA
        },
    ],
    ..CardRules::EMPTY
};
const FOOD_MANA: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[TAP_MANA],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn child(g: &mut Game, source: CardId, production: Mana) {
    choose(g, |a| {
        matches!(a, Action::AnnounceMana { card, ability: 1, production: output, .. }
            if *card == source && (*output == production || *output == EMPTY))
    });
}
fn route(g: &mut Game, route: ForageRoute) {
    act(g, Action::ChooseForageRoute { route });
}
fn pick(g: &Game, id: CardId) -> Action {
    Action::ChooseForage {
        object: g.object_ref(id),
    }
}
fn finish_mana(g: &mut Game, production: Mana) {
    pay(g);
    if matches!(g.state.pending, Pending::EffectMana { .. }) {
        act(g, Action::ChooseMana { production });
    }
}
fn grave_cards(g: &mut Game, owner: Player) -> [CardId; 3] {
    std::array::from_fn(|_| put(g, owner, "Forest", Zone::Graveyard))
}
fn start_targeted_spell(g: &mut Game, spell: CardId, target: Target) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

#[test]
fn priority_forage_checks_sickness_and_can_sacrifice_tapped_food_for_split_mana() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &FORAGER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].mana = EMPTY;
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let food = put(&mut g, p, "Candy Trail", Zone::Battlefield);
        g.state.objects[food as usize].tapped = true;
        g.state.objects[source as usize].summoning_sick = true;
        let announcement = Action::AnnounceMana {
            card: source,
            ability: 1,
            counter_payment: Counters::default(),
            production: SPLIT,
        };
        reject(&mut g, announcement);
        assert!(!g.legal_actions().iter().any(|a| matches!(
            a, Action::AnnounceMana { card, .. } | Action::ActivateMana { card, .. }
            if *card == source
        )));
        g.state.objects[source as usize].summoning_sick = false;
        child(&mut g, source, SPLIT);
        assert!(g.observe(p.other()).forage.is_none());
        assert_eq!(g.observe(p).forage.unwrap().routes, vec![ForageRoute::Food]);
        reject(&mut g, Action::DeclineForage);
        route(&mut g, ForageRoute::Food);
        let food_action = pick(&g, food);
        act(&mut g, food_action);
        assert_eq!(g.object_view(food).zone, Zone::Battlefield);
        assert!(!g.object_view(source).tapped);
        finish_mana(&mut g, SPLIT);
        assert_eq!(g.state.pending, Pending::Priority);
        assert!(g.state.stack.is_empty());
        assert_eq!(g.state.players[p.index()].mana, SPLIT);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.object_view(food).zone, Zone::Graveyard);
    }
}

#[test]
fn either_exact_forage_route_funds_a_cast_and_retains_or_reverses_its_mana_event() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &FORAGER)]);
    for p in [Player::P0, Player::P1] {
        for selected_route in [ForageRoute::Food, ForageRoute::ExileThree] {
            let mut g = scenario(p);
            g.state.players[p.index()].mana = EMPTY;
            let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let foreign = grave_cards(&mut g, p.other());
            let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
            reject(&mut g, Action::AnnounceCast { card: spell });
            let choices = if selected_route == ForageRoute::Food {
                vec![put(&mut g, p, "Candy Trail", Zone::Battlefield)]
            } else {
                grave_cards(&mut g, p).to_vec()
            };
            g.begin_combat_rollback(p);
            start_targeted_spell(&mut g, spell, Target::Player(p.other()));
            child(&mut g, source, SPLIT);
            assert_eq!(g.observe(p).forage.unwrap().routes, vec![selected_route]);
            route(&mut g, selected_route);
            let wrong = pick(&g, foreign[0]);
            reject(&mut g, wrong);
            for &object in &choices[..choices.len() - 1] {
                let action = pick(&g, object);
                act(&mut g, action.clone());
                reject(&mut g, action);
            }
            let last = pick(&g, *choices.last().unwrap());
            let index = g.legal_actions().iter().position(|a| *a == last).unwrap();
            let revision = g.revision();
            let checkpoint = g.snapshot();
            let mut raw = g.clone();
            act(&mut raw, last.clone());
            g.apply_index(p, revision, index).unwrap();
            assert_eq!(raw.snapshot(), g.snapshot());
            g.restore(&checkpoint).unwrap();
            assert_eq!(g.apply_index(p, revision, index), Err(Error::StaleRevision));
            act(&mut g, last);
            finish_mana(&mut g, SPLIT);
            assert!(matches!(g.state.pending, Pending::PayCosts { player } if player == p));
            assert!(g.state.stack.is_empty());
            assert_eq!(g.state.players[p.index()].mana, SPLIT);

            let mut failed = g.clone();
            failed.finish_cast_rollback(false);
            assert_eq!(failed.state.pending, Pending::ManaRollback { player: p });
            let mut stale = failed.clone();
            stale.state.objects[choices[0] as usize].generation += 1;
            reject(&mut stale, Action::ChooseManaRollback { keep: true });
            for keep in [false, true] {
                let mut branch = failed.clone();
                act(&mut branch, Action::ChooseManaRollback { keep });
                assert_eq!(branch.object_view(source).tapped, keep);
                assert_eq!(
                    branch.state.players[p.index()].mana,
                    if keep { SPLIT } else { EMPTY }
                );
                assert_eq!(branch.object_view(spell).zone, Zone::Hand);
                let expected = match (selected_route, keep) {
                    (ForageRoute::Food, false) => Zone::Battlefield,
                    (ForageRoute::Food, true) | (ForageRoute::ExileThree, false) => Zone::Graveyard,
                    (ForageRoute::ExileThree, true) => Zone::Exile,
                    (ForageRoute::Black, _) => unreachable!(),
                };
                for &object in &choices {
                    assert_eq!(branch.object_view(object).zone, expected);
                }
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 17);
            assert_eq!(g.state.players[p.index()].mana, EMPTY);
        }
    }
}

#[test]
fn paying_ward_during_resolution_can_use_forage_and_resume_the_same_payment() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &FORAGER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let food = put(&mut g, p, "Candy Trail", Zone::Battlefield);
        let warded = put(&mut g, p.other(), "Vren, the Relentless", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, shock, Some(Target::Card(warded)));
        for _ in 0..8 {
            if g.legal_actions().contains(&Action::DeclineResolutionMana) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::ResolutionMana { player: p });
        let stack = g.state.stack.clone();
        child(&mut g, source, SPLIT);
        route(&mut g, ForageRoute::Food);
        let food = pick(&g, food);
        act(&mut g, food);
        finish_mana(&mut g, SPLIT);
        assert_eq!(g.state.pending, Pending::ResolutionMana { player: p });
        assert_eq!(g.state.stack, stack);
        act(&mut g, Action::PayResolutionMana { payment: SPLIT });
        settle(&mut g);
        assert_eq!(g.object_view(warded).damage, 2);
    }
}

#[test]
fn nested_route_keeps_a_food_mana_source_when_the_parent_needs_it() {
    let _rules = fixture_rules::activate(&[
        (named("Grizzly Bears"), &FORAGER),
        (named("Candy Trail"), &FOOD_MANA),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    g.state.players[p.index()].mana = EMPTY;
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let food = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let fodder = grave_cards(&mut g, p);
    let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut g, p, "Murder", Zone::Hand);
    start_targeted_spell(&mut g, spell, Target::Card(target));
    child(&mut g, source, BLACKS);
    reject(
        &mut g,
        Action::ChooseForageRoute {
            route: ForageRoute::Food,
        },
    );
    route(&mut g, ForageRoute::ExileThree);
    for object in fodder {
        let action = pick(&g, object);
        act(&mut g, action);
    }
    finish_mana(&mut g, BLACKS);
    choose(
        &mut g,
        |a| matches!(a, Action::ActivateMana { card, .. } if *card == food),
    );
    pay(&mut g);
    settle(&mut g);
    assert!(g.object_view(food).tapped);
    assert_eq!(g.object_view(target).zone, Zone::Graveyard);
}
