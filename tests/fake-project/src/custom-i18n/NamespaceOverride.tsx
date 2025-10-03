import { useTranslation } from '@custom/i18n';

const App = () => {
  const { t } = useTranslation('namespace_1');
  return (
    <div>
      <h1>
        {t('NAMESPACE_OVERRIDE', {
          ns: 'namespace_2',
        })}
      </h1>
    </div>
  );
};

export default App;
