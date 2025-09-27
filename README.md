# I18n scanner rs
[![CI](https://github.com/Justinidlerz/i18n-scanner-rs/actions/workflows/CI.yml/badge.svg)](https://github.com/Justinidlerz/i18n-scanner-rs/actions/workflows/CI.yml)

A superfast i18next scanner tool written in Rust, based on [Oxc](https://github.com/oxc-project/oxc)  
The node.js API is implemented by [NAPI](https://github.com/napi-rs/napi-rs)

## How it works
This will follow the below flows to collect all the
I18n contents via passed entry file
1. Analyze all file references from entry files
2. Find all import statements are includes the list below:
    - import * from 'i18next'
    - import * from 'react-i18next'
    - or any you passed packages
3. find out the variable linked to the import statement
4. recursively analyze the variable's references
5. collect the first parameter of i18n function call
   or bypass from another function wrapped by the i18n function

## Usage

```ts
import { scan } from '@i18n-scanner-rs/main'

const payload = {
  tsconfigPath: './tsconfig.json',
  entryPaths: ['./src/index.ts'],
  externals: ['react-i18next'],
  extendI18NPackages: []
}

const result = scan(payload)

console.log(result)
```

## Type declarations 
```ts
export interface Member {
    name: string
    type: I18nType
    ns?: string
}
export interface I18NPackage {
    packagePath: string
    members: Array<Member>
}

export const enum I18nType {
    Hook = 'Hook',
    TMethod = 'TMethod',
    TransComp = 'TransComp',
    TranslationComp = 'TranslationComp',
    HocWrapper = 'HocWrapper',
    ObjectMemberT = 'ObjectMemberT'
}
export interface Payload {
    tsconfigPath: string
    entryPaths: Array<string>
    externals: Array<string>
    extendI18NPackages?: Array<I18NPackage>
}

export declare function scan(payload: Payload): Record<string, Array<string>>
```

## License

For a detailed explanation on how things work, checkout the [Oxc](https://github.com/oxc-project/oxc) and [NAPI](https://github.com/napi-rs/napi-rs) doc

Copyright (c) 2024-present, Idler.zhu