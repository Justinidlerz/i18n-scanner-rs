import { useTranslation, t as globalCustomT } from '@custom/i18n';

const CustomI18n = () => {
  const { t } = useTranslation('custom_namespace');

  const customGlobal = globalCustomT('CUSTOM_GLOBAL_T', { ns: 'custom_namespace' });

  return (
    <>
      {t('CUSTOM_HOOK_KEY')}
      {customGlobal && t('CUSTOM_HOOK_KEY_AGAIN')}
    </>
  );
};

export default CustomI18n;
