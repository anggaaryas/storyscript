/** Advisory, filesystem-backed workspace intelligence. Rust remains the release authority. */
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { parse as parseToml } from '@iarna/toml';
import { parse, SyntaxNode, Message, Term } from '@fluent/syntax';
import { TextDocument } from 'vscode-languageserver-textdocument';
import { CodeAction, CodeActionKind, CompletionItem, CompletionItemKind, Diagnostic, DiagnosticSeverity,
    CreateFile, Location, Position, Range, SymbolInformation, SymbolKind, TextDocumentEdit, TextEdit, WorkspaceEdit } from 'vscode-languageserver/node';
import { SourceSite, sourceInventory } from './localizationSource';

interface Snapshot { document: TextDocument; version: number | null }
interface Site extends SourceSite { uri: string }
interface Occurrence { id: string; uri: string; range: Range; declaration: boolean }
interface Edge { id: string; bound: Set<string> }
interface RecordInfo { id: string; range: Range; own: Set<string>; edges: Edge[] }
interface Catalog { uri: string; records: Map<string, RecordInfo>; occurrences: Occurrence[]; valid: boolean; missing?: boolean; work: number; expanded: Map<string, { args: Set<string>; used: Set<string> }> }
interface Project { root: string; realRoot: string; config: string; locales: string[]; sites: Site[]; catalogs: Catalog[]; files: Set<string> }
const validId = (id: string) => /^[A-Za-z][A-Za-z0-9_-]*$/.test(id) && Buffer.byteLength(id) <= 256;
const uriFor = (p: string) => pathToFileURL(p).href;
const emptyRange = Range.create(0, 0, 0, 1);

export class LocalizationIndex {
    private roots: string[] = [];
    private open = new Map<string, Snapshot>();
    private snapshots = new Map<string, Snapshot>();
    private projects: Project[] = [];
    readonly diagnostics = new Map<string, Diagnostic[]>();

    setRoots(uris: string[]): void { this.roots = uris.filter(u => u.startsWith('file:')).map(u => fileURLToPath(u)); this.rebuild(); }
    update(uri: string, text: string, version: number): boolean {
        const prior = this.open.get(uri);
        if (prior && prior.version! >= version) return false;
        this.open.set(uri, { document: TextDocument.create(uri, '', version, text), version });
        this.rebuild(); return true;
    }
    close(uri: string): void { this.open.delete(uri); this.rebuild(); }
    private read(p: string): Snapshot {
        const uri = uriFor(p);
        const overlay = this.open.get(uri);
        if (overlay) {
            if (Buffer.byteLength(overlay.document.getText()) > 16 * 1024 * 1024) throw new Error('file exceeds editor 16 MiB limit');
            this.snapshots.set(uri, overlay); return overlay;
        }
        if (fs.statSync(p).size > 16 * 1024 * 1024) throw new Error('file exceeds editor 16 MiB limit');
        const snapshot = { document: TextDocument.create(uri, '', 0, fs.readFileSync(p, 'utf8')), version: null };
        this.snapshots.set(uri, snapshot); return snapshot;
    }
    private diagnostic(uri: string, message: string, range = emptyRange): void {
        const list = this.diagnostics.get(uri) ?? [];
        list.push({ range, severity: DiagnosticSeverity.Error, source: 'storyscript-localization (advisory)', message });
        this.diagnostics.set(uri, list);
    }
    private range(uri: string, start: number, end: number): Range {
        const doc = this.snapshots.get(uri)!.document;
        return Range.create(doc.positionAt(start), doc.positionAt(end));
    }
    private safe(root: string, relative: string): string {
        if (!relative || path.isAbsolute(relative) || relative.includes('\\') || relative.split('/').some(s => !s || s === '.' || s === '..') || relative.normalize('NFC') !== relative) throw new Error('unsafe/noncanonical project path');
        const target = path.resolve(root, relative);
        let ancestor = target;
        while (!fs.existsSync(ancestor)) ancestor = path.dirname(ancestor);
        const real = fs.realpathSync(ancestor);
        const realRoot = fs.realpathSync(root);
        if (real !== realRoot && !real.startsWith(realRoot + path.sep)) throw new Error('symlink escapes project');
        return target;
    }
    rebuild(): void {
        this.snapshots.clear(); this.projects = []; this.diagnostics.clear();
        const configs = new Set<string>();
        let count = 0;
        const walk = (dir: string, depth: number) => {
            if (depth > 32 || ++count > 10000) return;
            for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
                if (entry.isDirectory() && !['node_modules', '.git', 'target', 'build', '.dart_tool', 'out'].includes(entry.name)) walk(path.join(dir, entry.name), depth + 1);
                if (entry.isFile() && entry.name === 'StoryScript.toml') configs.add(path.join(dir, entry.name));
            }
        };
        for (const root of this.roots) { try { walk(root, 0); } catch { /* Removed workspace folder. */ } }
        for (const uri of this.open.keys()) if (uri.endsWith('/StoryScript.toml')) configs.add(fileURLToPath(uri));
        for (const config of [...configs].sort()) this.loadProject(config);
    }
    private loadProject(config: string): void {
        const configUri = uriFor(config);
        try {
            const root = path.resolve(path.dirname(config));
            const data = parseToml(this.read(config).document.getText()) as unknown as { project: { entry: string }; localization?: { root: string; 'default-locale': string; 'supported-locales': string[] } };
            const loc = data.localization;
            if (!loc) return;
            if (Object.keys(loc).some(k => !['root', 'default-locale', 'supported-locales'].includes(k))) throw new Error('unknown localization configuration field');
            const locales = loc['supported-locales'];
            if (!Array.isArray(locales) || !locales.length || locales.length > 64 || new Set(locales).size !== locales.length || !locales.includes(loc['default-locale'])) throw new Error('invalid default/supported locales');
            for (const locale of locales) if (Intl.getCanonicalLocales(locale)[0] !== locale) throw new Error('invalid canonical locale');
            const catalogRoot = this.safe(root, loc.root);
            const project: Project = { root, realRoot: fs.realpathSync(root), config: configUri, locales, sites: [], catalogs: [], files: new Set([configUri]) };
            this.projects.push(project);
            const entry = this.safe(root, data.project.entry);
            const sources = new Map<string, string>();
            const visiting = new Set<string>();
            const canonicalSources = new Set<string>();
            const foldedSources = new Set<string>();
            const visit = (p: string) => {
                if (sources.size > 1000) throw new Error('too many include files');
                if (visiting.has(p)) throw new Error('include cycle');
                if (sources.has(p)) throw new Error('duplicate include');
                const canonical = fs.existsSync(p) ? fs.realpathSync(p) :
                    this.open.has(uriFor(p)) ? p : fs.realpathSync(p);
                const folded = p.normalize('NFC').toLowerCase();
                if (canonicalSources.has(canonical) || foldedSources.has(folded)) throw new Error('duplicate or aliased include');
                canonicalSources.add(canonical); foldedSources.add(folded);
                visiting.add(p);
                const uri = uriFor(p);
                const text = this.read(p).document.getText();
                sources.set(p, text); project.files.add(uri);
                for (const include of sourceInventory(text).includes) {
                    // Rust resolves every include relative to the root entry directory.
                    const includePath = this.safe(root, path.posix.join(path.relative(root, path.dirname(entry)).split(path.sep).join('/'), include));
                    if (include.split('/').includes('..') || path.isAbsolute(include)) throw new Error('unsafe include path');
                    visit(includePath);
                }
                visiting.delete(p);
            };
            visit(entry);
            const globals = sourceInventory(sources.get(entry)!).globals;
            for (const [p, text] of sources) {
                const uri = uriFor(p);
                for (const site of sourceInventory(text, globals).sites) {
                    project.sites.push({ ...site, uri });
                    if (!validId(site.id)) this.diagnostic(uri, 'invalid message ID', this.range(uri, site.start, site.end));
                }
            }
            if (fs.existsSync(catalogRoot)) {
                for (const name of fs.readdirSync(catalogRoot)) if (!locales.map(l => `${l}.ftl`).includes(name)) this.diagnostic(configUri, `unexpected catalog entry '${name}'`);
            }
            for (const locale of locales) {
                const p = this.safe(root, `${loc.root}/${locale}.ftl`);
                const uri = uriFor(p);
                project.files.add(uri);
                try { project.catalogs.push(this.catalog(p)); }
                catch {
                    this.diagnostic(configUri, `missing/unreadable ${locale}.ftl`);
                    if (!fs.existsSync(p) && !this.open.has(uri)) {
                        this.snapshots.set(uri, { document: TextDocument.create(uri, '', 0, ''), version: null });
                        project.catalogs.push({ uri, records: new Map(), occurrences: [], valid: true, missing: true, work: 0, expanded: new Map() });
                    }
                }
            }
            this.validate(project);
        } catch (error) {
            const message = (error as NodeJS.ErrnoException).code ? 'unreadable project resource' : (error as Error).message;
            this.diagnostic(configUri, `Project index: ${message}`);
        }
    }
    private catalog(p: string): Catalog {
        const uri = uriFor(p);
        const text = this.read(p).document.getText();
        // Guard recursive Fluent parsing on hostile editor input.
        let depth = 0;
        for (const ch of text) { if (ch === '{' && ++depth > 128) throw new Error('FTL nesting limit'); if (ch === '}') depth = Math.max(0, depth - 1); }
        const catalog: Catalog = { uri, records: new Map(), occurrences: [], valid: true, work: 0, expanded: new Map() };
        const span = (node: SyntaxNode) => this.range(uri, node.span!.start, node.span!.end);
        const issue = (node: SyntaxNode, message: string) => { catalog.valid = false; this.diagnostic(uri, message, span(node)); };
        for (const entry of parse(text, { withSpans: true }).body) {
            if (entry.type === 'Junk') { issue(entry, 'malformed FTL'); continue; }
            if (entry.type !== 'Message' && entry.type !== 'Term') continue;
            const id = (entry.type === 'Term' ? '-' : '') + entry.id.name;
            const record: RecordInfo = { id, range: span(entry.id), own: new Set(), edges: [] };
            if (!validId(entry.id.name)) issue(entry.id, 'invalid or oversized catalog ID');
            if (catalog.records.has(id)) issue(entry.id, `duplicate catalog ID '${id}'`);
            catalog.records.set(id, record);
            if (catalog.records.size > 100000) throw new Error('too many catalog IDs');
            catalog.occurrences.push({ id, uri, range: record.range, declaration: true });
            if (entry.attributes.length || !entry.value) issue(entry, 'unsupported attributes or missing message value');
            const walk = (node: SyntaxNode, level = 0): void => {
                if (level > 128) { issue(node, 'FTL expression depth limit'); return; }
                if (node.type === 'VariableReference') record.own.add((node.id as { name: string }).name);
                if (node.type === 'MessageReference' || node.type === 'TermReference') {
                    const ref = node as unknown as { id: { name: string; span: { start: number; end: number } }; attribute: unknown; arguments?: { positional: SyntaxNode[]; named: { name: { name: string }; value: SyntaxNode }[] } };
                    if (ref.attribute) issue(node, 'reference attributes are unsupported');
                    if (ref.arguments?.positional.length) issue(node, 'positional term arguments are unsupported');
                    const target = (node.type === 'TermReference' ? '-' : '') + ref.id.name;
                    record.edges.push({ id: target, bound: new Set(ref.arguments?.named.map(a => a.name.name) ?? []) });
                    catalog.occurrences.push({ id: target, uri, range: this.range(uri, ref.id.span.start, ref.id.span.end), declaration: false });
                }
                if (node.type === 'FunctionReference') {
                    const fn = node as unknown as { id: { name: string }; arguments: { positional: SyntaxNode[]; named: { name: { name: string }; value: { type: string; value: string } }[] } };
                    if (fn.id.name !== 'NUMBER' || fn.arguments.positional.length !== 1 || !['VariableReference', 'NumberLiteral'].includes(fn.arguments.positional[0]?.type)) issue(node, 'unsupported function or NUMBER argument');
                    for (const arg of fn.arguments.named) {
                        const grouping = arg.name.name === 'useGrouping' && arg.value.type === 'StringLiteral' && ['true', 'false'].includes(arg.value.value);
                        const precision = ['minimumIntegerDigits', 'minimumFractionDigits', 'maximumFractionDigits', 'minimumSignificantDigits', 'maximumSignificantDigits'].includes(arg.name.name) && arg.value.type === 'NumberLiteral' && /^\d+$/.test(arg.value.value) && Number(arg.value.value) <= 20;
                        if (!grouping && !precision) issue(node, 'unsupported NUMBER option');
                    }
                }
                for (const [key, value] of Object.entries(node)) {
                    if (['span', 'comment', 'id', 'attribute', 'key'].includes(key)) continue;
                    if (value instanceof SyntaxNode) walk(value, level + 1);
                    else if (Array.isArray(value)) for (const child of value) if (child instanceof SyntaxNode) walk(child, level + 1);
                }
            };
            if (entry.value) walk(entry.value);
        }
        return catalog;
    }
    private arguments(catalog: Catalog, id: string, stack = new Set<string>(), used = new Set<string>()): Set<string> {
        const cached = catalog.expanded.get(id);
        if (cached) { for (const key of cached.used) used.add(key); return cached.args; }
        if (++catalog.work > 100000) throw new Error('catalog analysis work limit');
        if (stack.size >= 128 || stack.has(id)) throw new Error('message/term cycle or depth limit');
        const record = catalog.records.get(id);
        if (!record) throw new Error(`missing referenced ID '${id}'`);
        const dependencies = new Set([id]); stack.add(id);
        const args = new Set(record.own);
        for (const edge of record.edges) {
            const names = this.arguments(catalog, edge.id, stack, dependencies);
            if (edge.id.startsWith('-') && ([...names].some(name => !edge.bound.has(name)) || [...edge.bound].some(name => !names.has(name)))) throw new Error('term parameters require complete literal named bindings');
            for (const name of names) if (!edge.bound.has(name)) args.add(name);
        }
        catalog.work += dependencies.size + args.size;
        if (catalog.work > 100000) throw new Error('catalog analysis work limit');
        for (const key of dependencies) used.add(key);
        catalog.expanded.set(id, { args, used: dependencies });
        stack.delete(id); return args;
    }
    private validate(project: Project): void {
        const groups = new Map<string, Site[]>();
        for (const site of project.sites) groups.set(site.id, [...(groups.get(site.id) ?? []), site]);
        for (const sites of groups.values()) if (sites.length > 1) for (const site of sites) this.diagnostic(site.uri, `duplicate source ID '${site.id}'`, this.range(site.uri, site.start, site.end));
        const expected = new Map<string, string>();
        for (const catalog of project.catalogs) {
            const used = new Set<string>();
            for (const record of catalog.records.values()) try { this.arguments(catalog, record.id); } catch (error) { this.diagnostic(catalog.uri, (error as Error).message, record.range); }
            for (const [id, sites] of groups) {
                if (!catalog.records.has(id)) { for (const site of sites) this.diagnostic(site.uri, `missing '${id}' in ${path.basename(fileURLToPath(catalog.uri))}`, this.range(site.uri, site.start, site.end)); continue; }
                try {
                    const args = this.arguments(catalog, id, new Set(), used);
                    const signature = [...args].sort().join(',');
                    if (expected.has(id) && expected.get(id) !== signature) this.diagnostic(catalog.uri, `variable-set drift for '${id}'`, catalog.records.get(id)!.range);
                    else expected.set(id, signature);
                    for (const site of sites) for (const name of args) {
                        const type = site.variables.get(name);
                        if (!type || type.startsWith('array')) this.diagnostic(catalog.uri, `${type ? 'array' : 'out-of-scope'} variable '$${name}' for '${id}'`, catalog.records.get(id)!.range);
                    }
                } catch { /* Dependency diagnostics above retain the catalog range. */ }
            }
            for (const record of catalog.records.values()) if (!used.has(record.id)) this.diagnostic(catalog.uri, `unused catalog ID '${record.id}'`, record.range);
        }
    }
    private project(uri: string): Project | undefined { return this.projects.find(p => p.files.has(uri)); }
    private selected(uri: string, position: Position): { project: Project; id: string; range: Range } | undefined {
        const project = this.project(uri); if (!project) return;
        const offset = this.snapshots.get(uri)?.document.offsetAt(position);
        const site = project.sites.find(s => s.uri === uri && offset !== undefined && s.start <= offset && offset <= s.end);
        if (site) return { project, id: site.id, range: this.range(uri, site.start, site.end) };
        const occurrence = project.catalogs.find(c => c.uri === uri)?.occurrences.find(o =>
            position.line >= o.range.start.line && position.line <= o.range.end.line && position.character >= o.range.start.character && position.character <= o.range.end.character);
        if (occurrence) return { project, id: occurrence.id, range: occurrence.range };
    }
    definitions(uri: string, position: Position): Location[] {
        const selected = this.selected(uri, position); if (!selected) return [];
        return [
            ...(uri.endsWith('.ftl') ? selected.project.sites.filter(s => s.id === selected.id).map(s => Location.create(s.uri, this.range(s.uri, s.start, s.end))) : []),
            ...selected.project.catalogs.flatMap(c => c.occurrences.filter(o => o.id === selected.id && o.declaration).map(o => Location.create(o.uri, o.range))),
        ];
    }
    references(uri: string, position: Position, declarations = true): Location[] {
        const selected = this.selected(uri, position); if (!selected) return [];
        return [ ...selected.project.sites.filter(s => s.id === selected.id).map(s => Location.create(s.uri, this.range(s.uri, s.start, s.end))),
            ...selected.project.catalogs.flatMap(c => c.occurrences.filter(o => o.id === selected.id && (declarations || !o.declaration)).map(o => Location.create(o.uri, o.range))) ];
    }
    prepareRename(uri: string, position: Position): Range | null {
        const selected = this.selected(uri, position);
        if (!selected || selected.id.startsWith('-') || selected.project.sites.filter(s => s.id === selected.id).length !== 1 || selected.project.catalogs.length !== selected.project.locales.length || selected.project.catalogs.some(c => c.missing || !c.valid || !c.records.has(selected.id))) return null;
        return selected.range;
    }
    private fresh(project: Project): boolean {
        try { if (fs.realpathSync(project.root) !== project.realRoot) return false; } catch { return false; }
        for (const uri of project.files) {
            try { this.safe(project.root, path.relative(project.root, fileURLToPath(uri)).split(path.sep).join('/')); } catch { return false; }
            const snapshot = this.snapshots.get(uri); if (!snapshot) return false;
            const open = this.open.get(uri);
            if (open) { if (open.version !== snapshot.version) return false; }
            else { try { if (fs.readFileSync(fileURLToPath(uri), 'utf8') !== snapshot.document.getText()) return false; } catch { if (!project.catalogs.some(c => c.uri === uri && c.missing)) return false; } }
        }
        return true;
    }
    rename(uri: string, position: Position, newName: string): WorkspaceEdit | null {
        const selected = this.selected(uri, position);
        if (!selected || !validId(newName) || !this.prepareRename(uri, position) || !this.fresh(selected.project) ||
            selected.project.sites.some(s => s.id === newName && s.id !== selected.id) || selected.project.catalogs.some(c => c.records.has(newName) && newName !== selected.id)) return null;
        const edits = new Map<string, TextEdit[]>();
        for (const ref of this.references(uri, position)) edits.set(ref.uri, [...(edits.get(ref.uri) ?? []), TextEdit.replace(ref.range, newName)]);
        return { documentChanges: [...edits].map(([target, changes]) => TextDocumentEdit.create({ uri: target, version: this.snapshots.get(target)!.version }, changes)) };
    }
    completions(uri: string, position: Position): CompletionItem[] {
        const project = this.project(uri); if (!project) return [];
        const doc = this.snapshots.get(uri)?.document; if (!doc) return [];
        const prefix = doc.getText().slice(doc.offsetAt(Position.create(position.line, 0)), doc.offsetAt(position));
        if (uri.endsWith('.ftl') && /\$[A-Za-z0-9_]*$/.test(prefix)) {
            const catalog = project.catalogs.find(c => c.uri === uri);
            const offset = doc.offsetAt(position);
            const ast = parse(doc.getText(), { withSpans: true });
            const record = ast.body.find(e => (e.type === 'Message' || e.type === 'Term') && e.span!.start <= offset && e.span!.end >= offset) as Message | Term | undefined;
            // In-progress `$` / missing closing brace becomes Fluent Junk. Use its
            // parser-owned span to find the header for completion only (never edits).
            const junk = ast.body.find(e => e.type === 'Junk' && e.span!.start <= offset && e.span!.end >= offset);
            const incompleteId = junk?.type === 'Junk' ? junk.content.match(/^([A-Za-z][A-Za-z0-9_-]*)\s*=/)?.[1] : undefined;
            const site = project.sites.find(s => s.id === (record?.id.name ?? incompleteId));
            const names = new Set(site ? [...site.variables].filter(([, type]) => !type.startsWith('array')).map(([name]) => name) : []);
            if (record && catalog) try { for (const name of this.arguments(catalog, record.id.name)) names.add(name); } catch { /* Incomplete editing pattern. */ }
            return [...names].sort().map(label => ({ label, kind: CompletionItemKind.Variable }));
        }
        if (!uri.endsWith('.ftl') && !/@"[^"\n]*$/.test(prefix)) return [];
        const names = new Set([...project.sites.map(s => s.id), ...project.catalogs.flatMap(c => [...c.records.keys()].filter(id => !id.startsWith('-')))]);
        return [...names].sort().map(label => ({ label, kind: CompletionItemKind.Text, detail: 'Fluent message ID' }));
    }
    sync(uri: string): CodeAction[] {
        const project = this.project(uri); if (!project || !this.fresh(project) || project.catalogs.length !== project.locales.length || project.catalogs.some(c => !c.valid)) return [];
        const ids = [...new Set(project.sites.map(s => s.id))].filter(validId).sort();
        const changes: (TextDocumentEdit | CreateFile)[] = [];
        for (const catalog of project.catalogs) {
            const missing = ids.filter(id => !catalog.records.has(id));
            if (!missing.length) continue;
            const snapshot = this.snapshots.get(catalog.uri)!;
            const end = snapshot.document.positionAt(snapshot.document.getText().length);
            const text = '\n' + missing.map(id => `# TODO: translate ${id}\n${id} = ${id}\n`).join('\n');
            if (catalog.missing) changes.push(CreateFile.create(catalog.uri, { overwrite: false, ignoreIfExists: false }));
            changes.push(TextDocumentEdit.create({ uri: catalog.uri, version: snapshot.version }, [TextEdit.insert(end, text)]));
        }
        return changes.length ? [{ title: 'Synchronize missing Fluent stubs (preserve translations)', kind: CodeActionKind.QuickFix, edit: { documentChanges: changes } }] : [];
    }
    symbols(query: string): SymbolInformation[] {
        return this.projects.flatMap(p => p.sites.filter(s => s.id.includes(query)).map(s => SymbolInformation.create(s.id, SymbolKind.String, this.range(s.uri, s.start, s.end), s.uri, path.basename(p.root))));
    }
}
