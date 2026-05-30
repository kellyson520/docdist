/// <reference types="vite/client" />
/// <reference types="vitest" />

declare module '*.svg' {
  const content: string;
  export default content;
}

declare module '*.png' {
  const content: string;
  export default content;
}
