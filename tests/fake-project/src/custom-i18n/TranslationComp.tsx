import { Translation } from '@custom/i18n';

function MyComponent() {
  return (
    <Translation>{(t) => <p>{t('TRANSLATION_COMPONENT')}</p>}</Translation>
  );
}

export default MyComponent;
