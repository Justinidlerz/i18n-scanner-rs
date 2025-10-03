import { useTranslation } from '@custom/i18n';

const key = 'MEMBER_CALL_T';
const App = () => {
  const trans = useTranslation();
  return (
    <div>
      <h1>{trans.t(key)}</h1>
    </div>
  );
};

export default App;
