/* ------------------------------------------------------------------ *
 *  StoryScript LSP — Completion provider                              *
 * ------------------------------------------------------------------ */

import {
	CompletionItem,
	CompletionItemKind,
	InsertTextFormat,
	Position,
} from 'vscode-languageserver/node';

import {
	DocumentInfo,
	KEYWORDS,
	TYPES,
	DIRECTIVES,
	BUILTIN_FUNCTIONS,
	POSITIONS,
	PHASE_TAGS,
} from './parser';

/** Return context-aware completions for the given cursor position. */
export function getCompletions(
	docInfo: DocumentInfo,
	text: string,
	position: Position
): CompletionItem[] {
	const lines = text.split(/\r?\n/);
	const line = lines[position.line] ?? '';
	const prefix = line.slice(0, position.character);

	const items: CompletionItem[] = [];

	// ── Directive completions (after @) ──
	if (prefix.match(/@[a-zA-Z]*$/)) {
		for (const dir of DIRECTIVES) {
			items.push({
				label: dir,
				kind: CompletionItemKind.Keyword,
				detail: `StoryScript directive`,
				insertText: dir.slice(1), // strip leading @ since user already typed it
			});
		}
		return items;
	}

	// ── Phase tag completions (after #) ──
	if (prefix.match(/#[A-Z]*$/)) {
		for (const tag of PHASE_TAGS) {
			items.push({
				label: tag,
				kind: CompletionItemKind.Keyword,
				detail: 'Phase tag',
				insertText: tag.slice(1),
			});
		}
		return items;
	}

	// ── Variable completions (after $) ──
	if (prefix.match(/\$[a-zA-Z_]*$/)) {
		const seen = new Set<string>();
		for (const v of docInfo.variables) {
			if (!seen.has(v.name)) {
				seen.add(v.name);
				items.push({
					label: `$${v.name}`,
					kind: CompletionItemKind.Variable,
					detail: `${v.scope} ${v.varType}`,
					insertText: v.name,
				});
			}
		}
		return items;
	}

	// ── Type completions (after "as ") ──
	if (prefix.match(/\bas\s+[a-z]*$/)) {
		for (const t of TYPES) {
			items.push({
				label: t,
				kind: CompletionItemKind.TypeParameter,
				detail: 'StoryScript type',
			});
		}
		items.push({
			label: 'array<integer>',
			kind: CompletionItemKind.TypeParameter,
			detail: 'Typed array',
		});
		items.push({
			label: 'array<string>',
			kind: CompletionItemKind.TypeParameter,
			detail: 'Typed array',
		});
		items.push({
			label: 'array<boolean>',
			kind: CompletionItemKind.TypeParameter,
			detail: 'Typed array',
		});
		items.push({
			label: 'array<decimal>',
			kind: CompletionItemKind.TypeParameter,
			detail: 'Typed array',
		});
		return items;
	}

	// ── Arrow target (after ->) — scene names ──
	if (prefix.match(/->\s*[a-zA-Z_]*$/)) {
		for (const scene of docInfo.scenes) {
			if (!scene.isInit && !scene.isRequire) {
				items.push({
					label: scene.name,
					kind: CompletionItemKind.Function,
					detail: 'Scene',
				});
			}
		}
		return items;
	}

	// ── After @jump or @start — scene names ──
	if (prefix.match(/@(?:jump|start)\s+[a-zA-Z_]*$/)) {
		for (const scene of docInfo.scenes) {
			if (!scene.isInit && !scene.isRequire) {
				items.push({
					label: scene.name,
					kind: CompletionItemKind.Function,
					detail: 'Scene',
				});
			}
		}
		return items;
	}

	// ── Position completions inside dialogue parentheses ──
	if (prefix.match(/[A-Z_][A-Z0-9_]*\s*\(\s*[a-zA-Z_]+\s*,\s*[a-zA-Z]*$/)) {
		for (const pos of POSITIONS) {
			items.push({
				label: pos,
				kind: CompletionItemKind.EnumMember,
				detail: 'Portrait position',
			});
		}
		return items;
	}

	// ── Actor emotion completions inside parentheses ──
	const dialogueCtx = prefix.match(/([A-Z_][A-Z0-9_]*)\s*\(\s*([a-zA-Z_]*)$/);
	if (dialogueCtx) {
		const actorId = dialogueCtx[1];
		const actor = docInfo.actors.find(a => a.id === actorId);
		if (actor) {
			for (const emotion of actor.emotions) {
				items.push({
					label: emotion,
					kind: CompletionItemKind.EnumMember,
					detail: `${actor.id} emotion`,
				});
			}
		}
		return items;
	}

	// ── General context — keywords, built-ins, actors, scenes ──
	// Keywords
	for (const kw of KEYWORDS) {
		items.push({
			label: kw,
			kind: CompletionItemKind.Keyword,
			detail: 'Keyword',
		});
	}

	// Built-in functions
	for (const fn of BUILTIN_FUNCTIONS) {
		items.push({
			label: fn,
			kind: CompletionItemKind.Function,
			detail: 'Built-in function',
			insertText: fn.startsWith('array_') ? fn : `${fn}($1)`,
			insertTextFormat: fn.startsWith('array_') ? InsertTextFormat.PlainText : InsertTextFormat.Snippet,
		});
	}

	// Actor IDs (for dialogue)
	for (const actor of docInfo.actors) {
		items.push({
			label: actor.id,
			kind: CompletionItemKind.Class,
			detail: `Actor: ${actor.displayName}`,
		});
	}

	// Snippets for common patterns
	items.push({
		label: '* scene',
		kind: CompletionItemKind.Snippet,
		detail: 'New scene block',
		insertText: '* ${1:scene_name} {\n\t#PREP\n\t$2\n\n\t#STORY\n\t$3\n}',
		insertTextFormat: InsertTextFormat.Snippet,
	});

	items.push({
		label: '@choice block',
		kind: CompletionItemKind.Snippet,
		detail: 'Choice block with options',
		insertText: '@choice {\n\t"${1:Option 1}" -> ${2:target_scene}\n\t"${3:Option 2}" -> ${4:target_scene2}\n}',
		insertTextFormat: InsertTextFormat.Snippet,
	});

	items.push({
		label: '@actor declaration',
		kind: CompletionItemKind.Snippet,
		detail: 'New actor declaration',
		insertText: '@actor ${1:ACTOR_ID} "${2:Display Name}" {\n\t${3:default} -> "${4:portrait.png}"\n}',
		insertTextFormat: InsertTextFormat.Snippet,
	});

	items.push({
		label: 'if block',
		kind: CompletionItemKind.Snippet,
		detail: 'Conditional block',
		insertText: 'if (${1:condition}) {\n\t$2\n}',
		insertTextFormat: InsertTextFormat.Snippet,
	});

	items.push({
		label: 'if/else block',
		kind: CompletionItemKind.Snippet,
		detail: 'Conditional with else',
		insertText: 'if (${1:condition}) {\n\t$2\n} else {\n\t$3\n}',
		insertTextFormat: InsertTextFormat.Snippet,
	});

	items.push({
		label: 'for snapshot loop',
		kind: CompletionItemKind.Snippet,
		detail: 'For-each loop over array snapshot',
		insertText: 'for (\\$${1:item} in snapshot \\$${2:array}) {\n\t$3\n}',
		insertTextFormat: InsertTextFormat.Snippet,
	});

	items.push({
		label: 'repeat loop',
		kind: CompletionItemKind.Snippet,
		detail: 'Repeat loop',
		insertText: 'repeat (${1:count}) {\n\t$2\n}',
		insertTextFormat: InsertTextFormat.Snippet,
	});

	return items;
}
