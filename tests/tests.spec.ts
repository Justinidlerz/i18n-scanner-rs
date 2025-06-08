import {describe, it, expect} from "vitest";
import {scan} from '..'
import * as path from 'node:path'

const root = path.join(__dirname, './fake-project');
const tsconfigPath = path.join(root, 'tsconfig.json');

describe("I18n-scanner-rs", () => {
    it('Should collect matched snapshot', () => {
        const result = scan({
            entryPaths: [path.join(root, './src/index.tsx')],
            tsconfigPath,
            externals: [],
        });
        const sortedResult = Object.fromEntries(Object.entries(result).map(([k, v]) => [k, v.sort()]));
        expect(sortedResult).toMatchInlineSnapshot(`
          {
            "default": [
              "GLOBAL_T",
              "I18N_CODE_FROM_STRING_LITERAL",
              "MEMBER_CALL_T",
              "MEMBER_T",
              "RENAME_BOTH",
              "RENAME_T",
              "RENAME_USE_TRANSLATION",
            ],
            "namespace_1": [
              "HOOK_WITH_NAMESPACE",
              "T_WITH_NAMESPACE",
            ],
            "namespace_2": [
              "NAMESPACE_OVERRIDE",
            ],
            "namespace_3": [
              "NAMESPACE_FROM_VAR",
              "WRAPPED_USE_TRANSLATION_NS",
            ],
          }
        `)
    })
})