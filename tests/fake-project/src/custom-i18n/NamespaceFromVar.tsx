import { useTranslation } from '@custom/i18n';

const ns = 'namespace_3';

const App = () => {
  const { t } = useTranslation();
  return (
    <div>
      <h1>
        {t('NAMESPACE_FROM_VAR', {
          ns,
        })}
      </h1>
    </div>
  );
};

export default App;
