# FabOMX

Parcours d’outils pour la ProtoMAX.

À partir d’un dessin vectoriel (SVG ou DXF), produire un parcours d’outil (fichier OMX) pour la découpeuse jet d’eau ProtoMAX.

État : jalon v0.1 (interface et dessin) en cours. Voir [STATUS.md](STATUS.md) et la feuille de route [ROADMAP.md](ROADMAP.md).

## Lancer

Prérequis : Rust (version fixée par `rust-toolchain.toml`), [wasm-pack](https://wasm-bindgen.github.io/wasm-pack/), Node.js.

```sh
cd ui
npm install
npm run wasm   # compile le moteur en WebAssembly
npm run dev    # http://localhost:5177
```

## Développer

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cd ui && npm run check
```

Consignes de développement : [AGENTS.md](AGENTS.md).

## Organisation

| Dossier | Rôle |
| --- | --- |
| `crates/fabomx-core` | Moteur pur : machine, géométrie, et plus tard parcours et OMX |
| `crates/fabomx-wasm` | Le moteur compilé en WebAssembly pour l’interface |
| `ui/` | Interface Svelte 5 (kit `fab-design-system`) |
| `branding/` | Figure au trait de la carte FabBoard |
| `roadmap/` | Feuille de route lue par FabBoard |

## Licence

GPL-3.0-or-later, comme FabVents, FabSpeed et FabCAM. Voir [LICENSE](LICENSE).
