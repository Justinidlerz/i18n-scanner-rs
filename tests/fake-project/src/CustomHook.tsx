import { useTranslation } from 'react-i18next'

const useInsuranceTranslation = () => {
  return useTranslation('namespace_4')
}

const CustomHook = () => {
  const { t } = useInsuranceTranslation()
  return (
    <>
      <div>
        <span>{t('CUSTOM_HOOK')}</span>
      </div>
    </>
  )
}

export default CustomHook
