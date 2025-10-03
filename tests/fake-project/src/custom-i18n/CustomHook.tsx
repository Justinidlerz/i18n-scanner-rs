import { useTranslation } from '@custom/i18n'

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
