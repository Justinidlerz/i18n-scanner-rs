import { useTranslation } from 'react-i18next';
import { key } from './constants';

const App = () => {
  const { t } = useTranslation();
  return (
    <div>
      <h1>{t(key)}</h1>
    </div>
  );
};

export default App;
