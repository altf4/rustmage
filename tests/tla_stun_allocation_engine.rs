//! Resolution-time distribution owns non-targeted choices until one counter event.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const WAVE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: usize::MAX,
        }),
        &[
            Instruction::Tap,
            Instruction::DistributeCountersAmongPermanents {
                total: 3,
                kind: CounterKind::Stun,
                filter: BattlefieldFilter {
                    kinds: 1,
                    relation: 1,
                    nonland: false,
                    tapped: Some(true),
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                },
            },
        ],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::RequiredX),
        targets_up_to_x: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HEXPROOF: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HEXPROOF,
    ..CardRules::EMPTY
};

fn cast_wave(game: &mut Game, spell: CardId, target: Option<CardId>) {
    act(game, Action::AnnounceCast { card: spell });
    act(
        game,
        Action::ChooseX {
            value: u32::from(target.is_some()),
        },
    );
    if let Some(card) = target {
        act(
            game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
}

fn begin_resolution(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn assign(game: &Game, card: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(card),
    }
}

#[test]
fn counters_reach_tapped_nontargets_after_target_taps_and_commit_together() {
    let wave = Card::from_name("Quick Study").unwrap();
    let protected = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(wave, &WAVE), (protected, &HEXPROOF)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        let spell = spawn(&mut game, actor, wave, Zone::Hand);
        let target = put(&mut game, actor.other(), "Bear Cub", Zone::Battlefield);
        let untapped = put(&mut game, actor.other(), "Bear Cub", Zone::Battlefield);
        let own = put(&mut game, actor, "Bear Cub", Zone::Battlefield);
        let noncreature = put(&mut game, actor.other(), "Short Bow", Zone::Battlefield);
        let hexproof = spawn(&mut game, actor.other(), protected, Zone::Battlefield);
        for tapped in [own, noncreature, hexproof] {
            game.state.objects[tapped as usize].tapped = true;
        }
        assert!(game.keywords(hexproof).contains(Keywords::HEXPROOF));
        game.state.players[actor.index()].mana = [0, 1, 0, 0, 0, 3];

        let mut zero = game.clone();
        cast_wave(&mut zero, spell, None);
        begin_resolution(&mut zero);
        for _ in 0..3 {
            let action = assign(&zero, hexproof);
            act(&mut zero, action);
        }
        assert_eq!(zero.counter_count(hexproof, CounterKind::Stun), 3);
        assert!(!zero.object_view(target).tapped);

        cast_wave(&mut game, spell, Some(target));
        let mut illegal = game.clone();
        illegal.move_object(target, Zone::Exile);
        begin_resolution(&mut illegal);
        assert_eq!(illegal.state.pending, Pending::Priority);
        assert_eq!(illegal.counter_count(hexproof, CounterKind::Stun), 0);

        begin_resolution(&mut game);
        assert!(game.object_view(target).tapped);
        assert_eq!(
            game.observe(actor).effect_choice.unwrap().required[actor.index()],
            3
        );
        reject(&mut game, Action::FinishEffectChoice);
        for forbidden in [own, untapped, noncreature] {
            let action = assign(&game, forbidden);
            reject(&mut game, action);
        }
        let target_choice = assign(&game, target);
        act(&mut game, target_choice.clone());
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        let hexproof_choice = assign(&game, hexproof);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == hexproof_choice)
            .unwrap();
        indexed
            .apply_index(actor, indexed.revision(), index)
            .unwrap();
        act(&mut game, hexproof_choice.clone());
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.counter_count(target, CounterKind::Stun), 0);
        assert_eq!(game.counter_count(hexproof, CounterKind::Stun), 0);
        act(&mut game, hexproof_choice.clone());
        assert_eq!(game.counter_count(target, CounterKind::Stun), 1);
        assert_eq!(game.counter_count(hexproof, CounterKind::Stun), 2);

        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, target_choice);
        act(&mut indexed, hexproof_choice);
        assert_eq!(indexed.counter_count(target, CounterKind::Stun), 2);
        assert_eq!(indexed.counter_count(hexproof, CounterKind::Stun), 1);
    }
}
