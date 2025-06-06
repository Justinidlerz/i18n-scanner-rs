import { useTranslation } from 'react-i18next';

const App = () => {
  const { t } = useTranslation('namespace_1');
  return (
    <div>
      <h1>{t('HOOK_WITH_NAMESPACE')}</h1>
    </div>
  );
};

export default App;
