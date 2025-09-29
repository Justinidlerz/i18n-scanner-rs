import {scan} from '../index.js'
import path from 'node:path'

const __dirname = path.dirname(new URL(import.meta.url).pathname);

const root = path.join(__dirname, './fake-project');
const tsconfigPath = path.join(root, 'tsconfig.json');

const result = scan({
    entryPaths: [path.join(root, './src/index.tsx')],
    tsconfigPath,
    externals: [],
});
const sortedResult = Object.fromEntries(Object.entries(result).map(([k, v]) => [k, v.sort()]));


if (Object.keys(sortedResult).length !== 4) {
    throw new Error('The number of keys in the result should be 4')
}