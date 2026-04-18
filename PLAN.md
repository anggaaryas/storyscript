# .StoryScript Language Specification

## 1. Global Initialization
Before any scenes are parsed, the engine must define global variables, load actor assets into memory, and explicitly define the game's entry point. This is strictly handled in the reserved `* INIT` block.

**Syntax Rules:**
* Exactly one `* INIT { ... }` block must exist in the entire script.
* Must be the absolute first block evaluated by the compiler.
* Handles global variable declaration (`$`) with explicit type annotation.
* Handles character registration (`@actor`) using block-based dictionary syntax.
* **Mandatory:** Must contain exactly one `@start` directive pointing to the first scene.

**Typed Variable Declaration:**
* Syntax: `$name as <type> = <expr>`
* Supported `<type>`: `integer`, `string`, `boolean`, `decimal`
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

---

## 2. The Scene Lifecycle
A standard scene is defined using `* <scene_label> { ... }`. Every scene operates on a strict, sequential two-phase lifecycle. Blocks must appear in exact order.

### Phase 1: `#PREP` (Execution Phase)
The invisible backend phase. The parser executes all math, updates state arrays, and queues engine assets instantly before rendering anything to the screen. 

* **Allowed Tokens:** `$`, `@bg`, `@bgm`, `@sfx`, `if`/`else`.
* **Forbidden Tokens:** `"Narrative text"`, `ActorID()`, standalone STORY output (`$var`), `@choice`, `@jump`, `@end`.

### Phase 2: `#STORY` (Rendering & Interaction Phase)
The player-facing phase. The UI sequentially renders text and dialogue. Execution pauses when requiring user input or a hard scene transition.

* **Allowed Tokens:** `"Narrative text"`, `ActorID()`, `if`/`else`, `@choice`, `@jump`, `@end`, standalone variable output (`$var`), and **read-only** variable access (`$var`) inside expressions.
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

**Mutation Rule:**
* In `#PREP`: variable reads and writes are allowed.
* In `#STORY`: variable reads are allowed, but writes are forbidden.
* Assignment in `#PREP` must preserve declared variable type (`decimal` may accept integer assignment).

**Arithmetic Operators:**
* Supported numeric operators: `+`, `-`, `*`, `/`, `%`.
* `*` and `/` follow normal precedence above `+` and `-`.
* Integer with integer arithmetic returns integer.
* Mixed integer/decimal arithmetic returns decimal.
* `%` (modulo) is valid only for integer operands.

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
* `pick([candidate_1, candidate_2, ...])`:
    * Requires non-empty list literal argument.
    * In decimal assignment context, integer and decimal candidates are allowed (integer widens to decimal).
    * In other contexts, candidates must be type-compatible with usage context.

```plaintext
if ($system_stability <= 30) {
    $critical_warning = true;
} else {
    $critical_warning = false;
}
```

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
* Any variable assignment violates declared type.
* Any compound assignment (`+=`, `-=`) targets non-numeric variable types.
* Any expression uses incompatible operand types for its operator.
* Any modulo expression (`%`) uses non-integer operands.
* Any `abs()` call uses non-numeric argument or wrong arity.
* Any `rand()`/`rand(min,max)` call is used without typed assignment context.
* Any `rand(min,max)` call has incompatible bound types for assignment target type.
* Any `pick()` call has wrong arity, non-list argument, or empty list.
* Any condition expression (`if`, `@choice if`) is not boolean.
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
| `E_SCENE_DUPLICATE` | Duplicate scene labels exist. |
| `E_ACTOR_DUPLICATE` | Duplicate actor IDs exist. |
| `E_EMOTION_DUPLICATE` | Duplicate emotion keys exist inside one actor portrait map. |
| `E_GLOBAL_DUPLICATE` | Duplicate global variable declarations exist in `* INIT`. |
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
| `E_VARIABLE_UNDECLARED_READ` | A variable is read before declaration (including `${var}` interpolation and standalone STORY output `$var`). |
| `E_VARIABLE_UNDECLARED_WRITE` | A variable assignment targets an undeclared variable. |
| `E_VARIABLE_TYPE_MISMATCH` | Variable assignment or initializer type is incompatible with its declared variable type. |
| `E_VARIABLE_COMPOUND_ASSIGN_INVALID` | Compound assignment (`+=`, `-=`) is used on non-numeric variable type. |
| `E_EXPRESSION_TYPE_INVALID` | Expression operator is used with incompatible operand types. |
| `E_CONDITION_TYPE_INVALID` | Condition expression does not evaluate to boolean. |
| `E_CHOICE_STATIC_EMPTY` | `@choice` is provably empty after compile-time constant folding. |
| `E_STORY_UNTERMINATED_PATH` | A reachable `#STORY` path can fall through without `@choice`, `@jump`, or `@end`. |

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
* A path is considered terminated only when its final executable statement is `@choice`, `@jump`, or `@end`.
* If any reachable path can fall through to the end of `#STORY` without a transition, compilation fails.
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
    
    if ($system_stability < 50) {
        @bgm "warning_siren.wav"
    } else {
        @bgm "steady_hum.wav"
    }

    #STORY
    "Sparks shower from the ceiling as the primary coolant line shudders."

    TEO(calm, Left): "Variance is up by twelve percent. Gippie, run a sector scan."

    if ($system_stability < 50) {
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
