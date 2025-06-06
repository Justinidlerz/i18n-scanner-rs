import { Translation } from 'react-i18next';

function MyComponent() {
  return (
    <Translation>{(t) => <p>{t('TRANSLATION_COMPONENT')}</p>}</Translation>
  );
}

export default MyComponent;
