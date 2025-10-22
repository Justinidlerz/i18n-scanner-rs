import { useTranslation } from '@custom/i18n';

const App = () => {
  const [t] = useTranslation();
  return (
    <div>
      <h1>{t('T_ARRAY_FROM_CUSTOM')}</h1>
    </div>
  );
};

export default App;
