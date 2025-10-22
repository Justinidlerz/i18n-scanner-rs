import { scan } from './index.js'

const res = scan({
  entryPaths: [
    '/Users/bytedance/code/logistics_seller_fe_us/apps/logistics_pc_mpa/src/entries/logistic-detail/index.tsx',
  ],
  tsconfigPath: '/Users/bytedance/code/logistics_seller_fe_us/apps/logistics_pc_mpa/tsconfig.json',
  externals: ['react', 'react-dom', 'moment-timezone', '@arco-design/iconbox-react-m4b-next', '@atlas-kernel/helper'],
  extendI18NPackages: [
    {
      packagePath: '@jupiter/plugin-runtime/i18n',
      members: [
        { name: 'useTranslation', type: 'Hook' },
        { name: 't', type: 'TMethod' },
        { name: 'Trans', type: 'TransComp' },
        { name: 'Translation', type: 'TranslationComp' },
        { name: 'withTranslation', type: 'HocWrapper' },
        { name: 'i18n', type: 'ObjectMemberT' },
      ],
    },
    {
      packagePath: '@jupiter-app/plugin-i18n',
      members: [
        { name: 'useTranslation', type: 'Hook' },
        { name: 't', type: 'TMethod' },
        { name: 'Trans', type: 'TransComp' },
        { name: 'Translation', type: 'TranslationComp' },
        { name: 'withTranslation', type: 'HocWrapper' },
        { name: 'i18n', type: 'ObjectMemberT' },
      ],
    },
    {
      packagePath: '@jupiter/plugin-i18n',
      members: [
        { name: 'useTranslation', type: 'Hook' },
        { name: 't', type: 'TMethod' },
        { name: 'Trans', type: 'TransComp' },
        { name: 'Translation', type: 'TranslationComp' },
        { name: 'withTranslation', type: 'HocWrapper' },
        { name: 'i18n', type: 'ObjectMemberT' },
      ],
    },
    {
      packagePath: '@atlas-kernel/i18n',
      members: [
        { name: 'useTranslation', type: 'Hook' },
        { name: 't', type: 'TMethod' },
        { name: 'Trans', type: 'TransComp' },
        { name: 'Translation', type: 'TranslationComp' },
        { name: 'withTranslation', type: 'HocWrapper' },
        { name: 'i18n', type: 'ObjectMemberT' },
      ],
    },
  ],
})

console.log(res);