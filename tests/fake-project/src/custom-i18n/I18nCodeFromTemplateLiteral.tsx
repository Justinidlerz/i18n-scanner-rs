import { useTranslation } from '@custom/i18n';

const key1 = 'I18N_CODE_FROM';
const key2 = 'TEMPLATE_LITERAL';

const App = () => {
  const { t } = useTranslation();
  return (
    <div>
      <h1>{t(`${key1}_${key2}`)}</h1>
    </div>
  );
};

export default App;
