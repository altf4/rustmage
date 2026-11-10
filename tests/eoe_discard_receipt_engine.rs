//! Mandatory discard receipts record completed events, including an empty batch.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const DISCARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DiscardHandRememberCount(PlayerSet::Controller),
            Instruction::DiscardRememberCount {
                players: PlayerSet::Opponent,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::LoseLife(PlayerSet::Opponent, 3),
        ],
    ),
    ..CardRules::EMPTY
};

fn begin(g: &mut Game, caster: Player) {
    // The first discard leaves a receipt of one, so the new empty result
    // must overwrite it before the fallback is evaluated.
    let prior = put(g, caster, "Forest", Zone::Hand);
    let spell = put(g, caster, "Divination", Zone::Hand);
    cast(g, spell, None);
    act(g, Action::Pass);
    act(g, Action::Pass);
    assert_eq!(g.object_view(prior).zone, Zone::Graveyard);
}

#[test]
fn empty_hand_sets_zero_and_nonempty_hand_requires_a_private_selection() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &DISCARD)]);
    for caster in [Player::P0, Player::P1] {
        let opponent = caster.other();
        let mut empty = scenario(caster);
        begin(&mut empty, caster);
        assert_eq!(empty.state.players[opponent.index()].life, 17);

        let mut g = scenario(caster);
        let first = put(&mut g, opponent, "Island", Zone::Hand);
        let second = put(&mut g, opponent, "Mountain", Zone::Hand);
        begin(&mut g, caster);
        assert_eq!(g.actor(), Some(opponent));
        assert!(matches!(
            g.state.pending,
            Pending::EffectDiscard { player, remaining: 1 } if player == opponent
        ));
        assert!(
            g.observe(caster)
                .objects
                .iter()
                .all(|o| o.id != first && o.id != second)
        );
        assert!(g.observe(caster).group_move_count.is_none());
        reject(&mut g, Action::ChooseOptional { accept: false });

        let first_action = Action::Discard { card: first };
        let snapshot = g.snapshot();
        let revision = g.revision();
        let index = g
            .legal_actions()
            .iter()
            .position(|a| *a == first_action)
            .unwrap();
        let mut indexed = g.clone();
        assert_eq!(
            g.apply(caster, first_action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(g.snapshot(), snapshot);
        act(&mut g, first_action);
        indexed.apply_index(opponent, revision, index).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        indexed.restore(&snapshot).unwrap();
        assert_eq!(
            indexed.apply_index(opponent, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, Action::Discard { card: second });
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        assert_eq!(g.object_view(second).zone, Zone::Hand);
        assert_eq!(indexed.object_view(second).zone, Zone::Graveyard);
        assert_eq!(indexed.object_view(first).zone, Zone::Hand);
        for branch in [&g, &indexed] {
            assert_eq!(branch.state.players[opponent.index()].life, 20);
        }
    }
}

#[test]
fn a_discard_to_battlefield_still_satisfies_the_receipt() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &DISCARD)]);
    let caster = Player::P0;
    let opponent = caster.other();
    let mut g = scenario(caster);
    let liege = put(&mut g, opponent, "Wilt-Leaf Liege", Zone::Hand);
    begin(&mut g, caster);
    act(&mut g, Action::Discard { card: liege });
    settle(&mut g);
    assert_eq!(g.object_view(liege).zone, Zone::Battlefield);
    assert_eq!(g.object_view(liege).controller, opponent);
    assert_eq!(g.state.players[opponent.index()].life, 20);
}
