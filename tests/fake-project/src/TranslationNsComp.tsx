import { Translation } from 'react-i18next'

const TranslationNsComp = () => {
  return <Translation ns={'namespace_translation'}>{(t) => <p>{t('TRANSLATION_COMPONENT_WITH_NS')}</p>}</Translation>
}

export default TranslationNsComp
