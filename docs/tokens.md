# Fixed creature tokens

Creation instructions refer to immutable catalogue factories. A factory specifies
power/toughness, colors, subtypes, optional artifact/enchantment type, name, and
implemented static abilities. Two tokens with the same name may have different
factories. Default names are subtypes plus “Token”; explicitly named Cadet retains
its distinct name. Colors use W/U/B/R/G bits (1/2/4/8/16).

Factories use synthetic `@token/<digest>` identities, separate from rules names.
`Card::token_info()` reports their immutable metadata. They are excluded from
`Card::ALL` and rejected in submitted decks, including Freeform. Their count is
separate from executable Standard-card coverage.

Creation allocates all tokens before dispatching entry triggers. Tokens enter
under their creator's control, with summoning sickness and the instructed tapped
status. Owned state stores compact factory identities, changing characteristics,
and incarnation generations; definitions remain shared across clones.

A departing token produces the normal zone-change event, including last-known
characteristics. Once it has left the battlefield it cannot move again, even
before state-based actions. It then ceases to exist at the next state-based check.
The engine repeats checks to a fixed point, so lethally damaged tokens move and
cease before priority. Tokens never qualify as creature cards in graveyards or
cards in hands. Predefined Treasure/Food/Clue and other ability-bearing tokens
stay gated until their abilities exist.

Ceased tokens leave reusable slots in an owned free list. Every departure,
cessation and new creation changes the generation, so previously bound targets
cannot alias replacements. Physical card slots remain stable. Snapshots copy
both slots and free lists, giving identical identities on replay. Memory scales
with the maximum simultaneous token population rather than every historical
creation. State-size accounting includes initialized slots and the free list.

Observation schema **8** adds `ObjectView.token` (null for actual cards), with
name, subtypes and colors. Factory identity, power/toughness and keywords remain
in the object's existing fields. The `Outside` sentinel represents vacant slots,
not a game zone, and never appears in observations or legal targets.

The compiler admits only complete fixed token clauses and entire supported
cards. Dynamic counts, copies, unsupported abilities, replacements, additional
costs and residual rules text remain gated. Current coverage is 142/5,164
Standard identities plus six legacy regression cards and 23 token factories.

Rules: [September 25, 2026 Comprehensive Rules](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.pdf),
especially 111.2–111.8 and 704.3.
