/* ------------------------------------------------------------------ *
 *  StoryScript LSP — Document symbols provider                        *
 * ------------------------------------------------------------------ */

import { DocumentSymbol, SymbolKind, Range, Position } from 'vscode-languageserver/node';
import { DocumentInfo } from './parser';

export function getDocumentSymbols(docInfo: DocumentInfo): DocumentSymbol[] {
	const symbols: DocumentSymbol[] = [];

	// Scenes
	for (const scene of docInfo.scenes) {
		const kind = scene.isInit
			? SymbolKind.Module
			: scene.isRequire
			? SymbolKind.Interface
			: SymbolKind.Function;

		const range = Range.create(
			Position.create(scene.line, scene.column),
			Position.create(scene.endLine, 999)
		);

		const children: DocumentSymbol[] = [];

		// Add variables declared in this scene
		for (const v of docInfo.variables) {
			if (v.sceneName === scene.name) {
				children.push({
					name: `$${v.name}`,
					detail: v.varType,
					kind: SymbolKind.Variable,
					range: Range.create(
						Position.create(v.line, v.column),
						Position.create(v.line, v.column + v.name.length + 1)
					),
					selectionRange: Range.create(
						Position.create(v.line, v.column),
						Position.create(v.line, v.column + v.name.length + 1)
					),
				});
			}
		}

		// Add actors declared in INIT
		if (scene.isInit) {
			for (const actor of docInfo.actors) {
				children.push({
					name: actor.id,
					detail: actor.displayName,
					kind: SymbolKind.Class,
					range: Range.create(
						Position.create(actor.line, actor.column),
						Position.create(actor.line, actor.column + actor.id.length + 7)
					),
					selectionRange: Range.create(
						Position.create(actor.line, actor.column),
						Position.create(actor.line, actor.column + actor.id.length + 7)
					),
				});
			}
		}

		symbols.push({
			name: scene.isInit ? '* INIT' : scene.isRequire ? '* REQUIRE' : `* ${scene.name}`,
			detail: scene.isInit ? 'Initialization' : scene.isRequire ? 'Requirements' : 'Scene',
			kind,
			range,
			selectionRange: Range.create(
				Position.create(scene.line, scene.column),
				Position.create(scene.line, scene.column + scene.name.length + 2)
			),
			children: children.length > 0 ? children : undefined,
		});
	}

	return symbols;
}
