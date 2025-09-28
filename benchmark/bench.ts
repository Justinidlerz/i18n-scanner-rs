import {Bench} from 'tinybench'

import {scan} from '../index.js'
import path from "node:path";
import {fileURLToPath} from "node:url";

const b = new Bench()

const __dirname = fileURLToPath(new URL('.', import.meta.url))
const root = path.join(__dirname, '../tests/fake-project');
const tsconfigPath = path.join(root, 'tsconfig.json');

b.add('scan', () => {
    scan({
        entryPaths: [path.join(root, './src/index.tsx')],
        tsconfigPath,
        externals: [],
    })
})

await b.run()

console.table(b.table())
