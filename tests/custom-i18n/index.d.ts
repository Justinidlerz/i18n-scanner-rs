export interface TFunction {
  (key: string, options?: { ns?: string }): unknown;
}

export interface UseTranslationResult {
  t: TFunction;
}

export declare function useTranslation(ns?: string): UseTranslationResult;
export declare const t: TFunction;
export declare function withTranslation(ns?: string): (Component: unknown) => unknown;
export declare const Trans: unknown;
export declare const Translation: unknown;
export declare const i18n: { t: TFunction };
