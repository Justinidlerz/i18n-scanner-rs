import * as i18n from 'react-i18next';

const App = () => {
  const { t: trans } = i18n.useTranslation();
  return (
    <div>
      <h1>{trans('NAMESPACE_IMPORT')}</h1>
    </div>
  );
};

export default App;
