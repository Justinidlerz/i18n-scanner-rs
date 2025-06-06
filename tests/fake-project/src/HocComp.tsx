import { withTranslation } from 'react-i18next';

const HocComp = ({ t }) => {
  return <>{t('HOC_COMPONENT')}</>;
};

export default withTranslation()(HocComp);
