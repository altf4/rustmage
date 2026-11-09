//! A dynamic Ward cost samples live power and exact pre-departure power.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WARD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterRelatedUnlessPayLifeScaled(
            AmountValue::SourcePower,
        )],
    }],
    ..CardRules::EMPTY
};

fn target_source(game: &mut Game, card: Card, caster: Player) -> (CardId, CardId) {
    let source = spawn(game, caster.other(), card, Zone::Battlefield);
    let shock = put(game, caster, "Shock", Zone::Hand);
    cast(game, shock, Some(Target::Card(source)));
    assert_eq!(game.state.stack.len(), 2);
    (source, shock)
}

fn resolve_ward(game: &mut Game, caster: Player, amount: i128) {
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.actor(), Some(caster));
    let request = game.observe(caster).resolution_payment.unwrap();
    assert_eq!(request.life, Some(amount));
    assert_eq!(request.payer, caster);
}

#[test]
fn wide_life_payment_is_frozen_and_replays_after_source_departure() {
    let raubahn = Card::from_name("Raubahn, Bull of Ala Mhigo").unwrap();
    let _rules = fixture_rules::activate(&[(raubahn, &WARD)]);
    let caster = Player::P1;
    let mut parent = scenario(caster);
    let (source, shock) = target_source(&mut parent, raubahn, caster);
    let power = 5_000_000_000i128;
    parent.state.players[caster.index()].life = power + 4;
    parent.state.objects[source as usize].growth.power = power - 2;
    assert_eq!(parent.stats(source).power, power);
    let mut live = parent.clone();
    resolve_ward(&mut live, caster, power);
    act(&mut live, Action::PayResolutionLife);
    assert_eq!(live.state.players[caster.index()].life, 4);
    assert_eq!(live.object_view(shock).zone, Zone::Stack);

    let original = parent.object_ref(source);
    parent.move_object(source, Zone::Exile);
    parent.move_object(source, Zone::Battlefield);
    assert_ne!(parent.object_ref(source), original);
    assert_eq!(parent.stats(source).power, 2);
    resolve_ward(&mut parent, caster, power);
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == Action::PayResolutionLife)
        .unwrap();
    let mut indexed = parent.clone();
    act(&mut parent, Action::PayResolutionLife);
    indexed.apply_index(caster, revision, index).unwrap();
    assert_eq!(parent.snapshot(), indexed.snapshot());
    assert_eq!(parent.state.players[caster.index()].life, 4);
    assert_eq!(parent.object_view(shock).zone, Zone::Stack);
    parent.restore(&saved).unwrap();
    act(&mut parent, Action::DeclineResolutionLife);
    assert_eq!(parent.state.players[caster.index()].life, power + 4);
    assert_eq!(parent.object_view(shock).zone, Zone::Graveyard);
}

#[test]
fn negative_power_allows_zero_and_an_unaffordable_positive_cost_is_rejected() {
    let raubahn = Card::from_name("Raubahn, Bull of Ala Mhigo").unwrap();
    let _rules = fixture_rules::activate(&[(raubahn, &WARD)]);
    let caster = Player::P0;
    let mut game = scenario(caster);
    let (source, shock) = target_source(&mut game, raubahn, caster);
    let mut cannot_pay = game.clone();
    cannot_pay.state.objects[source as usize].growth.power = 19;
    resolve_ward(&mut cannot_pay, caster, 21);
    reject(&mut cannot_pay, Action::PayResolutionLife);
    act(&mut cannot_pay, Action::DeclineResolutionLife);
    assert_eq!(cannot_pay.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(cannot_pay.state.players[caster.index()].life, 20);

    game.state.objects[source as usize].growth.power = -5;
    resolve_ward(&mut game, caster, 0);
    act(&mut game, Action::PayResolutionLife);
    assert_eq!(game.state.players[caster.index()].life, 20);
    assert_eq!(game.object_view(shock).zone, Zone::Stack);
}
