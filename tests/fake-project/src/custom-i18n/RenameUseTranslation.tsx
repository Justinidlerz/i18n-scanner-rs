import { useTranslation as useTrans } from '@custom/i18n';

const App = () => {
  const { t } = useTrans();
  return (
    <div>
      <h1>{t('RENAME_USE_TRANSLATION')}</h1>
    </div>
  );
};

export default App;
