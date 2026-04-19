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

// ── Connection & document store ───────────────────────────────────

const connection = createConnection(ProposedFeatures.all);
const documents = new TextDocuments(TextDocument);

// Cache parsed results per document URI
const docCache = new Map<string, DocumentInfo>();

// ── Initialization ────────────────────────────────────────────────

connection.onInitialize((_params: InitializeParams): InitializeResult => {
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
		},
	};
});

connection.onInitialized(() => {
	connection.client.register(DidChangeConfigurationNotification.type, undefined);
});

// ── Document synchronisation ──────────────────────────────────────

documents.onDidChangeContent((change) => {
	validateDocument(change.document);
});

documents.onDidClose((event) => {
	docCache.delete(event.document.uri);
	connection.sendDiagnostics({ uri: event.document.uri, diagnostics: [] });
});

function validateDocument(textDocument: TextDocument): void {
	const text = textDocument.getText();
	const info = parseDocument(text, textDocument.uri);
	docCache.set(textDocument.uri, info);

	connection.sendDiagnostics({
		uri: textDocument.uri,
		diagnostics: info.diagnostics,
	});
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
	return getDefinition(info, doc.getText(), params.position, params.textDocument.uri);
});

// ── References ────────────────────────────────────────────────────

connection.onReferences((params: ReferenceParams) => {
	const doc = documents.get(params.textDocument.uri);
	if (!doc) return [];
	const info = getDocInfo(params.textDocument.uri);
	return getReferences(info, doc.getText(), params.position, params.textDocument.uri);
});

// ── Document Symbols ──────────────────────────────────────────────

connection.onDocumentSymbol((params: DocumentSymbolParams) => {
	const info = getDocInfo(params.textDocument.uri);
	return getDocumentSymbols(info);
});

// ── Start ─────────────────────────────────────────────────────────

documents.listen(connection);
connection.listen();
