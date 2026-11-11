//! Target-based life costs are locked alongside the spell's mana cost.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const TAX_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentSpellsTargetingSourceLifeCost(3)],
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: TAX_GROUPS,
    extras: &CardExtras {
        static_ability_groups: TAX_GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const TWO_TARGETS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[],
    ),
    ..CardRules::EMPTY
};

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn life_costs(game: &Game, player: Player) -> Vec<u32> {
    game.observe(player)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .filter_map(|row| match row.clause {
            CastCostClause::Life(amount) if !row.paid => Some(amount),
            _ => None,
        })
        .collect()
}

fn finish_cast(game: &mut Game) {
    for _ in 0..12 {
        match game.state.pending {
            Pending::Priority => return,
            Pending::Targets { .. } => act(game, Action::FinishTargets),
            Pending::PayCosts { .. } => pay(game),
            Pending::CastCosts { .. } => {
                choose(game, |action| {
                    matches!(action, Action::ChooseCastCost { .. })
                });
            }
            ref pending => panic!("unexpected cast choice {pending:?}"),
        }
    }
    panic!("cast did not finish");
}

#[test]
fn unpaid_fee_excludes_the_cast_or_only_its_unaffordable_target() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].life = 2;
    let opponent = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    let start = Action::AnnounceCast { card: spell };
    reject(&mut game, start.clone());
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, start);
    reject(&mut game, target(opponent));
    act(&mut game, target(own));
    finish_cast(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(opponent).zone, Zone::Battlefield);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 2);
}

#[test]
fn one_provider_charges_once_across_groups_and_distinct_providers_charge_separately() {
    let shock = Card::from_name("Shock").unwrap();
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER), (shock, &TWO_TARGETS)]);
    let player = Player::P0;
    let mut base = scenario(player);
    let first = put(
        &mut base,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let second = put(
        &mut base,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut base, player, "Shock", Zone::Hand);
    for (life, chosen, fees) in [(4, first, vec![3]), (7, second, vec![3, 3])] {
        let mut game = base.clone();
        game.state.players[player.index()].life = life;
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, target(first));
        if life == 4 {
            reject(&mut game, target(second));
        }
        act(&mut game, target(chosen));
        if matches!(game.state.pending, Pending::Targets { .. }) {
            act(&mut game, Action::FinishTargets);
        }
        assert_eq!(life_costs(&game, player), fees);
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let menu = indexed.legal_actions();
        let index = menu
            .iter()
            .position(|action| matches!(action, Action::BeginOrderedCosts))
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, Action::BeginOrderedCosts);
        assert_eq!(game.snapshot(), indexed.snapshot());
        finish_cast(&mut game);
        finish_cast(&mut indexed);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.state.players[player.index()].life, 1);
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), checkpoint);
    }
}

#[test]
fn the_fee_is_paid_at_cast_and_stays_paid_if_the_spell_is_countered() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    let counter = put(&mut game, player.other(), "Negate", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(victim));
    finish_cast(&mut game);
    assert_eq!(game.state.players[player.index()].life, 17);
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Spell(spell)));
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 17);
}

#[test]
fn a_life_payment_replacement_applies_to_the_actual_locked_fee() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let ashiok = put(
        &mut game,
        player,
        "Ashiok, Wicked Manipulator",
        Zone::Battlefield,
    );
    game.add_counter(ashiok, CounterKind::Loyalty, 5);
    let before = game.state.players[player.index()].library.len();
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, target(victim));
    finish_cast(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.state.players[player.index()].library.len(), before - 3);
}
