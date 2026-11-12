// The page's strings come from the WebAssembly package (`i18n_catalog`): the page itself
// holds no translated text. Everything is inserted as text, never as markup.

export class Strings {
  /** @param {Record<string, string>} map @param {string} locale */
  constructor(map, locale) {
    this.map = map;
    this.locale = locale;
  }

  has(key) {
    return Object.prototype.hasOwnProperty.call(this.map, key);
  }

  /** The text of `key` with `{name}` placeholders replaced in a single pass. */
  t(key, params = {}) {
    const template = this.has(key) ? this.map[key] : key;
    return template.replace(/\{([a-z0-9_]+)\}/g, (whole, name) =>
      Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : whole);
  }
}

/**
 * Fills `[data-i18n]` elements (text) and `[data-i18n-attr]` elements
 * (`"attribute:key; attribute:key"`).
 */
export function applyStrings(root, strings) {
  for (const element of root.querySelectorAll('[data-i18n]')) {
    element.textContent = strings.t(element.dataset.i18n);
  }
  for (const element of root.querySelectorAll('[data-i18n-attr]')) {
    for (const pair of element.dataset.i18nAttr.split(';')) {
      const [attribute, key] = pair.split(':').map((part) => part.trim());
      if (attribute && key) element.setAttribute(attribute, strings.t(key));
    }
  }
}

/** The supported locale closest to `tag` (`"fr-CA"` → `"fr"`), or `null`. */
export function matchLocale(tag, supported) {
  if (!tag) return null;
  const language = String(tag).toLowerCase().split(/[-_]/)[0];
  return supported.includes(language) ? language : null;
}
