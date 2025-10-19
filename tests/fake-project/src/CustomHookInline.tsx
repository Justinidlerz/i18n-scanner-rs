import { useTranslation as useTrans } from '@custom/i18n';

const useTranslation = () => {
  return  useTrans('namespace_4');
}

const Comp = () => {
  const {t} = useTranslation();

  return <div>
    {t('CUSTOM_HOOK_INLINE')}
  </div>
}

export default Comp;