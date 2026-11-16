//! Remaining generic rules used by printed Star Trek T–Z cards.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SOURCE_COPY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        sorcery: true,
        once_per_turn: false,
        cost: ActivationCost {
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateSourceCopyWithDelayedSacrifice],
    }],
    ..CardRules::EMPTY
};

const GRAVE_CARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const OWNERS_SHUFFLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: GRAVE_CARD,
            min: 0,
            max: 2,
        }),
        &[Instruction::ShuffleGraveyardTargetsIntoOwnersLibraries { cards_group: 0 }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn exiled_grave_source_still_creates_its_exact_copy_with_next_end_step_sacrifice() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &SOURCE_COPY)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let reference = game.object_ref(source);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    let checkpoint = game.snapshot();
    let actor = game.actor().unwrap();
    let revision = game.revision();
    let action = Action::Pass;
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    settle(&mut game);
    let token = game
        .controlled_token_ids(Player::P0)
        .find(|&id| game.object_view(id).card == Card::from_name("Grizzly Bears").unwrap())
        .expect("source copy token");
    assert_eq!(stats(&game, token), (2, 2));
    assert!(!game.object_view(token).keywords.contains(Keywords::HASTE));
    let delayed = &game.observe(Player::P0).delayed_sacrifices;
    assert_eq!(delayed.len(), 1);
    assert_eq!(delayed[0].source, reference);
    assert_eq!(delayed[0].tokens, vec![game.object_ref(token)]);
    assert!(!delayed[0].own_end_step);
}

#[test]
fn grave_targets_move_together_and_each_owner_shuffles_once() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Unsummon").unwrap(), &OWNERS_SHUFFLE)]);
    let mut game = scenario(Player::P0);
    let ours = put(&mut game, Player::P0, "Forest", Zone::Graveyard);
    let theirs = put(&mut game, Player::P1, "Island", Zone::Graveyard);
    let library_sizes = [
        game.state.players[0].library.len(),
        game.state.players[1].library.len(),
    ];
    let spell = put(&mut game, Player::P0, "Unsummon", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ours),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(theirs),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(ours).zone, Zone::Library);
    assert_eq!(game.object_view(theirs).zone, Zone::Library);
    assert_eq!(game.state.players[0].library.len(), library_sizes[0] + 1);
    assert_eq!(game.state.players[1].library.len(), library_sizes[1] + 1);
}
