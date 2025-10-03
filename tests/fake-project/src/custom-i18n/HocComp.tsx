import { withTranslation } from '@custom/i18n';

const HocComp = ({ t }) => {
  return <>{t('HOC_COMPONENT')}</>;
};

export default withTranslation()(HocComp);
