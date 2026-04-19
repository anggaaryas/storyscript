/* ------------------------------------------------------------------ *
 *  StoryScript LSP — Hover provider                                   *
 * ------------------------------------------------------------------ */

import { Hover, Position, MarkupKind } from 'vscode-languageserver/node';
import { DocumentInfo } from './parser';

/** Built-in documentation for keywords, directives and functions. */
const DOCS: Record<string, string> = {
	// Phase tags
	'#PREP': 'Phase tag — state mutation and engine directives.\n\nUsed for variable assignments, `@bg`, `@bgm`, `@sfx`, and local variable declarations.',
	'#STORY': 'Phase tag — narration, dialogue, branching and transitions.\n\nContains story text, actor dialogue, `if`/`else`, `@choice`, `@jump`, and `@end`.',

	// Directives
	'@actor': 'Declares an actor with an ID, display name, and optional portrait map.\n\n```\n@actor CMDR "Commander" {\n  focus -> "cmdr_focus.png"\n}\n```',
	'@bg': 'Sets the background image.\n\n```\n@bg "bridge.png"\n```\n\nOnly valid in `#PREP`.',
	'@bgm': 'Sets background music, or stops it with `STOP`.\n\n```\n@bgm "theme.ogg"\n@bgm STOP\n```\n\nOnly valid in `#PREP`.',
	'@sfx': 'Plays a one-shot sound effect.\n\n```\n@sfx "alert.wav"\n```\n\nValid in `#PREP` and `#STORY`.',
	'@choice': 'Begins a choice block. Each entry is `"text" -> scene_name`.\n\n```\n@choice {\n  "Investigate" -> signal_room\n  "Lock down" -> lockdown\n}\n```\n\nRuntime choice cap: 9 options.',
	'@jump': 'Unconditional jump to another scene.\n\n```\n@jump next_scene\n```',
	'@end': 'Ends the story at this point.\n\n```\n@end\n```',
	'@start': 'Sets the entry-point scene. Must appear inside `* INIT`.\n\n```\n@start opening\n```',
	'@include': 'Includes child module files. Only valid in root `* INIT`.\n\n```\n@include ["modules/side_quest.StoryScript"]\n```',

	// Keywords
	'if': 'Conditional branch. Condition must be boolean-typed.\n\n```\nif ($morale > 50) {\n  "The crew is confident."\n}\n```',
	'else': 'Alternative branch after `if` or `else if`.',
	'for': 'Snapshot iteration over an array.\n\n```\nfor ($item in snapshot $inventory) {\n  "${item}"\n}\n```',
	'repeat': 'Counted loop.\n\n```\nrepeat (3) {\n  "Tick..."\n}\n```',
	'in': 'Used with `for` loops: `for ($item in snapshot $array)`.',
	'snapshot': 'Creates a frozen copy of an array for loop iteration.',
	'break': 'Exits the nearest enclosing `for` or `repeat` loop.',
	'continue': 'Skips to the next iteration of the nearest enclosing loop.',
	'as': 'Type annotation keyword.\n\n```\n$health as integer = 100\n```',
	'INIT': 'Global initialization block — variables, actors, includes, and `@start`.\n\n```\n* INIT {\n  $score as integer = 0\n  @start opening\n}\n```',
	'REQUIRE': 'Child module dependency declaration.\n\n```\n* REQUIRE {\n  $health as integer\n  @actor HERO [ idle, attack ]\n}\n```',
	'STOP': 'Stops background music when used with `@bgm`.\n\n```\n@bgm STOP\n```',

	// Types
	'integer': 'Whole number type. Example: `$count as integer = 42`',
	'string': 'Text type. Example: `$name as string = "Ari"`',
	'boolean': 'Boolean type (`true`/`false`). Example: `$flag as boolean = false`',
	'decimal': 'Floating-point type. Example: `$rate as decimal = 1.5`',
	'array': 'Typed array. Usage: `array<integer>`, `array<string>`, `array<boolean>`, `array<decimal>`',

	// Built-in functions
	'abs': '`abs(x)` — Returns the absolute value of a numeric expression.',
	'rand': '`rand()` — Random value (type matches assignment target).\n\n`rand(min, max)` — Random value in inclusive range.',
	'pick': '`pick(array)` — Picks a random element from an array.\n\n`pick(count, array)` — Picks `count` unique random elements.',
	'array_push': '`array_push($arr, value)` — Appends a value to the array. `#PREP` only.',
	'array_pop': '`array_pop($arr)` — Removes and returns the last element.',
	'array_strip': '`array_strip($arr, value)` — Removes all occurrences of value. `#PREP` only.',
	'array_clear': '`array_clear($arr)` — Removes all elements. `#PREP` only.',
	'array_contains': '`array_contains($arr, value)` — Returns `true` if array contains value.',
	'array_size': '`array_size($arr)` — Returns the number of elements.',
	'array_join': '`array_join($arr, separator)` — Joins elements into a string.',
	'array_get': '`array_get($arr, index)` — Returns element at index (0-based).',
	'array_insert': '`array_insert($arr, index, value)` — Inserts value at index. `#PREP` only.',
	'array_remove': '`array_remove($arr, index)` — Removes element at index. `#PREP` only.',
};

export function getHover(
	docInfo: DocumentInfo,
	text: string,
	position: Position
): Hover | null {
	const lines = text.split(/\r?\n/);
	const line = lines[position.line] ?? '';
	const col = position.character;

	// Get the word at position
	const wordRange = getWordRangeAtPosition(line, col);
	if (!wordRange) return null;

	let word = line.slice(wordRange.start, wordRange.end);

	// Check for # prefix (phase tags)
	if (wordRange.start > 0 && line[wordRange.start - 1] === '#') {
		word = '#' + word;
	}

	// Check for @ prefix (directives)
	if (wordRange.start > 0 && line[wordRange.start - 1] === '@') {
		word = '@' + word;
	}

	// Check for $ prefix (variables)
	if (wordRange.start > 0 && line[wordRange.start - 1] === '$') {
		const varName = word;
		const variable = docInfo.variables.find(v => v.name === varName);
		if (variable) {
			const scopeLabel = variable.scope === 'global' ? 'global' : `local (${variable.sceneName})`;
			return {
				contents: {
					kind: MarkupKind.Markdown,
					value: `**\$${variable.name}** — \`${variable.varType}\` (${scopeLabel})\n\nDeclared at line ${variable.line + 1}`,
				},
			};
		}
	}

	// Check documentation table
	if (DOCS[word]) {
		return {
			contents: {
				kind: MarkupKind.Markdown,
				value: DOCS[word],
			},
		};
	}

	// Check if it's an actor ID
	const actor = docInfo.actors.find(a => a.id === word);
	if (actor) {
		const emotions = actor.emotions.length > 0 ? actor.emotions.join(', ') : 'none';
		return {
			contents: {
				kind: MarkupKind.Markdown,
				value: `**Actor ${actor.id}** — "${actor.displayName}"\n\nEmotions: ${emotions}\n\nDeclared at line ${actor.line + 1}`,
			},
		};
	}

	// Check if it's a scene name
	const scene = docInfo.scenes.find(s => s.name === word && !s.isInit && !s.isRequire);
	if (scene) {
		return {
			contents: {
				kind: MarkupKind.Markdown,
				value: `**Scene** \`${scene.name}\`\n\nDefined at line ${scene.line + 1}`,
			},
		};
	}

	return null;
}

function getWordRangeAtPosition(line: string, col: number): { start: number; end: number } | null {
	const wordPattern = /[a-zA-Z_][a-zA-Z0-9_]*/g;
	let match;
	while ((match = wordPattern.exec(line)) !== null) {
		const start = match.index;
		const end = start + match[0].length;
		if (col >= start && col <= end) {
			return { start, end };
		}
	}
	return null;
}
