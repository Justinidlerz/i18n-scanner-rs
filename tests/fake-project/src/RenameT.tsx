import { useTranslation } from 'react-i18next';

const App = () => {
  const { t: trans } = useTranslation();
  return (
    <div>
      <h1>{trans('RENAME_T')}</h1>
    </div>
  );
};

export default App;
