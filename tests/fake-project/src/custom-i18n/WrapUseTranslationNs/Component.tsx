import { useFeTranslation } from './hook';

const Component = () => {
  const { t } = useFeTranslation();
  return (
    <div>
      <h1>{t('WRAPPED_USE_TRANSLATION_NS')}</h1>
    </div>
  );
};

export default Component;
