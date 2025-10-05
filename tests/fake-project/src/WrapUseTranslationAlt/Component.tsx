import { useTranslationCustom } from './hooks/useTranslationCustom';

const Component = () => {
  const content = useTranslationCustom('USE_TRANSLATION_ALT');
  return (
    <div>
      <h1>{content}</h1>
    </div>
  );
};

export default Component;
