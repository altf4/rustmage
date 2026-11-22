//! Nonlegendary copiable exception with separate haste and sacrifice lifetimes.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};

const GRAY_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::CreateCopyWithExceptionsAndTemporaryKeywordsAndDelayedSacrifice {
                exceptions: CopyExceptions {
                    remove_legendary: true,
                    ..CopyExceptions::NONE
                },
                keywords: Keywords::HASTE,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const PLAIN_COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[Instruction::CreateCopyTokenWithExceptions {
            subject: Subject::Targets,
            amount: 1,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn nonlegendary_copy_has_noncopyable_haste_and_only_that_copy_is_sacrificed() {
    let opt = Card::from_name("Opt").unwrap();
    let shock = Card::from_name("Shock").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &GRAY_COPY), (shock, &PLAIN_COPY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let legendary = put(&mut game, player, "Giada, Font of Hope", Zone::Battlefield);
    assert!(game.object_view(legendary).is_legendary());
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(legendary)));
    settle(&mut game);
    let first = game.controlled_token_ids(player).next().unwrap();
    assert!(!game.object_view(first).is_legendary());
    assert!(game.keywords(first).contains(Keywords::HASTE));
    assert_eq!(game.observe(player).delayed_sacrifices.len(), 1);
    assert_eq!(
        game.observe(player).delayed_sacrifices[0].tokens,
        vec![game.object_ref(first)]
    );

    let checkpoint = game.snapshot();
    let mut cleanup = game.clone();
    cleanup.state.step = Step::Cleanup;
    cleanup.finish_cleanup();
    assert!(cleanup.on_battlefield(first));
    assert!(!cleanup.keywords(first).contains(Keywords::HASTE));
    cleanup.restore(&checkpoint).unwrap();
    assert_eq!(cleanup.snapshot(), checkpoint);

    let copy_spell = spawn(&mut game, player, shock, Zone::Hand);
    cast(&mut game, copy_spell, Some(Target::Card(first)));
    settle(&mut game);
    let ordinary = game
        .controlled_token_ids(player)
        .find(|&id| id != first)
        .unwrap();
    assert!(!game.object_view(ordinary).is_legendary());
    assert!(!game.keywords(ordinary).contains(Keywords::HASTE));
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    assert!(!game.on_battlefield(first));
    assert!(game.on_battlefield(ordinary));
}
