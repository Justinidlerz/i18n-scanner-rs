import { Trans } from 'react-i18next';

function MyComponent() {
  return (
    <Trans
      i18nKey="TRANS_COMPONENT" // optional -> fallbacks to defaults if not provided
      defaults="hello <0>{{what}}</0>" // optional defaultValue
      values={{ what: 'world' }}
      components={[<strong>univers</strong>]}
    />
  );
}

export default MyComponent;
