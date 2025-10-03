import { useTranslation } from '@custom/i18n';

const App = () => {
  const { t: trans } = useTranslation();
  return (
    <div>
      <h1>{trans('RENAME_T')}</h1>
    </div>
  );
};

export default App;
