import WrapUseTranslation from './WrapUseTranslation/Component';
import WrapUseTranslationNs from './WrapUseTranslationNs/Component';
import I18nCodeCrossFile from './I18nCodeCrossFile/Component';
import { globalT } from './globalT';
import HocComp from './HocComp';
import I18nCodeFromStringLiteral from './I18nCodeFromStringLiteral';
import I18nCodeFromTemplateLiteral from './I18nCodeFromTemplateLiteral';
import NothingAboutI18n from './NothingAboutI18n';
import RenameBoth from './RenameBoth';
import RenameT from './RenameT';
import RenameUseTranslation from './RenameUseTranslation';
import TransComp from './TransComp';
import TranslationComp from './TranslationComp';
import I18nCodeDynamic from './I18nCodeDynamic';
import MemberCallT from './MemberCallT';
import NamespaceImport from './NamespaceImport';
import { init } from './i18nInstanceInitOnly';
import HookWithNamespace from './HookWithNamespace';
import TWithNamespace from './TWithNamespace';
import NamespaceOverride from './NamespaceOverride';
import { memberT } from './memberT';
import NamespaceFromVar from './NamespaceFromVar';
import CustomI18n from './CustomI18n';

init();

const Entry = () => {
  return (
    <>
      {memberT}
      {globalT}
      <NamespaceFromVar />
      <NamespaceOverride />
      <TWithNamespace />
      <HookWithNamespace />
      <I18nCodeDynamic />
      <WrapUseTranslation />
      <WrapUseTranslationNs />
      <HocComp />
      <I18nCodeFromStringLiteral />
      <I18nCodeFromTemplateLiteral />
      <NothingAboutI18n />
      <RenameBoth />
      <RenameT />
      <RenameUseTranslation />
      <TransComp />
      <TranslationComp />
      <MemberCallT />
      <NamespaceImport />
      <I18nCodeCrossFile />
      <CustomI18n />
    </>
  );
};

export default Entry;
