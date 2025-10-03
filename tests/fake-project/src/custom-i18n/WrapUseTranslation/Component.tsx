import { useTranslationCustom } from './hook';

const Component = () => {
  const content = useTranslationCustom('USE_TRANSLATION');
  return (
    <div>
      <h1>{content}</h1>
    </div>
  );
};

export default Component;
