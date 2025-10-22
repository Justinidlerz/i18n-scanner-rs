export type TFunction = (key: string, options?: Record<string, unknown>) => string;

export const t: TFunction = (key) => key;

export interface UseTranslationResult {
  t: TFunction;
  i18n: { t: TFunction };
}

export function useTranslation(_namespace?: string | string[]): any {
  const scopedT: TFunction = (key) => key;
  return {
    t: scopedT,
    i18n: { t: scopedT },
  };
}

export interface TranslationProps {
  children: (t: TFunction, opts: { i18n: { t: TFunction } }) => unknown;
}

export function Translation(props: TranslationProps) {
  props.children(t, { i18n: { t } });
  return null;
}

export interface TransProps {
  i18nKey: string;
}

export function Trans(_props: TransProps) {
  return null;
}

export function withTranslation(_namespace?: string | string[]) {
  return function withTranslationHoc<ComponentType>(Component: ComponentType): ComponentType {
    return Component;
  };
}

export const i18n = {
  t,
  init() {
    return Promise.resolve();
  },
};

export default {
  t,
  useTranslation,
  Trans,
  Translation,
  withTranslation,
  i18n,
};
