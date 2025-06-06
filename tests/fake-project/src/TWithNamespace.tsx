import { useTranslation } from 'react-i18next';

const App = () => {
  const { t } = useTranslation();
  return (
    <div>
      <h1>
        {t('T_WITH_NAMESPACE', {
          ns: 'namespace_1',
        })}
      </h1>
    </div>
  );
};

export default App;
