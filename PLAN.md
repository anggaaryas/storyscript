# .StoryScript Language Specification

## 1. Global Initialization
Before any scenes are parsed, the engine must define global variables and load actor assets into memory. This is strictly handled in the reserved `* INIT` block.

**Syntax Rules:**
* Must be the absolute first block evaluated by the compiler.
* Handles global variable declaration (`$`).
* Handles character registration (`@actor`) using block-based dictionary syntax.

```plaintext
* INIT {
    // Variable Registration
    $clearance_level = 1;
    $has_admin_key = false;

    // Standard Actor (With Portraits)
    @actor FL "Flaurel" {
        neutral -> "fl_neutral.png"
        nervous -> "fl_nerv.png"
        happy -> "fl_happy.png"
    }

    // Portrait-less Actor (Voice-only or un-drawn)
    @actor SYS "System AI";
}
```

---

## 2. The Scene Lifecycle
A standard scene is defined using `* <scene_label> { ... }`. Every scene operates on a strict, sequential two-phase lifecycle. Blocks must appear in exact order.

### Phase 1: `#PREP` (Execution Phase)
The invisible backend phase. The parser executes all math, updates state arrays, and queues engine assets instantly before rendering anything to the screen. 

* **Allowed Tokens:** `$`, `@bg`, `@bgm`, `@sfx`, `if`/`else`.
* **Forbidden Tokens:** `"Narrative text"`, `ActorID()`, `@choice`, `@jump`.

### Phase 2: `#STORY` (Rendering & Interaction Phase)
The player-facing phase. The UI sequentially renders text and dialogue. Execution pauses when requiring user input or a hard scene transition.

* **Allowed Tokens:** `"Narrative text"`, `ActorID()`, `if`/`else`, `@choice`, `@jump`.
* **Forbidden Tokens:** `@bg`, `@bgm`, `@sfx`, `$`. 
* **Strict Rule:** A navigation directive (`@choice` or `@jump`) must be the absolute final executed token in this block.

---

## 3. Syntax Reference

### Variables & Logic
Standard C-style conditionals are supported in both `#PREP` and `#STORY` blocks. Variables must be prefixed with `$`.

```plaintext
if ($clearance_level >= 2) {
    $access_granted = true;
} else {
    $access_granted = false;
}
```

### Engine Directives (Only in `#PREP`)
Directives tell the visual/audio engine what to queue.

| Directive | Purpose | Syntax | Example |
| :--- | :--- | :--- | :--- |
| **@bg** | Loads a background image. | `@bg "<path>"` | `@bg "server_room.png"` |
| **@bgm** | Plays looping background music. | `@bgm "<path>" / "STOP"` | `@bgm "tense_hum.wav"` |
| **@sfx** | Plays a one-shot sound effect. | `@sfx "<path>"` | `@sfx "alarm.wav"` |

### Dialogue & Narration (Only in `#STORY`)
Narration is handled via standard string literals. Dialogue utilizes the registered Actor IDs from the `INIT` block. 

The parser enforces strict rules based on the presence of parentheses to prevent missing-asset crashes.

* **With Portrait:** Must contain exactly two parameters: `(<emotion_key>, <Position>)`. Valid positions are `Left`, `Center`, `Right` (or `L`, `C`, `R`).
* **Without Portrait:** Omit parentheses entirely. The engine renders the display name and text with no sprite.

```plaintext
"The terminal sparks violently."

// Renders sprite and text
FL(nervous, Left): "I don't think that worked."

// Renders text only
SYS: "Warning. Intruder detected."
```

### Navigation Directives (Only in `#STORY`)
These directives handle transitioning out of the current scene and must be the final token read in a block.

**@choice**
Halts the engine and renders a user-selectable menu. Options map to the next scene via `->`. Supports nested conditionals.

```plaintext
@choice {
    "Examine the terminal" -> scene_examine;
    
    if ($has_admin_key == true) {
        "Initiate manual override" -> scene_override;
    }
}
```

**@jump**
Automatically transitions to the next scene without user input. Used for script chunking, invisible logic routing hubs, or seamless cinematic transitions.

```plaintext
"The screen fades to black."
@jump scene_next_day;
```

---

## 4. Comprehensive Parser Example

```plaintext
* INIT {
    $trust_metric = 5;
    $has_override = false;
    
    @actor FL "Flaurel" {
        neutral -> "fl_neutral.png"
        nervous -> "fl_nerv.png"
        happy -> "fl_happy.png"
    }
    
    @actor SYS "Angarsa System";
}

* lab_entrance {
    
    #PREP
    @bg "angarsa_labs_entrance.png"
    
    if ($trust_metric > 4) {
        @bgm "calm_drone.wav"
    } else {
        @bgm "tense_drone.wav"
    }

    #STORY
    "The heavy blast doors of the main laboratory loom ahead."

    FL(neutral, Left): "We finally made it to the central terminal."

    if ($trust_metric > 4) {
        FL(happy, Left): "I'm really glad you're the one watching my back."
    } else {
        FL(nervous, Left): "Stay sharp. The drones are active."
    }

    SYS: "Please present valid identification."

    @choice {
        "Scan standard ID card" -> scene_standard_entry;
        "Attempt manual bypass" -> scene_hack;

        if ($has_override == true) {
            "Use Admin Override" -> scene_admin_entry;
        }
    }
}

* scene_hack {
    
    #PREP
    @bgm "STOP"
    @sfx "alarm_blare.wav"
    $trust_metric = $trust_metric - 2;

    #STORY
    "You pry open the panel and short the connection. Sparks fly."
    FL(nervous, Left): "Are you crazy?! You just triggered the countermeasures!"
    
    @jump scene_game_over;
}

...
```
