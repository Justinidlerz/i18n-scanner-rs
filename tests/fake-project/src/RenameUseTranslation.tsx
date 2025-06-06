import { useTranslation as useTrans } from 'react-i18next';

const App = () => {
  const { t } = useTrans();
  return (
    <div>
      <h1>{t('RENAME_USE_TRANSLATION')}</h1>
    </div>
  );
};

export default App;
