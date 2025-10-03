import { useTranslation } from '@custom/i18n';
export const useTranslationCustom = (key: string) => {
  const { t } = useTranslation();

  return t(`WRAPPED_${key}`);
};
