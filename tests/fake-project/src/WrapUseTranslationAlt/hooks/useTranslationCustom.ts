import { useTranslation } from 'react-i18next';

export const useTranslationCustom = (key: string) => {
  const { t } = useTranslation();

  return t(`WRAPPED_${key}`);
};
