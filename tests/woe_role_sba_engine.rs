//! Role uniqueness and illegal Aura attachments are checked in the same SBA.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::replacements::TokenGroupView;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROTECT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::PROTECTION_RED,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn a_newer_role_that_became_illegal_still_displaces_the_older_role() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &PROTECT)]);
    let role = TokenGroupView {
        token: Card(5248), // Wicked Role
        amount: 1,
        tapped: false,
    };
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let recipient = game.object_ref(creature);
        let spell = put(&mut game, player, "Divination", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        act(&mut game, Action::Pass);

        // Start at a single SBA boundary with two distinct timestamps. Resolving
        // protection makes only the second Role illegal at that same boundary.
        let older =
            game.create_token_groups_attached(player, &[role], None, true, Some(recipient))[0];
        let newer =
            game.create_token_groups_attached(player, &[role], None, true, Some(recipient))[0];
        game.set_color_effects(&[newer.card], 8, AnimationDuration::EndOfTurn);
        assert_eq!(game.object_view(older.card).attached_to, Some(recipient));
        assert_eq!(game.object_view(newer.card).attached_to, Some(recipient));
        act(&mut game, Action::Pass);

        for role in [older, newer] {
            assert_ne!(game.object_view(role.card).zone, Zone::Battlefield);
        }
        for _ in 0..8 {
            match game.state.pending {
                Pending::OrderTriggers { .. } => {
                    choose(&mut game, |a| matches!(a, Action::PutTrigger { .. }))
                }
                Pending::Priority if game.state.stack.is_empty() => break,
                Pending::Priority => act(&mut game, Action::Pass),
                _ => panic!("unexpected Role SBA choice: {:?}", game.state.pending),
            }
        }
        assert_eq!(game.state.players[player.other().index()].life, 18);
    }
}
