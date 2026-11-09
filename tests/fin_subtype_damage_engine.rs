mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DOUBLE_WIZARD: ReplacementSpec = ReplacementSpec::DoubleControlledSubtypeDamage("Wizard");
const DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE_WIZARD)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE_WIZARD],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DEALER: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    keywords: Keywords::LIFELINK.union(Keywords::TRAMPLE),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::Damage(2)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn subtype_doubler_uses_the_dealers_control_and_last_known_characteristics() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &DOUBLER),
        (Card::from_name("Ghitu Lavarunner").unwrap(), &DEALER),
        (Card::from_name("Grizzly Bears").unwrap(), &DEALER),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    for (name, controller, leaves, expected) in [
        ("Ghitu Lavarunner", player, true, 4),
        ("Ghitu Lavarunner", opponent, false, 2),
        ("Grizzly Bears", player, false, 2),
    ] {
        let mut game = scenario(player);
        put(&mut game, player, "Short Bow", Zone::Battlefield);
        let dealer = put(&mut game, opponent, name, Zone::Battlefield);
        if controller == player {
            game.change_controller(game.object_ref(dealer), player);
        } else {
            act(&mut game, Action::Pass);
        }
        let permanent = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        game.state.objects[permanent as usize].growth.toughness = 3;
        let target = if leaves {
            Target::Card(permanent)
        } else {
            Target::Player(controller.other())
        };
        choose(&mut game, |action| {
            matches!(action, Action::Activate { card, ability: 0, target: Some(found), .. }
                if *card == dealer && *found == target)
        });
        if leaves {
            act(&mut game, Action::Pass);
            let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
            cast(&mut game, bounce, Some(Target::Card(dealer)));
        }
        settle(&mut game);
        assert_eq!(game.state.players[controller.index()].life, 20 + expected);
        if leaves {
            assert_eq!(game.object_view(dealer).zone, Zone::Hand);
            assert_eq!(game.object_view(dealer).controller, opponent);
            assert_eq!(game.object_view(permanent).damage, expected);
        } else {
            assert_eq!(
                game.state.players[controller.other().index()].life,
                20 - expected
            );
        }
    }
}

#[test]
fn combat_damage_is_assigned_before_doubling_each_wizard_packet() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &DOUBLER),
        (Card::from_name("Ghitu Lavarunner").unwrap(), &DEALER),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let attacker = put(&mut game, player, "Ghitu Lavarunner", Zone::Battlefield);
    let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    pass_priority(&mut game);
    pass_priority(&mut game);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    pass_priority(&mut game);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
    pass_priority(&mut game);
    let range = game.damage_assignment_range().unwrap();
    assert_eq!(
        (range.target, range.minimum, range.maximum),
        (Target::Card(blocker), 2, 4)
    );
    act(
        &mut game,
        Action::AssignDamage {
            target: range.target,
            amount: 2,
        },
    );
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    assert_eq!(game.state.players[player.index()].life, 28);
}
