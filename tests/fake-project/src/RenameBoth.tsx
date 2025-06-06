import { useTranslation as useTrans } from 'react-i18next';

const App = () => {
  const { t: trans } = useTrans();
  return (
    <div>
      <h1>{trans('RENAME_BOTH')}</h1>
    </div>
  );
};

export default App;
