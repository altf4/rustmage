//! An external tap-mana template pays life from the exact receiving source.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
    sorcery: false,
    once_per_turn: false,
};
const MONUMENT: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP,
        ActivatedAbility {
            cost: ActivationCost {
                life: 1,
                ..TAP.cost
            },
            program: &[Instruction::AddMana(ManaProduction::any_color(1))],
            ..TAP
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::OtherControlledSubtypeTapManaActivation {
                subtype: "Cave",
                ability: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const CAVE: CardRules = CardRules {
    effect: Effect::Land,
    ..CardRules::EMPTY
};

fn grant(game: &Game, source: CardId, grantor: CardId, production: Mana) -> Action {
    Action::AnnounceGrantedManaTemplate {
        source: game.object_ref(source),
        grantor: game.object_ref(grantor),
        ability: 1,
        production,
    }
}

fn fixture() -> (Card, Card) {
    (
        Card::from_name("Captivating Cave").unwrap(),
        Card::from_name("Hidden Cataract").unwrap(),
    )
}

#[test]
fn only_other_controlled_caves_get_the_exact_life_cost() {
    let (monument, cave) = fixture();
    let _scope = fixture_rules::activate(&[(monument, &MONUMENT), (cave, &CAVE)]);
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    game.state.players[p.index()].life = 3;
    let provider = spawn(&mut game, p, monument, Zone::Battlefield);
    let source = spawn(&mut game, p, cave, Zone::Battlefield);
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let enemy = spawn(&mut game, p.other(), cave, Zone::Battlefield);
    for ineligible in [provider, forest, enemy] {
        let action = grant(&game, ineligible, provider, BLUE);
        reject(&mut game, action);
    }
    let action = grant(&game, source, provider, BLUE);
    let mut stripped = game.clone();
    stripped.remove_abilities_exact(
        &[game.object_ref(source)],
        game.object_ref(provider),
        p,
        AnimationDuration::Permanent,
    );
    reject(&mut stripped, action.clone());
    let mut lost = game.clone();
    lost.record_control(game.object_ref(provider), p.other(), None);
    reject(&mut lost, action.clone());
    let mut reentered = game.clone();
    reentered.move_object(provider, Zone::Graveyard);
    reentered.move_object(provider, Zone::Battlefield);
    reject(&mut reentered, action.clone());
    assert!(
        reentered
            .legal_actions()
            .contains(&grant(&reentered, source, provider, BLUE))
    );

    act(&mut game, action);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        assert_eq!(game.state.players[p.index()].life, 3);
        pay(&mut game);
    }
    assert_eq!(game.state.players[p.index()].life, 2);
    assert_eq!(game.state.players[p.index()].mana, BLUE);
    assert!(game.object_view(source).tapped);
    assert!(!game.object_view(provider).tapped);
    assert!(game.state.stack.is_empty());
}

#[test]
fn nested_cast_funding_charges_each_life_payment_and_restores_exact_branches() {
    let (monument, cave) = fixture();
    let _scope = fixture_rules::activate(&[(monument, &MONUMENT), (cave, &CAVE)]);
    let p = Player::P1;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    game.state.players[p.index()].life = 3;
    let provider = spawn(&mut game, p, monument, Zone::Battlefield);
    let first = spawn(&mut game, p, cave, Zone::Battlefield);
    let second = spawn(&mut game, p, cave, Zone::Battlefield);
    let spell = put(&mut game, p, "Pond Prophet", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    let mut insufficient = game.clone();
    insufficient.state.players[p.index()].life = 1;
    reject(&mut insufficient, cast.clone());

    act(&mut game, cast);
    if matches!(game.state.pending, Pending::SpellCost { .. }) {
        let all_blue = game
            .legal_actions()
            .into_iter()
            .filter(|action| matches!(action, Action::ChooseSpellCost { .. }))
            .last()
            .unwrap();
        act(&mut game, all_blue);
    }
    let action = grant(&game, first, provider, BLUE);
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let saved = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    for branch in [&mut game, &mut indexed] {
        pay(branch);
        assert_eq!(branch.state.players[p.index()].life, 2);
        let second = grant(branch, second, provider, BLUE);
        act(branch, second);
        pay(branch);
        assert_eq!(branch.state.players[p.index()].life, 1);
        pay(branch);
        assert_eq!(branch.object_view(spell).zone, Zone::Stack);
        assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    pay(&mut indexed);
    assert_eq!(indexed.state.players[p.index()].life, 2);
}
