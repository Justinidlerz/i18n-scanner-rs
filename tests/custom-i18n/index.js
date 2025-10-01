const createT = (ns) => (key, options = {}) => ({ key, ns: options.ns ?? ns ?? 'default' });

const useTranslation = (ns) => ({
  t: createT(ns),
});

const t = createT();

const withTranslation = (ns) => (Component) => Component;

const Trans = () => null;
const Translation = ({ children }) => (typeof children === 'function' ? children(createT()) : null);

const i18n = {
  t,
};

module.exports = {
  useTranslation,
  t,
  withTranslation,
  Trans,
  Translation,
  i18n,
};
