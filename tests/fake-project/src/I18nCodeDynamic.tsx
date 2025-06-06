import { useTranslation } from 'react-i18next';

const keyPrefix = 'I18N_CODE_DYNAMIC';
const data = ['hello', 'world'];

const App = () => {
  const { t } = useTranslation();
  return (
    <div>
      <h1>{data.map((v) => t(keyPrefix + '_' + v))}</h1>
    </div>
  );
};

export default App;
