# Canister Metadata

## Overview

Canisters can store custom metadata, which is available from the state tree at `/canister/<canister id>/metadata/<name>`.

You can configure this metadata in dfx.json, per canister, in the `metadata` array.

Here is a simple example:

```json
{
  "canisters": {
    "app_backend": {
      "main": "src/app_backend/main.mo",
      "type": "motoko"
    },
    "app_frontend": {
      "dependencies": [
        "app_backend"
      ],
      "frontend": {
        "entrypoint": "src/app_frontend/src/index.html"
      },
      "source": [
        "src/app_frontend/assets",
        "dist/app_frontend/"
      ],
      "type": "assets",
      "metadata": [
        {
          "name": "alternative-domains",
          "visibility": "public",
          "path": "src/app_frontend/metadata/alternative-domains.cbor"
        }
      ]
    }
  },
  "version": 1
}
```
## Fields

The JSON schema also documents these fields.

### name

A string containing the name of the wasm section.

### visibility

A string containing either `private` or `public` (the default).

Anyone can read the public metadata of a canister.

Only a controller of the canister can read its private metadata.

It is not possible to define metadata with the same name with both `private` and `public` visibility, unless they are for different networks.

### networks

An array of strings containing the names of the networks that this metadata applies to.

If this field is absent, it applies to all networks.

If this field is present as an empty array, it does not apply to any networks.

If dfx.json contains more than one metadata entry with a given name, dfx will use the first entry that matches the current network and ignore any that follow.

### path

A string containing the path of a file containing the wasm section contents. Conflicts with "content".

### content

A string containing the wasm section content directly. Conflicts with "path".

## The candid:service metadata

Dfx automatically adds `candid:service` metadata, with public visibility, for Rust and Motoko canisters.

You can, however, override this behavior by defining a metadata entry with `"name": "candid:service"`.  You can change the visibility or the contents.

For Motoko canisters, if you specify a `path` for candid:service metadata (replacing the candid:service definition generated by `moc`), dfx will verify that the candid:service definition you provide is a valid subtype of the definition that `moc` generated.

## Canister Metadata Standard

`dfx` relies on several pre-defined **public** metadata sections.

The section names are prefixed with `dfx:` to avoid conflict with other metadata usage. Contents should be valid UTF-8 text.

When developer declares a canister to be `pull_ready`, following metadata sections will be added to the wasm module.

### `dfx:wasm_url`

A URL to download canister Wasm module which will be deployed locally.

This section must be set explicitly.

### `dfx:wasm_hash`

SHA256 hash of the Wasm module to be downloaded from `dfx:wasm_url`.

This section is optional. It is required when the Wasm module to be downloaded is different from the canister on chain.

When this metadata is provided, it will be used to verify the integrity of the downloaded Wasm module.

When it is not provided, the hash of the on chain canister will be used.

### `dfx:deps`

A list of `name:ID` pairs of direct dependencies separated by semicolon.

This section must not be set in `dfx.json`. `dfx` will generate the content.

### `dfx:init`

A message to guide consumers how to initialize the canister.

This section must be set explicitly.


## A more complex example

In this example, we change the visibility of the `candid:service` metadata on the ic and staging networks to private, but leave it public for the local network.

`dfx:wasm_url` and `dfx:init` are set with default visibility - public.

`dfx` resolves dependencies and adds `dfx:deps` section with content `dep1:rrkah-fqaaa-aaaaa-aaaaq-cai;dep2:ryjl3-tyaaa-aaaaa-aaaba-cai;`.

```json
{
  "canisters": {
    "app_backend": {
      "main": "src/app_backend/main.mo",
      "type": "motoko",
      "pull_ready": true,
      "dependencies": [
        "dep1",
        "dep2"
      ],
      "metadata": [
        {
          "name": "candid:service",
          "networks": [ "ic", "staging" ],
          "visibility": "private"
        },
        {
          "name": "candid:service",
          "networks": [ "local" ],
          "visibility": "public"
        },
        {
          "name": "dfx:wasm_url",
          "content": "example.com/e2e_project.wasm"
        },
        {
          "name": "dfx:init",
          "content": "NA"
        }
      ]
    },
    "app_frontend": {
      "dependencies": [
        "app_backend"
      ],
      "frontend": {
        "entrypoint": "src/app_frontend/src/index.html"
      },
      "source": [
        "src/app_frontend/assets",
        "dist/app_frontend/"
      ],
      "type": "assets"
    },
    "dep1": {
      "type": "pull",
      "id": "rrkah-fqaaa-aaaaa-aaaaq-cai"
    },
    "dep2": {
      "type": "pull",
      "id": "ryjl3-tyaaa-aaaaa-aaaba-cai"
    }
  },
  "version": 1
}
```
