## CosmWasm contracts template workspace enable

Advanced tool to generate CosmWasm template for advanced project

### Create new repo:

```ssh
cargo generate --git https://github.com/Rhaki/rhaki-cw-template.git --name project_name
```

The repo will be created with `workspace` defined on the primary `Cargo.toml`

### Add a contract into the project:
```
cargo make add-contract
```

This command launches a python script that:
- Create a new folder structure for the contract at the path `./contracts/`;
- Adds the new contract to members in the `workspace`;
- Create a new file inside `./package/src/`
- Insert the export of the new file created in `./package/src/lib.rs`
