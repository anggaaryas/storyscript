# .StoryScript Language Specification

## 1. Global Initialization
Before any scenes are parsed, the engine must define global variables, load actor assets into memory, and explicitly define the game's entry point. This is strictly handled in the reserved `* INIT` block.

**Syntax Rules:**
* Must be the absolute first block evaluated by the compiler.
* Handles global variable declaration (`$`).
* Handles character registration (`@actor`) using block-based dictionary syntax.
* **Mandatory:** Must contain exactly one `@start` directive pointing to the first scene.

```plaintext
* INIT {
    // Variable Registration
    $system_stability = 45;
    $manual_override = false;

    // Standard Actor (With Portraits)
    @actor TEO "Teona" {
        neutral -> "teo_neutral.png";
        calm -> "teo_calm.png";
        focus -> "teo_focus.png";
    }
    @actor GIP "Gippie" {
        default -> "gip_smile.png";
        playful -> "gip_wink.png";
        alert -> "gip_alert.png";
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
if ($system_stability <= 30) {
    $critical_warning = true;
} else {
    $critical_warning = false;
}
```

### Engine Directives (Only in `#PREP`)
Directives tell the visual/audio engine what to queue.

| Directive | Purpose | Syntax | Example |
| :--- | :--- | :--- | :--- |
| **@bg** | Loads a background image. | `@bg "<path>"` | `@bg "server_room.png"` |
| **@bgm** | Plays looping background music. | `@bgm "<path>" / STOP` | `@bgm "tense_hum.wav"` |
| **@sfx** | Plays a one-shot sound effect. | `@sfx "<path>"` | `@sfx "spark.wav"` |

### Dialogue & Narration (Only in `#STORY`)
Narration is handled via standard string literals. Dialogue utilizes the registered Actor IDs from the `INIT` block. 

The parser enforces strict rules based on the presence of parentheses to prevent missing-asset crashes.

* **With Portrait:** Must contain exactly two parameters: `(<emotion_key>, <Position>)`. Valid positions are `Left`, `Center`, `Right` (or `L`, `C`, `R`).
* **Without Portrait:** Omit parentheses entirely. The engine renders the display name and text with no sprite.

```plaintext
"The main console flashes red."

// Renders sprite and text
TEO(focus, Left): "We need to isolate the memory leak."

// Renders text only (if using a portrait-less setup or quick text)
GIP: "On it, boss!"
```

### Navigation Directives (Only in `#STORY`)
These directives handle transitioning out of the current scene and must be the final token read in a block.

**@choice**
Halts the engine and renders a user-selectable menu. Options map to the next scene via `->`. Supports nested conditionals.

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

---

## 4. Comprehensive Parser Example

```plaintext
* INIT {
    $system_stability = 40;
    $bypass_key = false;
    
    @actor TEO "Teona" {
        calm -> "teo_calm.png";
        focus -> "teo_focus.png";
    }
    
    @actor GIP "Gippie" {
        default -> "gip_smile.png";
        playful -> "gip_wink.png";
        alert -> "gip_alert.png";
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
