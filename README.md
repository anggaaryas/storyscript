# storycript-spec
The storyScript is Game-Development tool for prototyping Game's story. especially for visual novel Game.

## Build `.vsix`

From the extension folder:

```bash
cd tool/vscode-storyscript
npm install --save-dev @vscode/vsce
npx vsce package
```

That creates a `.vsix` file in the same folder.