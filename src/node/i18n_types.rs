/// Preset 5 types for collecting translation keys:
/// 1. Hook
///     : const { t } = useTranslation(ns);
/// 2. GlobalT
///     : t('key', { ns: 'ns' });
/// 3. TranslationComp
///     : <Translation ns="ns">
///         {(t) => {
///             return t(key)
///          }}
///       </Translation>
/// 4. TransComp
///     : <Trans i18nKey="key" ns="ns"></Trans>
/// 5. HocWrapper
///     :
///     const Component = ({ t }) => {
///         return <div>{t(key)}</div>
///     }
///     const hocWrapper = withTranslation(ns)(Component);
/// 6. ObjectMemberT
///     : i18n.t('key', { ns: 'ns' })
///
/// Those 6 types are calls `t` method to translate except `TransComp`

#[derive(Clone, Debug)]
#[napi(string_enum)]
pub enum I18nType {
  Hook,
  TMethod,
  TransComp,
  TranslationComp,
  HocWrapper,
  ObjectMemberT
}

#[derive(Clone, Debug)]
pub struct I18nMember {
  pub r#type: I18nType,
  pub ns: Option<String>,
}