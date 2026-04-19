/* ------------------------------------------------------------------ *
 *  StoryScript LSP — Lightweight document parser                      *
 *  Extracts symbols, diagnostics and structure from .StoryScript text *
 * ------------------------------------------------------------------ */

import { Diagnostic, DiagnosticSeverity, Range, Position } from 'vscode-languageserver/node';

// ── Symbol types ──────────────────────────────────────────────────

export interface SceneSymbol {
	name: string;
	line: number;  // 0-based
	column: number;
	endLine: number;
	isInit: boolean;
	isRequire: boolean;
}

export interface VariableSymbol {
	name: string;
	varType: string;
	line: number;
	column: number;
	scope: 'global' | 'local';
	sceneName?: string;
}

export interface ActorSymbol {
	id: string;
	displayName: string;
	emotions: string[];
	line: number;
	column: number;
}

export interface DirectiveUsage {
	name: string;
	target?: string;
	line: number;
	column: number;
}

export interface SceneRef {
	name: string;
	line: number;
	column: number;
}

export interface DocumentInfo {
	scenes: SceneSymbol[];
	variables: VariableSymbol[];
	actors: ActorSymbol[];
	directives: DirectiveUsage[];
	sceneRefs: SceneRef[];
	diagnostics: Diagnostic[];
}

// ── Keyword / built-in tables ─────────────────────────────────────

export const KEYWORDS = [
	'if', 'else', 'for', 'repeat', 'in', 'snapshot',
	'break', 'continue', 'as', 'INIT', 'REQUIRE', 'STOP',
	'true', 'false',
];

export const TYPES = ['integer', 'string', 'boolean', 'decimal', 'array'];

export const DIRECTIVES = [
	'@actor', '@bg', '@bgm', '@sfx', '@choice', '@jump',
	'@end', '@start', '@include',
];

export const BUILTIN_FUNCTIONS = [
	'abs', 'rand', 'pick',
	'array_push', 'array_pop', 'array_strip', 'array_clear',
	'array_contains', 'array_size', 'array_join',
	'array_get', 'array_insert', 'array_remove',
];

export const POSITIONS = ['Left', 'Right', 'Center', 'L', 'R', 'C'];

export const PHASE_TAGS = ['#PREP', '#STORY'];

// ── Parser ────────────────────────────────────────────────────────

export function parseDocument(text: string, uri: string): DocumentInfo {
	const lines = text.split(/\r?\n/);
	const scenes: SceneSymbol[] = [];
	const variables: VariableSymbol[] = [];
	const actors: ActorSymbol[] = [];
	const directives: DirectiveUsage[] = [];
	const sceneRefs: SceneRef[] = [];
	const diagnostics: Diagnostic[] = [];

	let currentScene: SceneSymbol | null = null;
	let braceDepth = 0;
	let inInit = false;
	let inRequire = false;
	let currentPhase: 'prep' | 'story' | null = null;
	let hasInit = false;
	let hasStart = false;
	const declaredSceneNames = new Set<string>();
	const declaredVarNames = new Set<string>();
	const declaredActorIds = new Set<string>();
	const referencedScenes: SceneRef[] = [];

	for (let i = 0; i < lines.length; i++) {
		const line = lines[i];
		const trimmed = line.replace(/\/\/.*$/, '').trim();

		// Count braces for scope tracking
		for (const ch of trimmed) {
			if (ch === '{') braceDepth++;
			if (ch === '}') {
				braceDepth--;
				if (braceDepth <= 0 && currentScene) {
					currentScene.endLine = i;
					currentScene = null;
					currentPhase = null;
					inInit = false;
					inRequire = false;
					braceDepth = 0;
				}
			}
		}

		// ── Scene declarations: * LABEL { ──
		const sceneMatch = trimmed.match(/^\*\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{?/);
		if (sceneMatch) {
			const name = sceneMatch[1];
			const col = line.indexOf('*');
			const isInit = name === 'INIT';
			const isReq = name === 'REQUIRE';

			if (isInit) {
				if (hasInit) {
					diagnostics.push(makeDiag(i, col, col + name.length + 2, 'Duplicate * INIT block', DiagnosticSeverity.Error));
				}
				hasInit = true;
				inInit = true;
			}

			if (isReq) {
				inRequire = true;
			}

			if (!isInit && !isReq && declaredSceneNames.has(name)) {
				diagnostics.push(makeDiag(i, col, col + name.length + 2, `Duplicate scene name '${name}'`, DiagnosticSeverity.Error));
			}

			const scene: SceneSymbol = {
				name,
				line: i,
				column: col,
				endLine: i,
				isInit,
				isRequire: isReq,
			};
			scenes.push(scene);
			currentScene = scene;
			if (!isInit && !isReq) declaredSceneNames.add(name);

			if (trimmed.includes('{')) {
				braceDepth = 1;
			}
			continue;
		}

		// ── Phase tags ──
		if (trimmed === '#PREP' || trimmed === '#STORY') {
			currentPhase = trimmed === '#PREP' ? 'prep' : 'story';
			if (!currentScene || currentScene.isInit) {
				diagnostics.push(makeDiag(i, 0, trimmed.length, `${trimmed} outside of a scene block`, DiagnosticSeverity.Error));
			}
			continue;
		}

		// ── @actor directive ──
		const actorMatch = trimmed.match(/^@actor\s+([A-Z_][A-Z0-9_]*)\s*("(?:[^"\\]|\\.)*")?/);
		if (actorMatch) {
			const id = actorMatch[1];
			const displayName = actorMatch[2] ? actorMatch[2].slice(1, -1) : id;
			const col = line.indexOf('@actor');

			if (declaredActorIds.has(id)) {
				diagnostics.push(makeDiag(i, col, col + id.length + 7, `Duplicate actor ID '${id}'`, DiagnosticSeverity.Error));
			}
			declaredActorIds.add(id);

			// Parse emotions from portrait map
			const emotions: string[] = [];
			const emotionRe = /([a-zA-Z_][a-zA-Z0-9_]*)\s*->\s*"[^"]*"/g;
			let em;
			while ((em = emotionRe.exec(line)) !== null) {
				emotions.push(em[1]);
			}

			actors.push({ id, displayName, emotions, line: i, column: col });
			directives.push({ name: '@actor', target: id, line: i, column: col });
			continue;
		}

		// ── @start directive ──
		const startMatch = trimmed.match(/^@start\s+([a-zA-Z_][a-zA-Z0-9_]*)/);
		if (startMatch) {
			const target = startMatch[1];
			const col = line.indexOf('@start');
			if (hasStart) {
				diagnostics.push(makeDiag(i, col, col + target.length + 7, 'Duplicate @start directive', DiagnosticSeverity.Error));
			}
			hasStart = true;
			directives.push({ name: '@start', target, line: i, column: col });
			referencedScenes.push({ name: target, line: i, column: line.indexOf(target, col + 6) });
			continue;
		}

		// ── @jump directive ──
		const jumpMatch = trimmed.match(/^@jump\s+([a-zA-Z_][a-zA-Z0-9_]*)/);
		if (jumpMatch) {
			const target = jumpMatch[1];
			const col = line.indexOf('@jump');
			directives.push({ name: '@jump', target, line: i, column: col });
			referencedScenes.push({ name: target, line: i, column: line.indexOf(target, col + 5) });
			continue;
		}

		// ── @include directive ──
		const includeMatch = trimmed.match(/^@include\b/);
		if (includeMatch) {
			const col = line.indexOf('@include');
			directives.push({ name: '@include', line: i, column: col });
			continue;
		}

		// ── @bg, @bgm, @sfx ──
		for (const dir of ['@bg', '@bgm', '@sfx']) {
			const re = new RegExp(`^${dir.replace('@', '@')}\\b`);
			if (re.test(trimmed)) {
				const col = line.indexOf(dir);
				directives.push({ name: dir, line: i, column: col });
				if (currentPhase === 'story' && dir !== '@sfx') {
					// @bg and @bgm are not valid in #STORY for @bg; @sfx is allowed
					// Actually @bg and @bgm are PREP-only
					diagnostics.push(makeDiag(i, col, col + dir.length, `${dir} is only allowed in #PREP`, DiagnosticSeverity.Error));
				}
				break;
			}
		}

		// ── @end directive ──
		if (trimmed.startsWith('@end')) {
			const col = line.indexOf('@end');
			directives.push({ name: '@end', line: i, column: col });
			continue;
		}

		// ── @choice block ──
		if (trimmed.startsWith('@choice')) {
			const col = line.indexOf('@choice');
			directives.push({ name: '@choice', line: i, column: col });
			continue;
		}

		// ── Choice arrows: "text" -> target ──
		const choiceArrow = trimmed.match(/"(?:[^"\\]|\\.)*"\s*->\s*([a-zA-Z_][a-zA-Z0-9_]*)/);
		if (choiceArrow) {
			const target = choiceArrow[1];
			const targetCol = line.indexOf(target, line.indexOf('->'));
			referencedScenes.push({ name: target, line: i, column: targetCol });
		}

		// ── Variable declarations: $name as type = value ──
		const varDeclMatch = trimmed.match(/^\$([a-zA-Z_][a-zA-Z0-9_]*)\s+as\s+(integer|string|boolean|decimal|array\s*<\s*(?:integer|string|boolean|decimal)\s*>)/);
		if (varDeclMatch) {
			const name = varDeclMatch[1];
			const varType = varDeclMatch[2].replace(/\s+/g, '');
			const col = line.indexOf('$');
			const scope: 'global' | 'local' = inInit ? 'global' : 'local';

			if (scope === 'global' && declaredVarNames.has(name)) {
				diagnostics.push(makeDiag(i, col, col + name.length + 1, `Duplicate variable '$${name}'`, DiagnosticSeverity.Error));
			}
			declaredVarNames.add(name);

			variables.push({
				name,
				varType,
				line: i,
				column: col,
				scope,
				sceneName: currentScene?.name,
			});
			continue;
		}

		// ── Variable assignments: $name = expr, $name += expr, $name -= expr ──
		const varAssignMatch = trimmed.match(/^\$([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:=|\+=|-=)/);
		if (varAssignMatch && !varDeclMatch) {
			const name = varAssignMatch[1];
			if (!declaredVarNames.has(name)) {
				const col = line.indexOf('$');
				diagnostics.push(makeDiag(i, col, col + name.length + 1, `Undeclared variable '$${name}'`, DiagnosticSeverity.Warning));
			}
		}

		// ── REQUIRE variable refs ──
		if (inRequire) {
			const reqVar = trimmed.match(/^\$([a-zA-Z_][a-zA-Z0-9_]*)\s+as\s+(integer|string|boolean|decimal|array\s*<\s*(?:integer|string|boolean|decimal)\s*>)/);
			if (reqVar) {
				// REQUIRE vars are tracked but not added to declared set
				variables.push({
					name: reqVar[1],
					varType: reqVar[2].replace(/\s+/g, ''),
					line: i,
					column: line.indexOf('$'),
					scope: 'global',
					sceneName: 'REQUIRE',
				});
			}
		}

		// ── Dialogue: check actor reference ──
		const dialogueMatch = trimmed.match(/^([A-Z_][A-Z0-9_]*)\s*(?:\(|:)/);
		if (dialogueMatch && !trimmed.startsWith('@') && !trimmed.startsWith('*')) {
			const actorId = dialogueMatch[1];
			if (actorId !== 'INIT' && actorId !== 'REQUIRE' && actorId !== 'STOP' &&
				declaredActorIds.size > 0 && !declaredActorIds.has(actorId)) {
				const col = line.indexOf(actorId);
				diagnostics.push(makeDiag(i, col, col + actorId.length, `Unknown actor '${actorId}'`, DiagnosticSeverity.Warning));
			}
		}
	}

	// Post-parse: check scene references
	for (const ref of referencedScenes) {
		sceneRefs.push(ref);
		if (!declaredSceneNames.has(ref.name)) {
			diagnostics.push(makeDiag(
				ref.line, ref.column, ref.column + ref.name.length,
				`Scene '${ref.name}' is not defined`,
				DiagnosticSeverity.Error
			));
		}
	}

	// Check for missing @start in root files (files with INIT)
	if (hasInit && !hasStart) {
		diagnostics.push(makeDiag(0, 0, 1, 'Missing @start directive in * INIT', DiagnosticSeverity.Error));
	}

	return { scenes, variables, actors, directives, sceneRefs, diagnostics };
}

function makeDiag(
	line: number,
	startCol: number,
	endCol: number,
	message: string,
	severity: DiagnosticSeverity
): Diagnostic {
	return {
		range: Range.create(Position.create(line, startCol), Position.create(line, endCol)),
		severity,
		source: 'storyscript',
		message,
	};
}
