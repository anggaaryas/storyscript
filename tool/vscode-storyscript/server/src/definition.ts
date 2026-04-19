/* ------------------------------------------------------------------ *
 *  StoryScript LSP — Definition provider                              *
 * ------------------------------------------------------------------ */

import { Location, Position, Range } from 'vscode-languageserver/node';
import { DocumentInfo } from './parser';

export function getDefinition(
	docInfo: DocumentInfo,
	text: string,
	position: Position,
	uri: string
): Location | null {
	const lines = text.split(/\r?\n/);
	const line = lines[position.line] ?? '';
	const col = position.character;

	const wordRange = getWordRangeAtPosition(line, col);
	if (!wordRange) return null;

	const word = line.slice(wordRange.start, wordRange.end);

	// Check if it's a $ variable reference
	if (wordRange.start > 0 && line[wordRange.start - 1] === '$') {
		const variable = docInfo.variables.find(v => v.name === word);
		if (variable) {
			return Location.create(uri, Range.create(
				Position.create(variable.line, variable.column),
				Position.create(variable.line, variable.column + variable.name.length + 1)
			));
		}
	}

	// Check if it's a scene reference (after ->, @jump, @start)
	const scene = docInfo.scenes.find(s => s.name === word && !s.isInit && !s.isRequire);
	if (scene) {
		return Location.create(uri, Range.create(
			Position.create(scene.line, scene.column),
			Position.create(scene.line, scene.column + scene.name.length + 2)
		));
	}

	// Check if it's an actor reference
	const actor = docInfo.actors.find(a => a.id === word);
	if (actor) {
		return Location.create(uri, Range.create(
			Position.create(actor.line, actor.column),
			Position.create(actor.line, actor.column + actor.id.length + 7)
		));
	}

	return null;
}

export function getReferences(
	docInfo: DocumentInfo,
	text: string,
	position: Position,
	uri: string
): Location[] {
	const lines = text.split(/\r?\n/);
	const line = lines[position.line] ?? '';
	const col = position.character;

	const wordRange = getWordRangeAtPosition(line, col);
	if (!wordRange) return [];

	const word = line.slice(wordRange.start, wordRange.end);
	const locations: Location[] = [];

	// Find all references to a scene
	const scene = docInfo.scenes.find(s => s.name === word && !s.isInit && !s.isRequire);
	if (scene) {
		// Definition
		locations.push(Location.create(uri, Range.create(
			Position.create(scene.line, scene.column),
			Position.create(scene.line, scene.column + scene.name.length + 2)
		)));
		// References (from @jump, @start, choice arrows)
		for (const ref of docInfo.sceneRefs) {
			if (ref.name === word) {
				locations.push(Location.create(uri, Range.create(
					Position.create(ref.line, ref.column),
					Position.create(ref.line, ref.column + ref.name.length)
				)));
			}
		}
		return locations;
	}

	// Find all references to a variable
	const isVarCtx = wordRange.start > 0 && line[wordRange.start - 1] === '$';
	if (isVarCtx) {
		const varName = word;
		// Scan all lines for $varName occurrences
		for (let i = 0; i < lines.length; i++) {
			const re = new RegExp(`\\$${escapeRegex(varName)}\\b`, 'g');
			let m;
			while ((m = re.exec(lines[i])) !== null) {
				locations.push(Location.create(uri, Range.create(
					Position.create(i, m.index),
					Position.create(i, m.index + varName.length + 1)
				)));
			}
		}
		return locations;
	}

	// Find all references to an actor
	const actor = docInfo.actors.find(a => a.id === word);
	if (actor) {
		for (let i = 0; i < lines.length; i++) {
			const re = new RegExp(`\\b${escapeRegex(word)}\\b`, 'g');
			let m;
			while ((m = re.exec(lines[i])) !== null) {
				locations.push(Location.create(uri, Range.create(
					Position.create(i, m.index),
					Position.create(i, m.index + word.length)
				)));
			}
		}
		return locations;
	}

	return locations;
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

function escapeRegex(s: string): string {
	return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
