import { useTranslation as useTrans } from '@custom/i18n';

const App = () => {
  const { t: trans } = useTrans();
  return (
    <div>
      <h1>{trans('RENAME_BOTH')}</h1>
    </div>
  );
};

export default App;
