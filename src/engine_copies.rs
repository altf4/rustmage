//! Runtime-created token recipes. Immutable definitions stay shared; copying a copy
//! flattens to its existing recipe, never to a pointer into another game object.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CopyRecipe {
    /// The complete immutable copiable definition, including fixed token exceptions.
    pub definition: Card,
    pub exceptions: CopyExceptions,
}
/// Frozen copiable values and the linked-ability instance which produced a packet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceCopyFacts {
    #[serde(flatten)]
    pub recipe: CopyRecipe,
    #[serde(skip_serializing_if = "is_original_ability")]
    pub ability_origin: u64,
}
fn is_original_ability(value: &u64) -> bool {
    *value == 0
}
impl CopyRecipe {
    pub const fn new(definition: Card) -> Self {
        Self {
            definition,
            exceptions: CopyExceptions::NONE,
        }
    }

    pub const fn factory_identity(self) -> &'static str {
        "Copy"
    }
    fn token_info(self) -> TokenInfo {
        let card = self.definition;
        TokenInfo {
            factory_identity: self.factory_identity(),
            name: card.name(),
            subtypes: card
                .info()
                .type_line
                .split_once('—')
                .map_or("", |(_, types)| types.trim()),
            colors: card.colors(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CopyRow {
    pub ability_origin: u64,
    pub object: ObjectRef,
    pub recipe: CopyRecipe,
}
pub(super) type Rows = Option<SharedVec<CopyRow>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<CopyRow>>() + rows.len() * std::mem::size_of::<CopyRow>()
    })
}
impl Game {
    fn copy_recipe_index(&self, object: ObjectRef) -> Option<usize> {
        let rows = self.state.copy_recipes.as_ref()?;
        let index = rows
            .binary_search_by_key(&object.card, |row| row.object.card)
            .ok()?;
        (rows[index].object == object).then_some(index)
    }
    pub(super) fn copy_recipe(&self, object: ObjectRef) -> Option<CopyRecipe> {
        let index = self.copy_recipe_index(object)?;
        Some(self.state.copy_recipes.as_ref().unwrap()[index].recipe)
    }
    pub(super) fn copy_ability_origin(&self, object: ObjectRef) -> u64 {
        self.copy_recipe_index(object).map_or(0, |index| {
            self.state.copy_recipes.as_ref().unwrap()[index].ability_origin
        })
    }
    pub(super) fn source_copy_facts(&self, object: ObjectRef) -> Option<Box<SourceCopyFacts>> {
        let recipe = self.copy_recipe(object)?;
        let ability_origin = self.copy_ability_origin(object);
        (recipe.exceptions != CopyExceptions::NONE || ability_origin != 0).then(|| {
            Box::new(SourceCopyFacts {
                recipe,
                ability_origin,
            })
        })
    }
    pub(super) fn object_token_info(&self, id: CardId) -> Option<TokenInfo> {
        let object = &self.state.objects[id as usize];
        if object.is_copy_token() {
            Some(
                // Ceasing releases the sparse recipe, but the reusable Outside
                // slot retains token identity so it cannot move as a physical
                // card. Live copies still require their exact-incarnation recipe.
                self.copy_recipe(self.object_ref(id))
                    .or_else(|| {
                        (object.zone == Zone::Outside).then(|| CopyRecipe::new(object.card))
                    })
                    .expect("runtime copy recipe")
                    .token_info(),
            )
        } else {
            object.card.token_info().copied()
        }
    }
    pub(super) fn clear_copy_recipe(&mut self, object: ObjectRef) {
        let Some(index) = self.copy_recipe_index(object) else {
            return;
        };
        let rows = self.state.copy_recipes.as_mut().unwrap();
        rows.remove(index);
        if rows.is_empty() {
            self.state.copy_recipes = None;
        }
    }
    pub(super) fn advance_copy_recipe(&mut self, before: ObjectRef, after: ObjectRef) {
        let Some(index) = self.copy_recipe_index(before) else {
            return;
        };
        assert_eq!(before.card, after.card);
        self.state.copy_recipes.as_mut().unwrap()[index].object = after;
    }
    pub(super) fn release_unentered_copy(&mut self, object: ObjectRef) {
        self.clear_copy_recipe(object);
        let slot = &mut self.state.objects[object.card as usize];
        assert_eq!(slot.zone, Zone::Outside);
        slot.generation = slot
            .generation
            .checked_add(1)
            .capacity("object generation exhausted");
        self.state.free_tokens.push(object.card);
    }
    pub(super) fn create_copy_groups(
        &mut self,
        player: Player,
        groups: &[TokenGroupView],
        recipes: &[Option<CopyRecipe>],
    ) {
        self.create_copy_groups_bound(player, groups, recipes, None, &[]);
    }
    pub(super) fn create_copy_groups_bound(
        &mut self,
        player: Player,
        groups: &[TokenGroupView],
        recipes: &[Option<CopyRecipe>],
        defenders: Option<&[CombatDefender]>,
        recipients: &[ObjectRef],
    ) -> Vec<ObjectRef> {
        assert_eq!(groups.len(), recipes.len());
        let total = groups
            .iter()
            .try_fold(0u128, |total, g| total.checked_add(g.amount))
            .capacity("token quantity exhausted");
        let amount = usize::try_from(total).capacity("token count representation exhausted");
        let fresh = amount.saturating_sub(self.state.free_tokens.len());
        (self.state.objects.len() as u128)
            .checked_add(fresh as u128)
            .filter(|&end| end <= u128::from(CardId::MAX) + 1)
            .capacity("object slots exhausted");
        self.state
            .objects
            .try_reserve(fresh)
            .capacity("object allocation exhausted");
        let mut moves = Vec::new();
        moves
            .try_reserve_exact(amount)
            .capacity("token batch allocation exhausted");
        let mut created = Vec::new();
        created
            .try_reserve_exact(amount)
            .capacity("token reference allocation exhausted");
        let mut defender_index = 0;
        for (index, (group, recipe)) in groups.iter().zip(recipes).enumerate() {
            let attachment = group
                .token
                .role()
                .and_then(|_| recipients.get(index).copied());
            if group.token.role().is_some() && !recipients.is_empty() && attachment.is_none()
                || attachment.is_some_and(|r| !self.aura_can_enchant(group.token, player, r))
            {
                continue;
            }
            assert!(recipe.is_some() || group.token.is_token());
            for _ in 0..group.amount {
                let (id, generation) = if let Some(id) = self.state.free_tokens.pop() {
                    let old = &self.state.objects[id as usize];
                    assert!((old.is_token() || old.is_spell_copy()) && old.zone == Zone::Outside);
                    (id, old.generation)
                } else {
                    (
                        CardId::try_from(self.state.objects.len())
                            .capacity("object slots exhausted"),
                        0,
                    )
                };
                let object = Object {
                    card: group.token,
                    copy_kind: if recipe.is_some() {
                        CopyKind::Token
                    } else {
                        CopyKind::None
                    },
                    owner: player,
                    controller: player,
                    zone: Zone::Outside,
                    generation,
                    tapped: false,
                    summoning_sick: false,
                    flashback: false,
                    damage: 0,
                    damaged_this_turn: false,
                    dealt_damage_this_turn: false,
                    deathtouch_since_sba: false,
                    growth: Stats::default(),
                    gained_keywords: Keywords::NONE,
                    counters: InitialCounters::default(),
                    used_abilities: 0,
                    attached_to: None,
                };
                if id as usize == self.state.objects.len() {
                    self.state.objects.push(object);
                } else {
                    self.state.objects[id as usize] = object;
                }
                if let Some(recipe) = recipe {
                    let reference = self.object_ref(id);
                    let rows = self.state.copy_recipes.get_or_insert_with(Default::default);
                    rows.try_reserve(1)
                        .capacity("copy recipe allocation exhausted");
                    let index = rows.partition_point(|row| row.object.card < id);
                    rows.insert(
                        index,
                        CopyRow {
                            ability_origin: 0,
                            object: reference,
                            recipe: *recipe,
                        },
                    );
                }
                let mut movement = self.entry_move(id, Zone::Battlefield, group.tapped);
                movement.attachment = attachment;
                if group.token.is_creature()
                    && let Some(defenders) = defenders
                {
                    movement.attacking = Some(defenders[defender_index]);
                    defender_index += 1;
                }
                created.push(ObjectRef {
                    card: id,
                    generation: generation
                        .checked_add(1)
                        .capacity("object generation exhausted"),
                });
                moves.push(movement);
            }
        }
        if !moves.is_empty() {
            self.track_nightmare_copy_entries(&moves);
            self.stage_entries(moves, None);
        }
        if let Some(defenders) = defenders {
            assert_eq!(defender_index, defenders.len());
        }
        created
    }
}
