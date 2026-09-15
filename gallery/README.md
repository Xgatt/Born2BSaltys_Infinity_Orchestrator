# Gallery feed

This folder is the source of the curated modlist gallery BIO shows on the Install page. It is laid out so it could be lifted into its own repository unchanged.

## Layout

```
gallery/
  README.md
  index.json            generated, committed
  <id>/
    entry.json           maintainer metadata
    modlist.biolist       the modlist file, exported from BIO
    cover.png             optional card art
```

BIO reads only `index.json`. Nothing else in this folder is read by the app.

## `entry.json` fields

| Field | Meaning | Rule |
|---|---|---|
| `id` | folder name, the entry's permanent key | lowercase letters, digits, dashes; must equal the folder name; unique |
| `name` | card title | 1 to 80 characters |
| `author` | handle shown as "by ..." | 1 to 80 characters |
| `description` | card text and the Details page description | 1 to 500 characters |
| `tags` | pills on the card, searchable | up to 6, each up to 20 characters |
| `game` | `BGEE`, `BG2EE`, `IWDEE` or `EET` | must match the game inside the modlist file |
| `featured` | shows first and passes the "Featured only" filter | true or false |
| `version` | the list's own version, shown on Details | free text, 1 to 20 characters |
| `requirements` | optional; the "Requires" fact on Details | when absent, the game's default sentence is used |

## Commands

```
cargo run --release --bin gallery-index -- build gallery
cargo run --release --bin gallery-index -- check gallery
```

`build` validates every folder and regenerates `index.json`. `check` runs the same validation and fails if the committed `index.json` is stale.

Submissions arrive on the BIO Discord; the maintainer places the file here.
