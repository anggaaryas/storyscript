# .StoryScript Language Specification

## 1. Global Initialization
Before any scenes are parsed, the engine must define global variables, load actor assets into memory, and explicitly define the game's entry point. This is strictly handled in the reserved `* INIT` block.

**Syntax Rules:**
* Exactly one `* INIT { ... }` block must exist in the entire script.
* Must be the absolute first block evaluated by the compiler.
* Handles global variable declaration (`$`) with explicit type annotation.
* Handles character registration (`@actor`) using block-based dictionary syntax.
* Optional child include manifest via `@include ["path/to/file.StoryScript", ...]`.
* **Mandatory:** Must contain exactly one `@start` directive pointing to the first scene.

**Typed Variable Declaration:**
* Syntax: `$name as <type> = <expr>`
* Supported `<type>`: `integer`, `string`, `boolean`, `decimal`, `array<integer|string|boolean|decimal>`
* Variable type is immutable after declaration.

```plaintext
* INIT {
    // Variable Registration
    $system_stability as integer = 45;
    $manual_override as boolean = false;
    $pilot_name as string = "Teona";
    $stability_ratio as decimal = 0.45;

    // Standard Actor (With Portraits)
    @actor TEO "Teona" {
        neutral -> "teo_neutral.png"
        calm -> "teo_calm.png"
        focus -> "teo_focus.png"
    }
    @actor GIP "Gippie" {
        default -> "gip_smile.png"
        playful -> "gip_wink.png"
        alert -> "gip_alert.png"
    }

    // Actor without portrait
    @actor SYS "System";

    // Explicit Entry Point
    @start server_core_hub;
}
```

### Child Modules and Atomic Include

StoryScript supports compile-time child modules to split large narratives safely.

**Root manifest syntax (INIT only):**

```plaintext
* INIT {
        @include [
                "modules/minigame_hack.StoryScript",
                "chapters/chapter_1.StoryScript"
        ];
}
```

**Child contract syntax (required in every included file):**

```plaintext
* REQUIRE {
    $system_stability as integer;
    $has_admin_key as boolean;
    @actor TEO [ focus ];
    @actor GIP [ alert ];
}
```

Child include semantics:
* `@include` is valid only inside root `* INIT`.
* Include paths are resolved relative to the root file path.
* Duplicate include path strings in one manifest are compile-time invalid.
* Included child files must contain exactly one `* REQUIRE` block.
* Included child files must not contain `* INIT`.
* Child files must not declare `@start`; root `* INIT` is the only entrypoint owner.
* Child scenes are merged after root parsing and then validated with normal scene/link/type rules.

---

## 2. The Scene Lifecycle
A standard scene is defined using `* <scene_label> { ... }`. Every scene operates on a strict, sequential two-phase lifecycle. Blocks must appear in exact order.

### Phase 1: `#PREP` (Execution Phase)
The invisible backend phase. The parser executes all math, updates state arrays, and queues engine assets instantly before rendering anything to the screen. 

* **Allowed Tokens:** `$`, `@bg`, `@bgm`, `@sfx`, standalone function calls (`name(...)`), `if`/`else if`/`else`.
* **Variable Declaration:** Typed local declarations are allowed only in `#PREP` (`$name as <type> = <expr>`).
* **Forbidden Tokens:** `"Narrative text"`, `ActorID()`, standalone STORY output (`$var`), `@choice`, `@jump`, `@end`.

### Phase 2: `#STORY` (Rendering & Interaction Phase)
The player-facing phase. The UI sequentially renders text and dialogue. Execution pauses when requiring user input or a hard scene transition.

* **Allowed Tokens:** `"Narrative text"`, `ActorID()`, `if`/`else if`/`else`, `@choice`, `@jump`, `@end`, standalone variable output (`$var`), and **read-only** variable access (`$var`) inside expressions.
* **Forbidden Tokens:** `@bg`, `@bgm`, `@sfx`, and variable assignment/mutation (`=`, `+=`, `-=`, etc.).
* **Strict Rule:** Every reachable execution path in `#STORY` must terminate with a transition directive (`@choice`, `@jump`, or `@end`) as its final executed token.
* **Compiler Enforcement:** The compiler must statically verify this termination rule for `#STORY` control flow.

### Scene Block Requirements
Each scene must follow these structural rules:

* `#PREP` is optional.
* `#PREP` may appear at most once.
* `#STORY` is mandatory and must appear exactly once.
* If `#PREP` exists, it must appear before `#STORY`.
* A scene definition ends when its block closing brace `}` is reached. Runtime script termination is handled by `@end` inside `#STORY`.

---

## 3. Syntax Reference

### Variables & Logic
Standard C-style conditionals are supported in both `#PREP` and `#STORY` blocks. Variables must be prefixed with `$`.

### Logic Functions
StoryScript supports top-level user-defined logic blocks for reusable PREP-side computation.

Syntax:

```plaintext
logic apply_damage($amount as integer) {
    $system_stability = $system_stability - $amount;
}

logic get_variance($modifier as integer) -> integer {
    $total as integer = $system_stability + $modifier;
    return $total;
}
```

Rules:
* Logic declarations are allowed only at top-level (same level as `* INIT` and scenes).
* Parameter syntax is mandatory typed form: `$name as <type>`.
* Optional return annotation uses `-> <type>`.
* Logic calls are valid only in `#PREP` and inside logic bodies.
* Logic calls are forbidden in `* INIT` expressions and in `#STORY` expressions.
* `return` is valid only inside logic bodies.
* Void logic (no `-> <type>`) must not return a value.
* Typed logic (`-> <type>`) must return a compatible value on all reachable paths.
* Recursion is forbidden in v1 (direct or indirect call cycles fail compile-time).
* Multiple `@bg` / `@bgm` / `@sfx` directives inside logic or PREP are valid and execute in source order.
* For `@bg` and `@bgm`, later directives override earlier state (`@bgm STOP` clears BGM).


### Branching Syntax
Use C-style branch chains in both `#PREP` and `#STORY`:

```plaintext
if (<condition_a>) {
    ...
} else if (<condition_b>) {
    ...
} else {
    ...
}
```

Execution semantics:
* Branches are evaluated top-to-bottom.
* Only the first branch with a `true` condition executes.
* `else` executes only when all previous conditions are `false`.
* Parentheses around each `if`/`else if` condition are mandatory.

**Mutation Rule:**
* In `#PREP`: variable reads and writes are allowed.
* In `#STORY`: variable reads are allowed, but writes are forbidden.
* Assignment in `#PREP` must preserve declared variable type (`decimal` may accept integer assignment).

**Scene-Local Declaration Rule:**
* In `#PREP`, local declaration syntax is `$name as <type> = <expr>`.
* Local variables are visible only inside the declaring scene (`#PREP` and `#STORY` of that scene).
* Local variables are reset when the scene is entered again (including loop/re-entry via `@jump` or `@choice`).
* Local declaration in `#STORY` is forbidden.
* Local names must not collide with global names.

**Arithmetic Operators:**
* Supported numeric operators: `+`, `-`, `*`, `/`, `%`.
* `*` and `/` follow normal precedence above `+` and `-`.
* Integer with integer arithmetic returns integer.
* Mixed integer/decimal arithmetic returns decimal.
* `%` (modulo) is valid only for integer operands.

**Array Type and Literals:**
* Array type syntax: `array<scalar_type>` where `scalar_type` is one of `integer`, `string`, `boolean`, `decimal`.
* Nested arrays are not supported.
* Array literal syntax: `[value_1, value_2, ...]`.
* Empty array literal `[]` is valid only when target array type is known from context.
* Array literal elements must resolve to one scalar element type (mixed integer/decimal literals infer decimal element type).
* Array arguments in collection functions must be either `$variable` or array literal.
* Scalar arguments (`value`, `index`, `count`, `string_separator`) in collection functions must be either literals or `$variable`.

**Built-in Functions:**
* `abs(x)`:
    * Requires exactly one numeric argument.
    * Returns integer for integer input, decimal for decimal input.
* `rand()`:
    * Valid only in typed assignment context.
    * Uses assignment target type:
        * integer target -> full integer range
        * decimal target -> decimal value in `0.0..1.0`
* `rand(min, max)`:
    * Valid only in typed assignment context.
    * Inclusive bounds (`[min, max]`).
    * Integer target requires integer bounds.
    * Decimal target accepts integer or decimal bounds (integer bounds widen to decimal).
* `pick(array)`:
    * Requires one non-empty array argument.
    * Returns one random element from the array.
* `pick(count, array)`:
    * `count` must be integer.
    * Returns `array<type>` with `count` members sampled by unique random index.
    * `pick(0, array)` returns an empty `array<type>`.
    * Runtime fails when `count > array_size(array)`.
* `array_push(array, value)`:
    * Mutates array by appending value.
    * Returns void.
    * Allowed in `#PREP` statement call form only.
* `array_pop(array)`:
    * Removes and returns the last array element.
    * Runtime fails for empty array.
* `array_strip(array, value)`:
    * Removes all elements equal to value.
    * Returns void.
    * Allowed in `#PREP` statement call form only.
* `array_clear(array)`:
    * Removes all elements (result `[]`).
    * Returns void.
    * Allowed in `#PREP` statement call form only.
* `array_contains(array, value)`:
    * Returns boolean.
    * For `array<decimal>`, integer probe value is allowed (widened to decimal).
* `array_size(array)`:
    * Returns integer array length.
* `array_join(array, string_separator)`:
    * Returns string by joining element text with separator.
* `array_get(array, index)`:
    * `index` is zero-based integer.
    * Returns element at index.
    * Runtime fails when index is out of range.
* `array_insert(array, index, value)`:
    * Inserts value at zero-based index.
    * Returns void.
    * Allowed in `#PREP` statement call form only.
    * Runtime fails when index is out of range for insertion.
* `array_remove(array, index)`:
    * Removes and returns element at zero-based index.
    * Runtime fails when index is out of range.

```plaintext
if ($system_stability <= 30) {
    $critical_warning = true;
} else if ($system_stability <= 50) {
    $critical_warning = true;
} else {
    $critical_warning = false;
}
```

### Loop Syntax
StoryScript supports explicit loop constructs in both `#PREP` and `#STORY`:

```plaintext
for ($item in snapshot $array) {
    ...
}

repeat (count) {
    ...
}
```

Loop control statements are also supported inside loop bodies:

```plaintext
break;
continue;
```

Loop semantics:
* `for ($item in snapshot $array)` iterates a snapshot copy of `$array` captured at loop entry.
* Mutating `$array` inside the loop body does not change the current iteration sequence.
* `$item` is an implicit loop-local variable scoped to the loop body.
* `$item` is read-only and assignment to `$item` is compile-time invalid.
* `repeat(count)` accepts only integer literal or `$integer_variable` count source.
* `repeat(count)` requires `count > 0`.
* `break` and `continue` are valid only inside loop bodies and always target the nearest loop.
* Inside `@choice`, loop entries support nested `if`, `repeat`, and snapshot `for` groups in any combination.

Explicitly not supported:
* General `for` loop forms (index-init/condition/step syntax).
* Labeled `break`/`continue`.

### Variable Read Output & Interpolation

StoryScript supports two read-only variable rendering forms:

* **Standalone variable output (only in `#STORY`):**
    * Syntax: `$variable_name` (optional trailing `;`).
    * Must be exactly `$name`; arithmetic/comparison tails are invalid in this statement form.
* **Inline interpolation in string literals (all phases):**
    * Placeholder syntax: `${variable_name}`.
    * Supported in all string literal contexts: narration/dialogue text, choice labels, directive paths, actor display names, and string expressions.
    * Placeholder names follow normal identifier rules (`[A-Za-z_][A-Za-z0-9_]*`).
    * Literal dollar uses escape `\$`.

Interpolation and standalone variable output are read-only and evaluated against current runtime variable state at execution time.

```plaintext
#STORY
"Apple count: ${apple_count}"
"Price tag stays literal: \$5"
$apple_count
```

### Engine Directives (Only in `#PREP`)
Directives tell the visual/audio engine what to queue.

| Directive | Purpose | Syntax | Example |
| :--- | :--- | :--- | :--- |
| **@bg** | Loads a background image. | `@bg "<path>"` | `@bg "server_room.png"` |
| **@bgm** | Plays looping background music, or stops it. | `@bgm "<path>"` or `@bgm STOP` | `@bgm "tense_hum.wav"` |
| **@sfx** | Plays a one-shot sound effect. | `@sfx "<path>"` | `@sfx "spark.wav"` |

### Dialogue & Narration (Only in `#STORY`)
Narration is handled via standard string literals. Dialogue utilizes the registered Actor IDs from the `INIT` block. 

The parser supports two dialogue forms for any registered actor ID.

* **Portrait Form:** `ActorID(<emotion_key>, <Position>): "..."` renders sprite and text. Valid positions are `Left`, `Center`, `Right` (or `L`, `C`, `R`).
* **Name-Only Form:** `ActorID: "..."` renders text only and suppresses sprite rendering.
* **Constraint:** If portrait form is used, `<emotion_key>` must exist on that actor's portrait map.
* **Constraint:** If an actor was declared without a portrait map, portrait form is invalid for that actor.

```plaintext
"The main console flashes red."

// Renders sprite and text
TEO(focus, Left): "We need to isolate the memory leak."

// Renders text only (if using a portrait-less setup or quick text)
GIP: "On it, boss!"
```

### Navigation & Termination Directives (Only in `#STORY`)
These directives handle transitioning out of the current scene and must be the final executed token on each reachable `#STORY` path.

**@choice**
Halts the engine and renders a user-selectable menu. Options map to the next scene via `->`. Supports nested conditionals.

If all options are filtered out after conditional evaluation, the engine raises a runtime error (`ChoiceExhausted`) and stops execution.

Nested `if`, `repeat`, and `for ($item in snapshot $array)` entry groups are valid inside `@choice` and are expanded in source order at runtime.
If expansion produces more than 9 options, runtime fails with `R_CHOICE_OPTION_CAP_EXCEEDED`.

```plaintext
@choice {
    "Run diagnostic sweep" -> scene_diagnostic;
    
    if ($manual_override == true) {
        "Force hard reboot" -> scene_reboot;
    }
}
```

**@jump**
Automatically transitions to the next scene without user input. Used for script chunking, invisible logic routing hubs, or seamless cinematic transitions.

```plaintext
"The servers finally quiet down into a steady hum."
@jump scene_rest_period;
```

**@end**
Terminates script execution immediately. Use this when the current scene is a terminal/final scene.

```plaintext
"The reactor cools to silence."
@end;
```

### Compile-Time Validation Rules
The compiler must fail the script when any of the following is true:

* Any lexical or syntax error exists.
* The script contains zero or multiple `* INIT` blocks.
* `* INIT` is not the first top-level block.
* `* INIT` contains zero or multiple `@start` directives.
* Any `@include` appears outside root `* INIT`.
* Any include path listed in root `@include` cannot be resolved/read.
* Any include path string is duplicated in one `@include` manifest.
* Any included child file contains `* INIT`.
* Any included child file does not contain exactly one `* REQUIRE` block.
* Any child `* REQUIRE` variable is missing in root `* INIT`.
* Any child `* REQUIRE` variable type does not match the root declaration type.
* Any child `* REQUIRE` declaration includes initializer value (`=` expression).
* Any child `* REQUIRE` actor ID is missing in root `* INIT`.
* Any child `* REQUIRE` emotion key is missing in the referenced root actor portrait map.
* Duplicate scene labels exist.
* Duplicate actor IDs exist.
* Duplicate emotion keys exist inside an actor portrait map.
* Duplicate global variable declarations exist in `* INIT`.
* `@start` points to a non-existent scene.
* Any `@jump` target does not exist.
* Any `@choice` option target does not exist.
* Any scene has invalid phase structure (missing `#STORY`, repeated `#PREP`, repeated `#STORY`, or `#PREP` placed after `#STORY`).
* Any statement appears in a forbidden phase (for example: `@bg` in `#STORY`, dialogue/narration in `#PREP`, transition directives in `#PREP`, or variable mutation in `#STORY`).
* Any dialogue line references an unknown actor ID.
* Any portrait-form dialogue has invalid parameter shape (must be exactly `<emotion_key>, <Position>`).
* Any portrait-form dialogue uses an invalid position token.
* Any portrait-form dialogue uses an unknown emotion key.
* Any portrait-form dialogue targets an actor declared without portraits.
* Any variable is read before declaration.
* Any variable assignment targets an undeclared variable.
* Any local variable declaration appears outside `#PREP`.
* Any scene declares duplicate local variable names.
* Any local variable declaration collides with an existing global variable name.
* Any variable assignment violates declared type.
* Any compound assignment (`+=`, `-=`) targets non-numeric variable types.
* Any expression uses incompatible operand types for its operator.
* Any modulo expression (`%`) uses non-integer operands.
* Any expression calls an unknown function name.
* Any duplicate logic declaration name exists.
* Any logic declaration reuses a built-in function name.
* Any logic declaration contains duplicate parameter names.
* Any logic call appears in `* INIT` expression context.
* Any logic call appears in `#STORY` expression context.
* Any logic call has arity mismatch or incompatible argument types.
* Any `return` appears outside a logic body.
* Any typed logic function has a reachable path without `return`.
* Any logic `return` value is incompatible with declared return type.
* Any void logic function returns a value.
* Any direct or indirect recursive logic call cycle exists.
* Any `abs()` call uses non-numeric argument or wrong arity.
* Any `rand()`/`rand(min,max)` call is used without typed assignment context.
* Any `rand(min,max)` call has incompatible bound types for assignment target type.
* Any `array<type>` declaration uses unsupported element type.
* Any empty array literal `[]` appears without inferable target array type.
* Any array literal contains mixed incompatible element types.
* Any nested array literal appears.
* Any collection function scalar argument (`value`, `index`, `count`, `string_separator`) is not a literal or `$variable`.
* Any collection function array argument is not `$variable` or array literal.
* Any collection function argument type is incompatible with function contract.
* Any void collection function (`array_push`, `array_strip`, `array_clear`, `array_insert`) is used in expression context.
* Any mutating collection function is used in `#STORY`.
* Any `pick()` call has wrong arity, invalid argument shape/type, or empty one-argument literal.
* Any condition expression (`if`, `else if`, `@choice if`) is not boolean.
* Any `else if` branch that does not follow `else if (<expr>) { ... }` syntax.
* Any `for` header that is not exactly `for ($item in snapshot $array) { ... }`.
* Any `for` snapshot source variable that is undeclared or not an array variable.
* Any `for` iterator variable name that collides with an existing global/local scene variable.
* Any assignment targeting a loop iterator variable.
* Any `@choice` expansion that produces more than 9 options at runtime.
* Any `repeat(count)` count source that is not integer literal or `$integer_variable`.
* Any `repeat(count)` integer literal where `count <= 0`.
* Any `break` or `continue` statement used outside a loop body.
* Any interpolation placeholder is malformed (for example: `${`, `${}`, `${1bad}`, `${name`).
* Any constant-folded `@choice` block is provably empty at compile time.
* Any reachable `#STORY` path can complete without executing `@choice`, `@jump`, or `@end`.

### Standard Diagnostic Codes
Use the following stable diagnostic codes in compiler/runtime output.

Diagnostic code naming:
* `E_*` = compile-time error
* `W_*` = compile-time warning
* `R_*` = runtime error

#### Compile-Time
| Code | Trigger |
| :--- | :--- |
| `E_SYNTAX` | Any lexical/tokenization/parser error, including malformed interpolation placeholder syntax. |
| `E_INIT_COUNT` | The script contains zero or multiple `* INIT` blocks. |
| `E_INIT_ORDER` | `* INIT` is not the first top-level block. |
| `E_START_COUNT` | `* INIT` contains zero or multiple `@start` directives. |
| `E_INCLUDE_FILE_NOT_FOUND` | Include path in `@include` cannot be resolved/read. |
| `E_INCLUDE_DUPLICATE_PATH` | Duplicate include path strings are declared in one `@include` manifest. |
| `E_INCLUDE_CHILD_INIT_FORBIDDEN` | Included child file contains `* INIT`. |
| `E_REQUIRE_COUNT` | Included child file does not contain exactly one `* REQUIRE` block. |
| `E_REQUIRE_VARIABLE_MISSING` | Child `* REQUIRE` variable does not exist in root `* INIT`. |
| `E_REQUIRE_ACTOR_MISSING` | Child `* REQUIRE` actor ID does not exist in root `* INIT`. |
| `E_REQUIRE_EMOTION_MISSING` | Child `* REQUIRE` emotion key does not exist in root actor portrait map. |
| `E_SCENE_DUPLICATE` | Duplicate scene labels exist. |
| `E_ACTOR_DUPLICATE` | Duplicate actor IDs exist. |
| `E_EMOTION_DUPLICATE` | Duplicate emotion keys exist inside one actor portrait map. |
| `E_GLOBAL_DUPLICATE` | Duplicate global variable declarations exist in `* INIT`. |
| `E_LOCAL_DUPLICATE` | Duplicate local variable declarations exist within one scene scope. |
| `E_VARIABLE_SCOPE_CONFLICT` | A local variable declaration collides with a global variable name. |
| `E_START_TARGET_MISSING` | `@start` points to a non-existent scene. |
| `E_JUMP_TARGET_MISSING` | Any `@jump` target does not exist. |
| `E_CHOICE_TARGET_MISSING` | Any `@choice` option target does not exist. |
| `E_SCENE_STRUCTURE` | A scene has invalid phase structure (`#STORY` missing/repeated, `#PREP` repeated, or `#PREP` after `#STORY`). |
| `E_PHASE_TOKEN_FORBIDDEN` | A statement/token is used in a forbidden phase (`#PREP` or `#STORY`). |
| `E_ACTOR_UNKNOWN` | A dialogue line references an unknown actor ID. |
| `E_DIALOGUE_SHAPE_INVALID` | Portrait-form dialogue does not use exactly `<emotion_key>, <Position>`. |
| `E_POSITION_INVALID` | Portrait-form dialogue uses an invalid position token. |
| `E_EMOTION_UNKNOWN` | Portrait-form dialogue uses an unknown emotion key. |
| `E_PORTRAIT_MODE_INVALID` | Portrait-form dialogue targets an actor declared without portraits. |
| `E_VARIABLE_UNDECLARED_READ` | A variable is read before declaration or outside its scope (including `${var}` interpolation and standalone STORY output `$var`). |
| `E_VARIABLE_UNDECLARED_WRITE` | A variable assignment targets an undeclared variable or a variable outside its scope. |
| `E_VARIABLE_TYPE_MISMATCH` | Variable assignment or initializer type is incompatible with its declared variable type. |
| `E_VARIABLE_COMPOUND_ASSIGN_INVALID` | Compound assignment (`+=`, `-=`) is used on non-numeric variable type. |
| `E_EXPRESSION_TYPE_INVALID` | Expression operator is used with incompatible operand types. |
| `E_FUNCTION_UNKNOWN` | Function name is not recognized. |
| `E_FUNCTION_DUPLICATE` | Duplicate logic function name exists, or a logic name conflicts with a reserved built-in function name. |
| `E_FUNCTION_PARAM_DUPLICATE` | Logic declaration contains duplicate parameter names. |
| `E_FUNCTION_ARITY_INVALID` | Function call has invalid argument count. |
| `E_FUNCTION_CONTEXT_INVALID` | Function call is used in an invalid context (for example: `rand()` outside typed assignment context). |
| `E_FUNCTION_ARGUMENT_INVALID` | Function argument type or shape is invalid. |
| `E_FUNCTION_RETURN_MISSING` | Typed logic function does not return on all reachable paths. |
| `E_RETURN_CONTEXT_INVALID` | `return` is used outside a logic body. |
| `E_RETURN_TYPE_MISMATCH` | Logic return expression is incompatible with declared return type, or void/typed return form is invalid. |
| `E_FUNCTION_RECURSION_FORBIDDEN` | Direct or indirect recursive logic call cycle is detected. |
| `E_RANGE_INVALID` | Function range input is invalid (for example: `rand(min,max)` where `min > max`). |
| `E_LIST_EMPTY` | Function requires non-empty list argument but received empty list. |
| `E_CONDITION_TYPE_INVALID` | Condition expression does not evaluate to boolean. |
| `E_CHOICE_STATIC_EMPTY` | `@choice` is provably empty after compile-time constant folding. |
| `E_STORY_UNTERMINATED_PATH` | A reachable `#STORY` path can fall through without `@choice`, `@jump`, or `@end`. |
| `E_LOOP_CONTROL_OUTSIDE_LOOP` | `break`/`continue` is used outside a loop body. |
| `E_LOOP_ITERATOR_READ_ONLY` | Assignment targets a read-only loop iterator variable. |

#### Scene-Local Variable Diagnostic Mapping
| Rule ID | Validation Condition | Diagnostic Code | Rationale |
| :--- | :--- | :--- | :--- |
| `LS001` | Local declaration appears outside `#PREP`. | `E_PHASE_TOKEN_FORBIDDEN` | Phase legality already standardized by existing forbidden-token rule. |
| `LS002` | Two local declarations with the same name appear in one scene. | `E_LOCAL_DUPLICATE` | Distinguishes local redeclaration from global redeclaration. |
| `LS003` | A local declaration name matches a global declaration name. | `E_VARIABLE_SCOPE_CONFLICT` | Enforces no-shadowing policy with explicit scope-conflict code. |
| `LS004` | A local variable is read outside its declaring scene. | `E_VARIABLE_UNDECLARED_READ` | Out-of-scope reads are treated as undeclared in the current scene scope. |
| `LS005` | A local variable is written outside its declaring scene. | `E_VARIABLE_UNDECLARED_WRITE` | Out-of-scope writes are treated as undeclared in the current scene scope. |

#### Else-If Diagnostic Mapping
| Rule ID | Validation Condition | Diagnostic Code | Rationale |
| :--- | :--- | :--- | :--- |
| `F001` | `else if` branch does not match `else if (<expr>) { ... }` token shape. | `E_SYNTAX` | Malformed branch chain is a parser-level syntax failure. |
| `F002` | Any `else if` condition expression is non-boolean. | `E_CONDITION_TYPE_INVALID` | Branch conditions require explicit boolean typing. |
| `F003` | A reachable `#STORY` path through an `if`/`else if` chain can fall through without `@choice`, `@jump`, or `@end`. | `E_STORY_UNTERMINATED_PATH` | Existing story termination invariant applies to every reachable branch arm. |

#### Loop Diagnostic Mapping
| Rule ID | Validation Condition | Diagnostic Code | Rationale |
| :--- | :--- | :--- | :--- |
| `LOOP001` | `for` header does not match `for ($item in snapshot $array) { ... }`. | `E_SYNTAX` | Loop syntax contract is explicit and parser-enforced. |
| `LOOP002` | `for` snapshot source is undeclared in current scope. | `E_VARIABLE_UNDECLARED_READ` | Snapshot source is a variable read operation. |
| `LOOP003` | `for` snapshot source is declared but not an array variable. | `E_FUNCTION_ARGUMENT_INVALID` | Snapshot iterable contract requires array variable source. |
| `LOOP004` | Loop iterator name collides with an existing global or scene-local variable. | `E_VARIABLE_SCOPE_CONFLICT` or `E_LOCAL_DUPLICATE` | Reuses existing scope and duplicate-name diagnostics. |
| `LOOP005` | Assignment targets loop iterator variable. | `E_LOOP_ITERATOR_READ_ONLY` | Iterator is body-scoped read-only binding. |
| `LOOP006` | `repeat(count)` source is not integer literal or `$integer_variable`. | `E_SYNTAX` or `E_FUNCTION_ARGUMENT_INVALID` | Parser enforces source shape and validator enforces integer variable type. |
| `LOOP007` | `repeat(count)` literal is `<= 0`. | `E_RANGE_INVALID` | Literal count validity is deterministically provable at compile-time. |
| `LOOP008` | `break`/`continue` appears outside loop body. | `E_LOOP_CONTROL_OUTSIDE_LOOP` | Loop control statements are nearest-loop scoped only. |
| `LOOP009` | Runtime repeat count from variable evaluates to `<= 0`. | `R_REPEAT_COUNT_INVALID` | Variable-driven count validity is runtime-dependent. |
| `LOOP010` | Runtime STORY loop emits invalid terminal behavior (zero terminal on exit or multiple terminal triggers). | `R_STORY_LOOP_TERMINATION_INVALID` | Deferred STORY loop termination constraints are enforced at runtime. |
| `LOOP011` | Runtime `@choice` expansion emits more than 9 options after nested entry expansion. | `R_CHOICE_OPTION_CAP_EXCEEDED` | Player choice input is bounded to 1-9; oversized menus must fail deterministically. |

#### Include/REQUIRE Diagnostic Mapping
| Rule ID | Validation Condition | Diagnostic Code | Rationale |
| :--- | :--- | :--- | :--- |
| `INC001` | Include path cannot be read from root `@include` manifest. | `E_INCLUDE_FILE_NOT_FOUND` | Child module cannot be compiled if source file is unavailable. |
| `INC002` | Same include path string appears more than once in one manifest. | `E_INCLUDE_DUPLICATE_PATH` | Prevents accidental double-merge of child scenes. |
| `INC003` | Included child file declares `* INIT`. | `E_INCLUDE_CHILD_INIT_FORBIDDEN` | Root-only bootstrap ownership keeps global state deterministic. |
| `INC004` | Included child file has zero or multiple `* REQUIRE` blocks. | `E_REQUIRE_COUNT` | Child contract cardinality must be exactly one. |
| `INC005` | Child REQUIRE variable is absent in root INIT. | `E_REQUIRE_VARIABLE_MISSING` | Child variable dependency must be declared by root bootstrap. |
| `INC006` | Child REQUIRE actor ID is absent in root INIT. | `E_REQUIRE_ACTOR_MISSING` | Child dialogue/render dependency must exist before execution. |
| `INC007` | Child REQUIRE emotion key is absent in root actor portrait map. | `E_REQUIRE_EMOTION_MISSING` | Child portrait-form dialogue dependency must be guaranteed at compile-time. |
| `INC008` | Child REQUIRE declaration includes initializer value. | `E_SYNTAX` | REQUIRE blocks describe dependency contracts, not runtime initialization. |

#### Array Collection Diagnostic Mapping
| Rule ID | Validation Condition | Diagnostic Code | Rationale |
| :--- | :--- | :--- | :--- |
| `ARR001` | Scalar argument source for `value`/`index`/`count`/`string_separator` is not literal or `$variable`. | `E_FUNCTION_ARGUMENT_INVALID` | Restricts collection scalar arguments to deterministic source shapes. |
| `ARR002` | Array argument source is not `$variable` or array literal. | `E_FUNCTION_ARGUMENT_INVALID` | Prevents unsupported expression-shape array arguments. |
| `ARR003` | Argument shape is valid but inferred type is incompatible with function contract. | `E_FUNCTION_ARGUMENT_INVALID` | Keeps type failures explicit at function boundary. |
| `ARR004` | Empty array literal `[]` appears without inferable target array type context. | `E_FUNCTION_CONTEXT_INVALID` | Empty literals need contextual type to resolve element type. |
| `ARR005` | Mutating collection call is used in `#STORY`. | `E_FUNCTION_CONTEXT_INVALID` | Preserves `#STORY` read-only mutation model. |
| `ARR006` | `pick(count, array)` count argument is not integer. | `E_FUNCTION_ARGUMENT_INVALID` | Count semantics require integer cardinality. |
| `ARR007` | `array_get`/`array_insert`/`array_remove` index argument is not integer. | `E_FUNCTION_ARGUMENT_INVALID` | Index semantics are zero-based integer only. |
| `ARR008` | `array_join(array, string_separator)` separator argument is not string. | `E_FUNCTION_ARGUMENT_INVALID` | Join separator contract is string-only. |
| `ARR009` | `array_contains(array<decimal>, value)` receives non-numeric value. | `E_FUNCTION_ARGUMENT_INVALID` | Decimal arrays support integer widening but still require numeric probe. |
| `ARR010` | Runtime `pick(count, array)` receives count greater than array size (or negative). | `R_ARRAY_SAMPLE_COUNT_INVALID` | Sampling cardinality cannot exceed available members. |
| `ARR011` | Runtime array index operation is outside valid bounds. | `R_ARRAY_INDEX_OUT_OF_RANGE` | Ensures deterministic failure for invalid index access. |
| `ARR012` | Runtime pop/get/remove operation requires element but array is empty. | `R_ARRAY_EMPTY` | Empty-collection element access/removal is invalid. |

#### Logic Function Diagnostic Mapping
| Rule ID | Validation Condition | Diagnostic Code | Rationale |
| :--- | :--- | :--- | :--- |
| `LOGIC001` | Duplicate logic declaration name, or logic name collides with built-in function name. | `E_FUNCTION_DUPLICATE` | Ensures deterministic call resolution and protects reserved built-ins. |
| `LOGIC002` | A logic declaration repeats a parameter name. | `E_FUNCTION_PARAM_DUPLICATE` | Prevents ambiguous parameter binding. |
| `LOGIC003` | Logic call appears outside PREP/logic body (for example in INIT or STORY expression). | `E_FUNCTION_CONTEXT_INVALID` | Enforces phase-bounded side-effect model. |
| `LOGIC004` | Logic call argument count does not match declaration arity. | `E_FUNCTION_ARITY_INVALID` | Reuses standard function arity contract. |
| `LOGIC005` | Logic call argument type is incompatible with parameter type. | `E_FUNCTION_ARGUMENT_INVALID` | Reuses standard function argument type contract. |
| `LOGIC006` | `return` appears outside logic body. | `E_RETURN_CONTEXT_INVALID` | Keeps return semantics scoped to logic bodies only. |
| `LOGIC007` | Typed logic has a reachable path without `return`. | `E_FUNCTION_RETURN_MISSING` | Enforces complete return contract for typed logic. |
| `LOGIC008` | Return expression/type form mismatches declared return contract. | `E_RETURN_TYPE_MISMATCH` | Separates return-contract typing from generic expression typing. |
| `LOGIC009` | Direct or indirect recursive logic cycle is present. | `E_FUNCTION_RECURSION_FORBIDDEN` | v1 recursion is intentionally compile-time forbidden. |

#### Compile-Time Warnings
| Code | Trigger |
| :--- | :--- |
| `W_CHOICE_POSSIBLY_EMPTY` | `@choice` cannot be proven empty at compile time, but may evaluate to no options at runtime. |

#### Runtime
| Code | Trigger |
| :--- | :--- |
| `R_CHOICE_EXHAUSTED` | All `@choice` options are filtered out after condition evaluation. |
| `R_ASSET_NOT_FOUND` | Referenced background, portrait, BGM, or SFX asset cannot be found at runtime. |
| `R_ASSET_LOAD_FAILED` | Asset exists but fails to load/decode at runtime. |
| `R_AUDIO_DEVICE_FAILURE` | Audio subsystem/device cannot initialize or play requested sound. |
| `R_SAVE_STATE_CORRUPT` | Save data is malformed or incompatible with expected schema. |
| `R_DIVIDE_BY_ZERO` | Division operation attempted with zero divisor at runtime. |
| `R_MODULO_BY_ZERO` | Modulo operation attempted with zero divisor at runtime. |
| `R_NUMERIC_OVERFLOW` | Numeric operation overflowed at runtime (for example: integer `abs()` overflow edge case). |
| `R_ARRAY_EMPTY` | Array operation requires at least one element but array is empty at runtime. |
| `R_ARRAY_INDEX_OUT_OF_RANGE` | Array index operation used index outside valid bounds at runtime. |
| `R_ARRAY_SAMPLE_COUNT_INVALID` | `pick(count, array)` requested count is negative or larger than array size at runtime. |
| `R_REPEAT_COUNT_INVALID` | `repeat(count)` evaluated runtime count is `<= 0`. |
| `R_STORY_LOOP_TERMINATION_INVALID` | Deferred STORY loop termination constraints were violated at runtime. |
| `R_CHOICE_OPTION_CAP_EXCEEDED` | `@choice` nested entry expansion produced more than 9 options at runtime. |

Backward compatibility:
* `ChoiceExhausted` is retained as a legacy alias for `R_CHOICE_EXHAUSTED`.

### Diagnostic Message Format
All diagnostics (compile-time, warning, runtime) must be emitted in a consistent structure.

Required fields:
* `code`: Diagnostic code (`E_*`, `W_*`, `R_*`).
* `message`: Human-readable explanation.
* `phase`: One of `LEX`, `PARSE`, `VALIDATION`, `PREP`, `STORY`, `RUNTIME`.
* `scene`: Scene label when applicable; otherwise `INIT` or `GLOBAL`.
* `line`: 1-based source line when source-mapped.
* `column`: 1-based source column when source-mapped.

Canonical text format:
* `<code> [<phase>] <scene>:<line>:<column> <message>`

Canonical JSON format:
```json
{
    "code": "E_STORY_UNTERMINATED_PATH",
    "message": "Reachable #STORY path can fall through without @choice, @jump, or @end.",
    "phase": "VALIDATION",
    "scene": "server_core_hub",
    "line": 42,
    "column": 5
}
```

Formatting rules:
* `line` and `column` may be `0` when no exact source location exists (for example: deserialized runtime state errors).
* Multiple diagnostics must be sorted by ascending `(line, column, code)` within the same file.
* The compiler should continue reporting additional compile-time diagnostics after the first error when safe recovery is possible.

### `#STORY` Termination Analysis
The compiler must perform static control-flow analysis on each `#STORY` block.

* If a condition cannot be proven constant at compile time, both branches are treated as reachable.
* In `if`/`else if` chains, each non-constant arm is treated as reachable unless pruned by constant folding.
* A path is considered terminated only when its final executable statement is `@choice`, `@jump`, or `@end`.
* If the final executable statement is `for (...)` or `repeat(...)`, compile-time termination may be deferred to runtime.
* If any reachable path can fall through to the end of `#STORY` without a transition, compilation fails.
* Deferred loop termination checks at runtime must raise `R_STORY_LOOP_TERMINATION_INVALID` when a loop exits without terminal emission or would trigger terminal directives multiple times.
* Branches proven unreachable by constant folding do not need to terminate.

### Statement Terminators
Semicolons are optional.

* A statement may end with `;` or a newline.
* Trailing semicolons are accepted and ignored by the parser.
* Use one style consistently within a project to improve readability.

---

## 4. Comprehensive Parser Example

```plaintext
* INIT {
    $system_stability as integer = 40;
    $bypass_key as boolean = false;
    
    @actor TEO "Teona" {
        calm -> "teo_calm.png"
        focus -> "teo_focus.png"
    }
    
    @actor GIP "Gippie" {
        default -> "gip_smile.png"
        playful -> "gip_wink.png"
        alert -> "gip_alert.png"
    }
    
    @start server_core_hub;
}

* server_core_hub {
    
    #PREP
    @bg "core_chamber.png"
    
    if ($system_stability < 30) {
        @bgm "critical_alarm.wav"
    } else if ($system_stability < 50) {
        @bgm "warning_siren.wav"
    } else {
        @bgm "steady_hum.wav"
    }

    #STORY
    "Sparks shower from the ceiling as the primary coolant line shudders."

    TEO(calm, Left): "Variance is up by twelve percent. Gippie, run a sector scan."

    if ($system_stability < 30) {
        GIP(alert, Right): "Critical threshold crossed! Coolant pressure is collapsing!"
        TEO(focus, Left): "Emergency route now. Seal sector four and evacuate corridor B."
    } else if ($system_stability < 50) {
        GIP(alert, Right): "Yikes! Sector four is throwing a major temper tantrum, Teona!"
        TEO(focus, Left): "Understood. Let's patch the routing matrix before it cascades."
    } else {
        GIP(playful, Right): "Easy peasy! Just a little hiccup in sector four."
        TEO(focus, Left): "Understood. Let's patch the routing matrix before it cascades."
    }

    @choice {
        "Reroute coolant manually" -> scene_coolant_fix;
        "Deploy Gippie to the mainframe" -> scene_gippie_deploy;

        if ($bypass_key == true) {
            "Use Admin Bypass to purge cache" -> scene_admin_purge;
        }
    }
}

* scene_gippie_deploy {
    
    #PREP
    @bgm STOP
    @sfx "digital_dive.wav"
    $system_stability = $system_stability + 30;

    #STORY
    "Gippie's avatar dissolves into a stream of green code, diving directly into the terminal."
    
    GIP(playful, Center): "Wheeee! Sweeping out the bad sectors now!"
    TEO(calm, Left): "Good work. System is stabilizing."
    
    @jump scene_rest_period;
}

...
```
