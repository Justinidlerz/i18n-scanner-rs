import { useTranslation } from 'react-i18next'

const UseTranslationNsArray = () => {
  const { t } = useTranslation(['namespace_array', 'namespace_unused'])

  return <p>{t('USE_TRANSLATION_NS_ARRAY')}</p>
}

export default UseTranslationNsArray
