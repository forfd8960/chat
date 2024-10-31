
# Run Test

```sh

chat (main) [101]> cargo test --package chat_server --lib -- models::workspace::tests::test_workspace_should_create --exact --show-output

running 1 test
test models::workspace::tests::test_workspace_should_create ... ok

successes:

---- models::workspace::tests::test_workspace_should_create stdout ----
Current directory: /Users/zhiruchen/Documents/Code/github/forfd8960/chat/chat_server
start create workspace...
created workspace: Workspace { id: 5, name: "test-ws2", owner_id: 2, created_at: 2024-10-31T06:43:51.368004Z }
get workspace: Workspace { id: 5, name: "test-ws2", owner_id: 2, created_at: 2024-10-31T06:43:51.368004Z }
```
