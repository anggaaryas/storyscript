/* ------------------------------------------------------------------ *
 *  StoryScript LSP — Main server                                      *
 * ------------------------------------------------------------------ */

import {
	createConnection,
	TextDocuments,
	ProposedFeatures,
	InitializeParams,
	InitializeResult,
	TextDocumentSyncKind,
	CompletionParams,
	HoverParams,
	DefinitionParams,
	ReferenceParams,
	DocumentSymbolParams,
	DidChangeConfigurationNotification,
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';
import { parseDocument, DocumentInfo } from './parser';
import { getCompletions } from './completions';
import { getHover } from './hover';
import { getDocumentSymbols } from './symbols';
import { getDefinition, getReferences } from './definition';
import { LocalizationIndex } from './localization';

// ── Connection & document store ───────────────────────────────────

const connection = createConnection(ProposedFeatures.all);
const documents = new TextDocuments(TextDocument);

// Cache parsed results per document URI
const docCache = new Map<string, DocumentInfo>();
const localization = new LocalizationIndex();
let published = new Set<string>();
let workspaceRoots: string[] = [];
function publishLocalization(): void {
	const uris = new Set([...published, ...localization.diagnostics.keys(), ...documents.all().map(d => d.uri)]);
	for (const uri of uris) connection.sendDiagnostics({ uri, diagnostics: [
		...(uri.endsWith('.StoryScript') ? getDocInfo(uri).diagnostics : []),
		...(localization.diagnostics.get(uri) ?? []),
	] });
	published = uris;
}

// ── Initialization ────────────────────────────────────────────────

connection.onInitialize((params: InitializeParams): InitializeResult => {
	workspaceRoots = params.workspaceFolders?.map(f => f.uri) ?? (params.rootUri ? [params.rootUri] : []);
	localization.setRoots(workspaceRoots);
	return {
		capabilities: {
			textDocumentSync: TextDocumentSyncKind.Incremental,
			completionProvider: {
				triggerCharacters: ['@', '#', '$', '.', '>', ' '],
				resolveProvider: false,
			},
			hoverProvider: true,
			definitionProvider: true,
			referencesProvider: true,
			documentSymbolProvider: true,
			workspaceSymbolProvider: true,
			renameProvider: { prepareProvider: true },
			codeActionProvider: true,
			workspace: { workspaceFolders: { supported: true, changeNotifications: true } },
		},
	};
});

connection.onInitialized(() => {
	connection.client.register(DidChangeConfigurationNotification.type, undefined);
	publishLocalization();
});
connection.onDidChangeWatchedFiles(() => { localization.rebuild(); publishLocalization(); });
connection.onDidChangeConfiguration(() => { localization.rebuild(); publishLocalization(); });
connection.workspace.onDidChangeWorkspaceFolders(event => {
	workspaceRoots = [...workspaceRoots.filter(uri => !event.removed.some(f => f.uri === uri)), ...event.added.map(f => f.uri)];
	localization.setRoots(workspaceRoots); publishLocalization();
});

// ── Document synchronisation ──────────────────────────────────────

documents.onDidChangeContent((change) => {
	validateDocument(change.document);
});

documents.onDidClose((event) => {
	docCache.delete(event.document.uri);
	localization.close(event.document.uri);
	publishLocalization();
});

function validateDocument(textDocument: TextDocument): void {
	const text = textDocument.getText();
	if (!localization.update(textDocument.uri, text, textDocument.version)) return;
	if (textDocument.uri.endsWith('.StoryScript')) docCache.set(textDocument.uri, parseDocument(text, textDocument.uri));
	publishLocalization();
}

function getDocInfo(uri: string): DocumentInfo {
	const cached = docCache.get(uri);
	if (cached) return cached;
	const doc = documents.get(uri);
	if (doc) {
		const info = parseDocument(doc.getText(), uri);
		docCache.set(uri, info);
		return info;
	}
	return { scenes: [], variables: [], actors: [], directives: [], sceneRefs: [], diagnostics: [] };
}

// ── Completions ───────────────────────────────────────────────────

connection.onCompletion((params: CompletionParams) => {
	const doc = documents.get(params.textDocument.uri);
	if (!doc) return [];
	const info = getDocInfo(params.textDocument.uri);
	const localized = localization.completions(doc.uri, params.position);
	if (localized.length || !doc.uri.endsWith('.StoryScript')) return localized;
	return getCompletions(info, doc.getText(), params.position);
});

// ── Hover ─────────────────────────────────────────────────────────

connection.onHover((params: HoverParams) => {
	const doc = documents.get(params.textDocument.uri);
	if (!doc) return null;
	const info = getDocInfo(params.textDocument.uri);
	return getHover(info, doc.getText(), params.position);
});

// ── Go to Definition ──────────────────────────────────────────────

connection.onDefinition((params: DefinitionParams) => {
	const doc = documents.get(params.textDocument.uri);
	if (!doc) return null;
	const info = getDocInfo(params.textDocument.uri);
	const localized = localization.definitions(doc.uri, params.position);
	return localized.length ? localized : getDefinition(info, doc.getText(), params.position, params.textDocument.uri);
});

// ── References ────────────────────────────────────────────────────

connection.onReferences((params: ReferenceParams) => {
	const doc = documents.get(params.textDocument.uri);
	if (!doc) return [];
	const info = getDocInfo(params.textDocument.uri);
	const localized = localization.references(doc.uri, params.position, params.context.includeDeclaration);
	return localized.length ? localized : getReferences(info, doc.getText(), params.position, params.textDocument.uri);
});

// ── Document Symbols ──────────────────────────────────────────────

connection.onDocumentSymbol((params: DocumentSymbolParams) => {
	const info = getDocInfo(params.textDocument.uri);
	return getDocumentSymbols(info);
});
connection.onWorkspaceSymbol(params => localization.symbols(params.query));
connection.onPrepareRename(params => localization.prepareRename(params.textDocument.uri, params.position));
connection.onRenameRequest(params => localization.rename(params.textDocument.uri, params.position, params.newName));
connection.onCodeAction(params => localization.sync(params.textDocument.uri));

// ── Start ─────────────────────────────────────────────────────────

documents.listen(connection);
connection.listen();
