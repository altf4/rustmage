//! A melded permanent is one game object represented by two physical cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PAIR: MeldPair = MeldPair {
    source: Card::from_catalogue_id(4846),
    partner: Card::from_catalogue_id(1443),
    combined: Card::from_catalogue_id(3344),
};

const SOURCE: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::OwnSourceAndNamedCreature(PAIR.partner),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 1, 0, 1, 0],
                    generic: 3,
                },
                skip: 1,
            },
            Instruction::Meld(PAIR),
        ],
    }],
    ..CardRules::EMPTY
};
const PARTNER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    ..CardRules::EMPTY
};
const COMBINED: CardRules = CardRules {
    effect: Effect::Creature(7, 6),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        meld_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const fn one_creature(program: &'static [Instruction]) -> CardRules {
    CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program,
        ),
        ..CardRules::EMPTY
    }
}

const COPY: CardRules = one_creature(&[Instruction::CreateCopyToken {
    subject: Subject::Targets,
    amount: 1,
}]);
const DESTROY: CardRules = one_creature(&[Instruction::Destroy]);
const TOP: CardRules = one_creature(&[Instruction::TopLibrary]);

fn begin_first_main(game: &mut Game) {
    game.state.step = Step::Draw;
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.step, Step::Main1);
    assert_eq!(game.state.stack.len(), 1);
}

fn pay_meld(game: &mut Game) {
    let player = game.state.active;
    let lands = ["Swamp", "Forest", "Forest", "Forest", "Forest"]
        .map(|name| put(game, player, name, Zone::Battlefield));
    act(game, Action::Pass);
    act(game, Action::Pass);
    for card in lands {
        act(game, Action::TapMana { card });
    }
    choose(game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
}

fn setup_pair(player: Player) -> (Game, CardId, CardId) {
    let mut game = scenario(player);
    let source = spawn(&mut game, player, PAIR.source, Zone::Battlefield);
    let partner = spawn(&mut game, player, PAIR.partner, Zone::Battlefield);
    game.state.objects[partner as usize].tapped = true;
    game.state.objects[partner as usize].counters.plus_one = 1;
    begin_first_main(&mut game);
    pay_meld(&mut game);
    (game, source, partner)
}

fn choose_partner(game: &mut Game, partner: CardId) {
    let object = game.object_ref(partner);
    act(game, Action::ChooseEffectObject { object });
    settle(game);
}

fn resolve_targeting_spell(game: &mut Game, player: Player, name: &str, target: CardId) {
    game.state.players[player.index()].mana = [8; 6];
    let spell = put(game, player, name, Zone::Hand);
    cast(game, spell, Some(Target::Card(target)));
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn meld_enters_once_and_copies_use_the_combined_face_without_the_physical_pair() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
        (Card::from_name("Think Twice").unwrap(), &COPY),
    ]);
    let player = Player::P1;
    let (mut game, source, partner) = setup_pair(player);
    let selected = Action::ChooseEffectObject {
        object: game.object_ref(partner),
    };
    let before = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == selected)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, selected);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    choose_partner(&mut indexed, partner);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let view = game.object_view(source);
    assert_eq!(view.card, PAIR.combined);
    assert_eq!(view.zone, Zone::Battlefield);
    assert_eq!((view.power, view.toughness), (7, 6));
    assert!(!view.tapped);
    assert_eq!(game.object_mana_value(source), 7);
    assert!(
        !game
            .observe(player)
            .objects
            .iter()
            .any(|object| object.id == partner)
    );
    assert_eq!(
        game.observe(player.other()).melded_permanents[0].front_cards,
        [PAIR.source, PAIR.partner]
    );

    act(&mut game, Action::Pass);
    resolve_targeting_spell(&mut game, player.other(), "Think Twice", source);
    settle(&mut game);
    let copy = game
        .observe(player.other())
        .objects
        .into_iter()
        .find(|object| {
            object.card == PAIR.combined && object.id != source && object.zone == Zone::Battlefield
        })
        .unwrap();
    assert!(copy.is_token());
    assert_eq!(game.object_mana_value(copy.id), 0);
    assert_eq!(game.observe(player).melded_permanents.len(), 1);
}

#[test]
fn graveyard_order_and_library_privacy_follow_the_actual_two_fronts() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
        (Card::from_name("Opt").unwrap(), &DESTROY),
        (Card::from_name("Think Twice").unwrap(), &TOP),
    ]);
    let player = Player::P0;
    let (mut original, source, partner) = setup_pair(player);
    choose_partner(&mut original, partner);
    for (spell, zone) in [("Opt", Zone::Graveyard), ("Think Twice", Zone::Library)] {
        let mut game = original.clone();
        resolve_targeting_spell(&mut game, player, spell, source);
        assert_eq!(game.state.pending, Pending::MeldOrdering { player });
        assert_eq!(game.object_view(source).card, PAIR.combined);
        assert!(game.observe(player.other()).meld_ordering_choice.is_none());
        let order = game.observe(player).meld_ordering_choice.unwrap();
        assert_eq!(order.destination, zone);
        let selected = Action::ChooseMeldOrder {
            first: order.components[1],
        };
        let saved = game.snapshot();
        let mut alternative = game.clone();
        act(&mut game, selected);
        settle(&mut game);
        assert_eq!(alternative.snapshot(), saved);
        act(
            &mut alternative,
            Action::ChooseMeldOrder {
                first: order.components[0],
            },
        );
        settle(&mut alternative);
        for branch in [&game, &alternative] {
            assert_eq!(branch.object_view(source).card, PAIR.source);
            assert_eq!(branch.object_view(partner).card, PAIR.partner);
            assert_eq!(branch.object_view(source).zone, zone);
            assert_eq!(branch.object_view(partner).zone, zone);
            assert!(branch.observe(player).melded_permanents.is_empty());
        }
        if zone == Zone::Graveyard {
            assert_eq!(game.hand_count(player), 1);
            assert_eq!(
                game.observe(player.other()).meld_departure_orders[0].cards,
                [game.object_ref(partner), game.object_ref(source)]
            );
        } else {
            assert_eq!(game.top_library_card(player), Some(partner));
            assert_eq!(alternative.top_library_card(player), Some(source));
            assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
            assert_eq!(
                game.observe(player.other()).meld_departure_orders,
                alternative.observe(player.other()).meld_departure_orders
            );
        }
    }
}

#[test]
fn stolen_partners_fail_the_intervening_condition_and_a_named_copy_cannot_meld() {
    let _rules = fixture_rules::activate(&[
        (PAIR.source, &SOURCE),
        (PAIR.partner, &PARTNER),
        (PAIR.combined, &COMBINED),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, PAIR.source, Zone::Battlefield);
    let partner = spawn(&mut game, player, PAIR.partner, Zone::Battlefield);
    begin_first_main(&mut game);
    game.record_control(game.object_ref(partner), player.other(), None);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert!(game.observe(player).melded_permanents.is_empty());

    let mut game = scenario(player);
    let source = spawn(&mut game, player, PAIR.source, Zone::Battlefield);
    let copy = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.apply_temporary_copy(
        game.object_ref(copy),
        CopyRecipe::new(PAIR.partner),
        CopyDeadline::EndOfTurn,
    );
    begin_first_main(&mut game);
    pay_meld(&mut game);
    choose_partner(&mut game, copy);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    assert_eq!(game.object_view(copy).zone, Zone::Exile);
    assert_eq!(game.object_view(copy).card, Card::GrizzlyBears);
    assert!(game.observe(player).melded_permanents.is_empty());
}
