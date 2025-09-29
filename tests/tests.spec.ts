console.log('system info', process.platform, process.arch)
import {describe, it, expect} from "vitest";
import {scan} from '../index'
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
              "HOC_COMPONENT",
              "I18N_CODE_CROSS_FILE",
              "I18N_CODE_DYNAMIC_hello",
              "I18N_CODE_DYNAMIC_world",
              "I18N_CODE_FROM_STRING_LITERAL",
              "I18N_CODE_FROM_TEMPLATE_LITERAL",
              "MEMBER_CALL_T",
              "MEMBER_T",
              "NAMESPACE_IMPORT",
              "RENAME_BOTH",
              "RENAME_T",
              "RENAME_USE_TRANSLATION",
              "TRANSLATION_COMPONENT",
              "TRANS_COMPONENT",
              "WRAPPED_USE_TRANSLATION",
              "WRAPPED_USE_TRANSLATION_NS",
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