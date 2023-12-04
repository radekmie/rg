declare module 'bundle-text:*' {
  const source: string;
  export = source;
}

declare module 'url:*' {
  const source: string;
  export = source;
}

declare module '*.module.css' {
  const css: Record<string, string>;
  export = css;
}
