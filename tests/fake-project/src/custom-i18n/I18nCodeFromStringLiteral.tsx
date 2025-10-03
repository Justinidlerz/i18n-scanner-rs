import { useTranslation } from '@custom/i18n';

const key = 'I18N_CODE_FROM_STRING_LITERAL';
const App = () => {
  const { t } = useTranslation();
  return (
    <div>
      <h1>{t(key)}</h1>
    </div>
  );
};

export default App;

