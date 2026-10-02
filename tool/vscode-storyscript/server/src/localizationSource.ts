/** Source-only lexical inventory. Edits use token offsets, never text replacement. */
export interface Token { value: string; start: number; end: number; kind: string }
export interface SourceSite { id: string; start: number; end: number; variables: Map<string, string> }
export interface SourceInventory { sites: SourceSite[]; includes: string[]; globals: Map<string, string> }

export function tokens(text: string): Token[] {
    const result: Token[] = [];
    let i = 0;
    while (i < text.length) {
        if (result.length >= 100000) throw new Error('source token analysis limit');
        if (/\s/.test(text[i])) { i++; continue; }
        if (text.startsWith('//', i)) { while (i < text.length && text[i] !== '\n') i++; continue; }
        const start = i;
        const keyed = text.startsWith('@"', i);
        if (keyed || text[i] === '"') {
            i += keyed ? 2 : 1;
            const content = i;
            while (i < text.length && text[i] !== '"') {
                if (text[i] === '\\') i++;
                i++;
            }
            result.push({ value: text.slice(content, i), start: content, end: i, kind: keyed ? 'keyed' : 'string' });
            i = Math.min(i + 1, text.length);
        } else if (/[A-Za-z_]/.test(text[i])) {
            while (i < text.length && /[A-Za-z0-9_-]/.test(text[i])) i++;
            result.push({ value: text.slice(start, i), start, end: i, kind: 'word' });
        } else {
            result.push({ value: text[i++], start, end: i, kind: 'punctuation' });
        }
    }
    return result;
}

export function sourceInventory(text: string, globals = new Map<string, string>()): SourceInventory {
    const ts = tokens(text);
    const includes: string[] = [];
    const ownGlobals = new Map<string, string>();
    const locals = new Map<string, Map<string, string>>();
    let scene = '';
    let phase = '';
    // PREP declarations are scene-scoped, including conditional declarations (Rust inventory rule).
    for (let i = 0; i < ts.length; i++) {
        if (ts[i].value === '*' && ts[i + 1]?.kind === 'word' && ts[i + 2]?.value === '{') { scene = ts[i + 1].value; phase = ''; }
        if (ts[i].value === '#') phase = ts[i + 1]?.value ?? '';
        if (ts[i].value === '@' && ts[i + 1]?.value === 'include' && ts[i + 2]?.value === '[') {
            for (let j = i + 3; j < ts.length && ts[j].value !== ']'; j++) {
                if (ts[j].kind === 'string') includes.push(ts[j].value);
                else if (ts[j].value !== ',') break;
            }
        }
        if (ts[i].value === '$' && ts[i + 2]?.value === 'as') {
            const name = ts[i + 1].value;
            let type = ts[i + 3]?.value ?? '';
            if (type === 'array') type = `array<${ts[i + 5]?.value}>`;
            if (scene === 'INIT') ownGlobals.set(name, type);
            else if (phase === 'PREP') {
                if (!locals.has(scene)) locals.set(scene, new Map());
                locals.get(scene)!.set(name, type);
            }
        }
    }
    const allGlobals = ownGlobals.size ? new Map([...globals, ...ownGlobals]) : globals;
    const sceneScopes = new Map<string, Map<string, string>>();
    let scopeWork = allGlobals.size;
    for (const [name, variables] of locals) {
        scopeWork += allGlobals.size + variables.size;
        if (scopeWork > 100000) throw new Error('source scope analysis limit');
        sceneScopes.set(name, new Map([...allGlobals, ...variables]));
    }
    const sites: SourceSite[] = [];
    const frames: Map<string, string>[] = [];
    let pendingLoop: { name: string; array: string } | undefined;
    scene = '';
    for (let i = 0; i < ts.length; i++) {
        const token = ts[i];
        if (token.value === '*' && ts[i + 1]?.kind === 'word' && ts[i + 2]?.value === '{') { scene = ts[i + 1].value; frames.length = 0; }
        const header = i + (ts[i + 1]?.value === '(' ? 2 : 1);
        if (token.value === 'for' && ts[header]?.value === '$' && ts[header + 2]?.value === 'in' && ts[header + 3]?.value === 'snapshot' && ts[header + 4]?.value === '$') {
            pendingLoop = { name: ts[header + 1].value, array: ts[header + 5]?.value };
        }
        if (token.value === '{') {
            // Share immutable visible scope snapshots across sites/ordinary blocks.
            const parent = frames[frames.length - 1] ?? sceneScopes.get(scene) ?? allGlobals;
            let frame = parent;
            if (pendingLoop) {
                const type = parent.get(pendingLoop.array);
                if (type?.startsWith('array<')) {
                    scopeWork += parent.size + 1;
                    if (scopeWork > 100000) throw new Error('source scope analysis limit');
                    frame = new Map(parent);
                    frame.set(pendingLoop.name, type.slice(6, -1));
                }
                pendingLoop = undefined;
            }
            frames.push(frame);
        }
        if (token.value === '}') frames.pop();
        if (token.kind === 'keyed') sites.push({ id: token.value, start: token.start, end: token.end,
            variables: frames[frames.length - 1] ?? sceneScopes.get(scene) ?? allGlobals });
    }
    return { sites, includes, globals: ownGlobals };
}
