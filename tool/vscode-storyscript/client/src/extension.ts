/* ------------------------------------------------------------------ *
 *  StoryScript LSP — VS Code client extension                         *
 * ------------------------------------------------------------------ */

import * as path from 'path';
import { ExtensionContext, workspace } from 'vscode';
import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext): void {
	const serverModule = context.asAbsolutePath(path.join('out', 'server', 'server.js'));

	const serverOptions: ServerOptions = {
		run: { module: serverModule, transport: TransportKind.ipc },
		debug: {
			module: serverModule,
			transport: TransportKind.ipc,
			options: { execArgv: ['--nolazy', '--inspect=6009'] },
		},
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [
			{ scheme: 'file', language: 'storyscript' },
			{ scheme: 'file', language: 'fluent' },
			{ scheme: 'file', pattern: '**/StoryScript.toml' },
		],
		synchronize: { fileEvents: workspace.createFileSystemWatcher('**/*.{StoryScript,ftl,toml}') },
	};

	client = new LanguageClient(
		'storyscriptLanguageServer',
		'StoryScript Language Server',
		serverOptions,
		clientOptions
	);

	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) return undefined;
	return client.stop();
}
