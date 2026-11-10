//! The optional battlefield destination follows the surviving target packet.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 2,
        }),
        &[Instruction::ReturnGraveyardTargetsToHandOrBattlefield {
            maximum_mana_value: 4,
            bargain_only: true,
        }],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn announce(game: &mut Game, player: Player, bargain: bool, targets: &[CardId]) {
    let source = put(game, player, "Quick Study", Zone::Hand);
    let artifact = put(game, player, "Short Bow", Zone::Battlefield);
    let sacrifice = bargain.then(|| game.object_ref(artifact));
    act(game, Action::AnnounceCast { card: source });
    act(game, Action::ChooseBargain { sacrifice });
    for &target in targets {
        act(
            game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    if targets.len() < 2 {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

struct ResetPrevention;
impl Drop for ResetPrevention {
    fn drop(&mut self) {
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
    }
}

#[test]
fn choosing_or_declining_routes_exact_targets_in_one_owned_entry_batch() {
    let spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &RETURN)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    let automaton = put(&mut parent, player, "Adaptive Automaton", Zone::Graveyard);
    let giant = put(&mut parent, player, "Hill Giant", Zone::Graveyard);
    let unselected = put(&mut parent, player, "Grizzly Bears", Zone::Graveyard);
    let foreign = put(&mut parent, player.other(), "Bear Cub", Zone::Graveyard);
    announce(&mut parent, player, true, &[automaton, giant]);
    resolve_top(&mut parent);
    let auto = parent.object_ref(automaton);
    let giant_object = parent.object_ref(giant);
    let expected = [auto, giant_object];
    for viewer in [player, player.other()] {
        let view = parent.observe(viewer).effect_choice.unwrap();
        assert!(view.optional);
        assert_eq!(view.candidates, expected);
    }
    for invalid in [unselected, foreign] {
        let object = parent.object_ref(invalid);
        reject(&mut parent, Action::SelectGraveyardReturn { object });
    }

    let saved = parent.snapshot();
    let select = Action::SelectGraveyardReturn { object: auto };
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == select)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    act(&mut raw, select);
    indexed
        .apply_index(player, parent.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for branch in [&mut raw, &mut indexed] {
        assert_eq!(branch.object_ref(automaton), auto);
        assert_eq!(branch.object_ref(giant), giant_object);
        assert_eq!(branch.object_view(giant).zone, Zone::Graveyard);
        choose(branch, |action| {
            matches!(action, Action::ChooseEntryValue { .. })
        });
        assert_eq!(branch.object_view(automaton).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(giant).zone, Zone::Hand);
        assert_eq!(branch.object_view(unselected).zone, Zone::Graveyard);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());

    raw.restore(&saved).unwrap();
    act(&mut raw, Action::FinishGraveyardReturn);
    assert_eq!(raw.object_view(automaton).zone, Zone::Hand);
    assert_eq!(raw.object_view(giant).zone, Zone::Hand);

    act(
        &mut parent,
        Action::SelectGraveyardReturn {
            object: giant_object,
        },
    );
    assert_eq!(parent.object_view(automaton).zone, Zone::Hand);
    assert_eq!(parent.object_view(giant).zone, Zone::Battlefield);
}

#[test]
fn unpaid_ineligible_stale_and_prevented_destinations_follow_the_live_result() {
    let spell = Card::from_name("Quick Study").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &RETURN)]);
    let player = Player::P1;
    let mut root = scenario(player);
    let four = put(&mut root, player, "Hill Giant", Zone::Graveyard);
    let six = put(&mut root, player, "Shivan Dragon", Zone::Graveyard);

    let mut unpaid = root.clone();
    announce(&mut unpaid, player, false, &[four, six]);
    resolve_top(&mut unpaid);
    assert_eq!(unpaid.state.pending, Pending::Priority);
    assert_eq!(unpaid.object_view(four).zone, Zone::Hand);
    assert_eq!(unpaid.object_view(six).zone, Zone::Hand);

    let mut no_targets = root.clone();
    announce(&mut no_targets, player, true, &[]);
    resolve_top(&mut no_targets);
    assert_eq!(no_targets.state.pending, Pending::Priority);
    assert_eq!(no_targets.object_view(four).zone, Zone::Graveyard);

    announce(&mut root, player, true, &[four, six]);
    let mut stale = root.clone();
    stale.move_object(four, Zone::Exile);
    stale.move_object(four, Zone::Graveyard);
    resolve_top(&mut stale);
    assert_eq!(stale.state.pending, Pending::Priority);
    assert_eq!(stale.object_view(four).zone, Zone::Graveyard);
    assert_eq!(stale.object_view(six).zone, Zone::Hand);

    let mut none_survive = root.clone();
    none_survive.move_object(four, Zone::Exile);
    none_survive.move_object(six, Zone::Exile);
    resolve_top(&mut none_survive);
    assert_eq!(none_survive.state.pending, Pending::Priority);
    assert_eq!(none_survive.object_view(four).zone, Zone::Exile);
    assert_eq!(none_survive.object_view(six).zone, Zone::Exile);

    resolve_top(&mut root);
    let four = root.object_ref(four);
    let ineligible = root.object_ref(six);
    reject(
        &mut root,
        Action::SelectGraveyardReturn { object: ineligible },
    );
    let _reset = ResetPrevention;
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(four)));
    root.state.zone_rules.get_or_insert_default();
    act(&mut root, Action::SelectGraveyardReturn { object: four });
    assert_eq!(root.object_ref(four.card), four);
    assert_eq!(root.object_view(four.card).zone, Zone::Graveyard);
    assert_eq!(root.object_view(six).zone, Zone::Hand);
}
