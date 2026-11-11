//! A scaled token copy spell protects and counts the settled battlefield group.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const OWN_PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const COMMON_GOOD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &OWN_PERMANENT,
            predicate: &TargetPredicate::Token,
        })),
        &[
            Instruction::CreateCopyTokensScaled {
                subject: Subject::Targets,
                amount: AmountValue::ChosenX,
            },
            Instruction::GrantControlledTokenKeywordsUntilNextTurn(Keywords::INDESTRUCTIBLE),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::TokensYouControl),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn setup() -> fixture_rules::Guard {
    fixture_rules::activate(&[(Card::from_name("Genesis Wave").unwrap(), &COMMON_GOOD)])
}
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn food() -> Card {
    named("@token/283fd83a9aec83a51b736926")
}
fn tokens(g: &Game, player: Player) -> Vec<ObjectRef> {
    g.observe(player)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == player && o.is_token())
        .map(|o| o.reference)
        .collect()
}
fn create(g: &mut Game, controller: Player, token: Card) -> Vec<ObjectRef> {
    let before = tokens(g, controller);
    g.create_tokens(controller, token, 1, false);
    assert!(!g.advance_replacements(false));
    assert!(!g.advance_entries());
    tokens(g, controller)
        .into_iter()
        .filter(|token| !before.contains(token))
        .collect()
}
fn declare(g: &mut Game, spell: CardId, x: u32) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseX { value: x });
}
fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}
fn commit(g: &mut Game, card: CardId) {
    act(g, target(card));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn counts_actual_copy_results_and_protects_exact_tokens_through_the_next_own_turn() {
    let _rules = setup();
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = scenario(p);
            let original = create(&mut g, p, food())[0];
            let foreign = create(&mut g, p.other(), food())[0];
            let kept = create(&mut g, p, named("@token/b60f6cd3b44b22bd289fe4e2"))[0];
            let nontoken = put(&mut g, p, "Candy Trail", Zone::Battlefield);
            put(&mut g, p, "Doubling Season", Zone::Battlefield);
            g.add_counter(original.card, CounterKind::Charge, 1);
            g.state.objects[original.card as usize].tapped = true;

            let spell = put(&mut g, p, "Genesis Wave", Zone::Hand);
            declare(&mut g, spell, x);
            reject(&mut g, target(foreign.card));
            reject(&mut g, target(nontoken));
            commit(&mut g, original.card);
            settle(&mut g);
            let protected = tokens(&g, p);
            assert_eq!(protected.len(), 2 + 2 * x as usize);
            assert_eq!(
                g.state.players[p.index()].life,
                20 + protected.len() as i128
            );
            for token in &protected {
                assert!(g.keywords(token.card).contains(Keywords::INDESTRUCTIBLE));
                if *token != original && *token != kept {
                    assert_eq!(g.object_view(token.card).card, food());
                    assert!(!g.object_view(token.card).tapped);
                    assert_eq!(g.counter_count(token.card, CounterKind::Charge), 0);
                }
            }
            assert!(!g.keywords(foreign.card).contains(Keywords::INDESTRUCTIBLE));
            assert!(!g.keywords(nontoken).contains(Keywords::INDESTRUCTIBLE));

            let later = create(&mut g, p, food());
            assert_eq!(later.len(), 2);
            assert!(
                later
                    .iter()
                    .all(|token| !g.keywords(token.card).contains(Keywords::INDESTRUCTIBLE))
            );
            g.state.objects[kept.card as usize].controller = p.other();

            let disenchant = put(&mut g, p, "Disenchant", Zone::Hand);
            cast(&mut g, disenchant, Some(Target::Card(original.card)));
            settle(&mut g);
            assert_eq!(g.object_ref(original.card), original);
            assert_eq!(g.object_view(original.card).zone, Zone::Battlefield);
            g.finish_cleanup();
            assert_eq!(g.state.active, p.other());
            assert!(
                protected
                    .iter()
                    .all(|token| g.keywords(token.card).contains(Keywords::INDESTRUCTIBLE))
            );
            g.finish_cleanup();
            assert_eq!(g.state.active, p);
            assert!(
                protected
                    .iter()
                    .all(|token| !g.keywords(token.card).contains(Keywords::INDESTRUCTIBLE))
            );
        }
    }
}

#[test]
fn a_stale_target_prevents_every_effect_even_at_zero_x() {
    let _rules = setup();
    let p = Player::P0;
    let mut g = scenario(p);
    let original = create(&mut g, p, food())[0];
    let survivor = create(&mut g, p, food())[0];
    let spell = put(&mut g, p, "Genesis Wave", Zone::Hand);
    declare(&mut g, spell, 0);
    commit(&mut g, original.card);
    if g.actor() != Some(p.other()) {
        act(&mut g, Action::Pass);
    }
    let disenchant = put(&mut g, p.other(), "Disenchant", Zone::Hand);
    cast(&mut g, disenchant, Some(Target::Card(original.card)));
    settle(&mut g);
    assert_eq!(tokens(&g, p), [survivor]);
    assert!(!g.keywords(survivor.card).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(g.state.players[p.index()].life, 20);
}
